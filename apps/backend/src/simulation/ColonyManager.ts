import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database, Colony } from '../types/ant-colony'

export class ColonyManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    console.log('ColonyManager initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('ColonyManager not initialized')
    }

    // Get all active colonies for this simulation
    const { data: colonies } = await this.supabase
      .from('colonies')
      .select('*')
      .eq('simulation_id', this.simulationId)
      .eq('is_active', true)

    if (!colonies) return

    // Process each colony
    for (const colony of colonies) {
      await this.processColony(colony, tick)
    }
  }

  private async processColony(colony: Colony, tick: number): Promise<void> {
    try {
      // Update colony population count based on living ants
      const { data: ants } = await this.supabase
        .from('ants')
        .select('id')
        .eq('colony_id', colony.id)
        .neq('state', 'dead')

      const currentPopulation = ants?.length || 0

      if (currentPopulation !== colony.population) {
        await this.supabase
          .from('colonies')
          .update({ population: currentPopulation })
          .eq('id', colony.id)
      }

      // Process resource consumption and growth
      await this.processColonyResources(colony, tick)

      // Check if colony needs to spawn new ants
      await this.handleColonyGrowth(colony, tick)

    } catch (error) {
      console.error(`Error processing colony ${colony.id}:`, error)
    }
  }

  private async processColonyResources(colony: Colony, tick: number): Promise<void> {
    const resources = (colony.resources as any) || {}
    
    // Calculate resource consumption based on population
    const dailyConsumption = {
      seeds: colony.population * 0.1,
      sugar: colony.population * 0.05,
      protein: colony.population * 0.03
    }

    // Consume resources every 100 ticks (simulating daily consumption)
    if (tick % 100 === 0) {
      const updatedResources = {
        seeds: Math.max(0, (resources.seeds || 0) - dailyConsumption.seeds),
        sugar: Math.max(0, (resources.sugar || 0) - dailyConsumption.sugar),
        protein: Math.max(0, (resources.protein || 0) - dailyConsumption.protein)
      }

      await this.supabase
        .from('colonies')
        .update({ resources: updatedResources })
        .eq('id', colony.id)
    }
  }

  private async handleColonyGrowth(colony: Colony, tick: number): Promise<void> {
    const resources = (colony.resources as any) || {}
    const totalResources = (resources.seeds || 0) + (resources.sugar || 0) + (resources.protein || 0)

    // Spawn new ants if colony has enough resources and isn't too crowded
    const shouldSpawnAnt = (
      totalResources > 50 && // Minimum resources needed
      colony.population < 100 && // Population cap
      tick % 200 === 0 // Spawn every 200 ticks
    )

    if (shouldSpawnAnt) {
      await this.spawnAnt(colony)
    }
  }

  private async spawnAnt(colony: Colony): Promise<void> {
    try {
      // Get a random worker ant type
      const { data: antType } = await this.supabase
        .from('ant_types')
        .select('*')
        .eq('role', 'worker')
        .limit(1)
        .single()

      if (!antType) {
        console.warn('No worker ant type found for spawning')
        return
      }

      // Spawn ant near colony center with some randomness
      const spawnRadius = 10
      const randomAngle = Math.random() * 2 * Math.PI
      const randomDistance = Math.random() * spawnRadius

      const spawnX = colony.center_x + Math.cos(randomAngle) * randomDistance
      const spawnY = colony.center_y + Math.sin(randomAngle) * randomDistance

      await this.supabase
        .from('ants')
        .insert({
          colony_id: colony.id,
          ant_type_id: antType.id,
          position_x: spawnX,
          position_y: spawnY,
          current_speed: antType.base_speed,
          health: antType.base_health,
          state: 'wandering',
          energy: 100,
          mood: 'neutral'
        })

      console.log(`Spawned new ant for colony ${colony.name}`)
    } catch (error) {
      console.error(`Error spawning ant for colony ${colony.id}:`, error)
    }
  }

  async getColonyStats(colonyId: string) {
    const { data: colony } = await this.supabase
      .from('colonies')
      .select('*')
      .eq('id', colonyId)
      .single()

    if (!colony) return null

    const { data: ants } = await this.supabase
      .from('ants')
      .select('*')
      .eq('colony_id', colonyId)
      .neq('state', 'dead')

    return {
      colony,
      population: ants?.length || 0,
      averageHealth: ants?.reduce((sum, ant) => sum + ant.health, 0) / (ants?.length || 1),
      resourceCount: Object.keys((colony.resources as any) || {}).length
    }
  }
} 