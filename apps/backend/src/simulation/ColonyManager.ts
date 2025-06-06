import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database, Colony } from '../types/ant-colony'

export class ColonyManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
    console.log('🏰 ColonyManager: Constructor initialized')
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    console.log('🏰 ColonyManager: Initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('ColonyManager not initialized')
    }

    console.log(`🏰 ColonyManager: Processing tick ${tick}`)

    // Get all active colonies for this simulation
    const { data: colonies } = await this.supabase
      .from('colonies')
      .select('*')
      .eq('simulation_id', this.simulationId)
      .eq('is_active', true)

    if (!colonies || colonies.length === 0) {
      console.log(`🏰 ColonyManager: No active colonies found for simulationId ${this.simulationId}`)
      console.log('🏰 ColonyManager: Creating two initial colonies...')
      
      await this.createInitialColonies()
      
      // Re-fetch colonies after creating them
      const { data: newColonies } = await this.supabase
        .from('colonies')
        .select('*')
        .eq('simulation_id', this.simulationId)
        .eq('is_active', true)
      
      if (!newColonies || newColonies.length === 0) {
        console.error('🏰 ColonyManager: Failed to create or retrieve initial colonies')
        return
      }
      
      console.log(`🏰 ColonyManager: Successfully created ${newColonies.length} initial colonies`)
      // Process the newly created colonies
      for (const colony of newColonies) {
        try {
          await this.processColony(colony, tick)
        } catch (error) {
          console.error(`🏰 ColonyManager: Error processing new colony ${colony.id} (${colony.name}):`, error)
        }
      }
      return
    }

    console.log(`🏰 ColonyManager: Processing ${colonies.length} colonies`)

    // Process each colony
    let processedCount = 0
    let errorCount = 0

    for (const colony of colonies) {
      try {
        await this.processColony(colony, tick)
        processedCount++
      } catch (error) {
        errorCount++
        console.error(`🏰 ColonyManager: Error processing colony ${colony.id} (${colony.name}):`, error)
      }
    }

    console.log(`🏰 ColonyManager: Tick ${tick} complete - Processed: ${processedCount}, Errors: ${errorCount}`)
  }

  private async processColony(colony: Colony, tick: number): Promise<void> {
    try {
      console.log(`🏰 Processing colony: ${colony.name} (ID: ${colony.id})`)
      
      // Update colony population count based on living ants
      const { data: ants } = await this.supabase
        .from('ants')
        .select('id, state, energy')
        .eq('colony_id', colony.id)
        .neq('state', 'dead')

      const currentPopulation = ants?.length || 0
      const avgEnergy = ants?.reduce((sum, ant) => sum + ant.energy, 0) / (currentPopulation || 1)

      console.log(`🏰 Colony ${colony.name}: Population ${currentPopulation}, Avg Energy: ${avgEnergy.toFixed(1)}`)

      if (currentPopulation !== colony.population) {
        console.log(`🏰 Colony ${colony.name}: Updating population from ${colony.population} to ${currentPopulation}`)
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
      console.error(`🏰 Error processing colony ${colony.id} (${colony.name}):`, error)
      throw error
    }
  }

  private async processColonyResources(colony: Colony, tick: number): Promise<void> {
    const resources = (colony.resources as Record<string, number>) || {}
    
    console.log(`🏰 Colony ${colony.name} resources:`, resources)
    
    // Calculate resource consumption based on population
    const dailyConsumption = {
      seeds: colony.population * 0.1,
      sugar: colony.population * 0.05,
      protein: colony.population * 0.03
    }

    // Consume resources every 100 ticks (simulating daily consumption)
    if (tick % 100 === 0) {
      console.log(`🏰 Colony ${colony.name}: Daily resource consumption - Seeds: ${dailyConsumption.seeds.toFixed(2)}, Sugar: ${dailyConsumption.sugar.toFixed(2)}, Protein: ${dailyConsumption.protein.toFixed(2)}`)
      
      const updatedResources = {
        seeds: Math.max(0, (resources.seeds || 0) - dailyConsumption.seeds),
        sugar: Math.max(0, (resources.sugar || 0) - dailyConsumption.sugar),
        protein: Math.max(0, (resources.protein || 0) - dailyConsumption.protein)
      }

      const consumedSeeds = (resources.seeds || 0) - updatedResources.seeds
      const consumedSugar = (resources.sugar || 0) - updatedResources.sugar
      const consumedProtein = (resources.protein || 0) - updatedResources.protein

      console.log(`🏰 Colony ${colony.name}: Consumed - Seeds: ${consumedSeeds.toFixed(2)}, Sugar: ${consumedSugar.toFixed(2)}, Protein: ${consumedProtein.toFixed(2)}`)
      console.log(`🏰 Colony ${colony.name}: Remaining - Seeds: ${updatedResources.seeds.toFixed(2)}, Sugar: ${updatedResources.sugar.toFixed(2)}, Protein: ${updatedResources.protein.toFixed(2)}`)

      await this.supabase
        .from('colonies')
        .update({ resources: updatedResources })
        .eq('id', colony.id)

      // Warn if resources are getting low
      const totalResources = updatedResources.seeds + updatedResources.sugar + updatedResources.protein
      if (totalResources < 20) {
        console.warn(`🏰 Colony ${colony.name}: ⚠️ Resources running low! Total: ${totalResources.toFixed(2)}`)
      }
    }
  }

  private async handleColonyGrowth(colony: Colony, tick: number): Promise<void> {
    const resources = (colony.resources as Record<string, number>) || {}
    const totalResources = (resources.seeds || 0) + (resources.sugar || 0) + (resources.protein || 0)

    // Spawn new ants if colony has enough resources and isn't too crowded
    const shouldSpawnAnt = (
      totalResources > 50 && // Minimum resources needed
      colony.population < 100 && // Population cap
      tick % 200 === 0 // Spawn every 200 ticks
    )

    if (shouldSpawnAnt) {
      console.log(`🏰 Colony ${colony.name}: Conditions met for spawning new ant (Resources: ${totalResources.toFixed(2)}, Population: ${colony.population}/100)`)
      await this.spawnAnt(colony)
    } else if (tick % 200 === 0) {
      console.log(`🏰 Colony ${colony.name}: Cannot spawn ant - Resources: ${totalResources.toFixed(2)}, Population: ${colony.population}/100`)
    }
  }

  private async spawnAnt(colony: Colony): Promise<void> {
    try {
      console.log(`🏰 Colony ${colony.name}: Attempting to spawn new ant...`)
      
      // Get a random worker ant type
      const { data: antType } = await this.supabase
        .from('ant_types')
        .select('*')
        .eq('role', 'worker')
        .limit(1)
        .single()

      if (!antType) {
        console.warn('🏰 No worker ant type found for spawning')
        return
      }

      console.log(`🏰 Colony ${colony.name}: Using ant type: ${antType.name} (${antType.role})`)

      // Spawn ant near colony center with some randomness
      const spawnRadius = 10
      const randomAngle = Math.random() * 2 * Math.PI
      const randomDistance = Math.random() * spawnRadius

      const spawnX = colony.center_x + Math.cos(randomAngle) * randomDistance
      const spawnY = colony.center_y + Math.sin(randomAngle) * randomDistance

      console.log(`🏰 Colony ${colony.name}: Spawning ant at (${spawnX.toFixed(1)}, ${spawnY.toFixed(1)})`)

      const { data: newAnt } = await this.supabase
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
        .select('id')
        .single()

      if (newAnt) {
        console.log(`🏰 Colony ${colony.name}: ✅ Successfully spawned new ant (ID: ${newAnt.id})`)
      } else {
        console.warn(`🏰 Colony ${colony.name}: ⚠️ Failed to spawn ant - no data returned`)
      }
    } catch (error) {
      console.error(`🏰 Colony ${colony.name}: ❌ Error spawning ant:`, error)
    }
  }

  async getColonyStats(colonyId: string) {
    console.log(`🏰 ColonyManager: Getting stats for colony ${colonyId}`)
    
    const { data: colony } = await this.supabase
      .from('colonies')
      .select('*')
      .eq('id', colonyId)
      .single()

    if (!colony) {
      console.warn(`🏰 ColonyManager: Colony ${colonyId} not found`)
      return null
    }

    const { data: ants } = await this.supabase
      .from('ants')
      .select('*')
      .eq('colony_id', colonyId)
      .neq('state', 'dead')

    const stats = {
      colony,
      population: ants?.length || 0,
      averageHealth: ants?.reduce((sum, ant) => sum + ant.health, 0) / (ants?.length || 1),
      resourceCount: Object.keys((colony.resources as Record<string, number>) || {}).length
    }

    console.log(`🏰 ColonyManager: Colony ${colony.name} stats:`, stats)
    return stats
  }

  private async createInitialColonies(): Promise<void> {
    if (!this.simulationId) {
      throw new Error('ColonyManager not initialized')
    }

    try {
      console.log('🏰 ColonyManager: Fetching simulation world dimensions...')
      
      // Get simulation world dimensions
      const { data: simulation } = await this.supabase
        .from('simulations')
        .select('world_width, world_height')
        .eq('id', this.simulationId)
        .single()

      if (!simulation) {
        throw new Error('Simulation not found')
      }

      const worldWidth = simulation.world_width
      const worldHeight = simulation.world_height
      
      console.log(`🏰 ColonyManager: World dimensions: ${worldWidth}x${worldHeight}`)

      // Position colonies with adequate spacing
      const spacing = Math.min(worldWidth, worldHeight) * 0.4 // 40% of smaller dimension
      const centerX = worldWidth / 2
      const centerY = worldHeight / 2

      const colonies = [
        {
          name: 'Red Colony',
          center_x: centerX - spacing / 2,
          center_y: centerY,
          color_hue: 0, // Red
          resources: { seeds: 100, sugar: 50, protein: 25 }
        },
        {
          name: 'Blue Colony',
          center_x: centerX + spacing / 2,
          center_y: centerY,
          color_hue: 240, // Blue
          resources: { seeds: 80, sugar: 60, protein: 30 }
        }
      ]

      console.log('🏰 ColonyManager: Creating colonies with positions:')
      
      for (const colonyData of colonies) {
        console.log(`🏰 ColonyManager: - ${colonyData.name} at (${colonyData.center_x.toFixed(1)}, ${colonyData.center_y.toFixed(1)})`)
        
        const { data: newColony, error } = await this.supabase
          .from('colonies')
          .insert({
            simulation_id: this.simulationId,
            name: colonyData.name,
            center_x: colonyData.center_x,
            center_y: colonyData.center_y,
            radius: 30.0,
            population: 0,
            color_hue: colonyData.color_hue,
            resources: colonyData.resources,
            nest_level: 1,
            territory_radius: 100.0,
            aggression_level: 0.5,
            is_active: true
          })
          .select('id, name')
          .single()

        if (error) {
          console.error(`🏰 ColonyManager: Error creating ${colonyData.name}:`, error)
          throw error
        }

        if (newColony) {
          console.log(`🏰 ColonyManager: ✅ Created ${colonyData.name} (ID: ${newColony.id})`)
        }
      }

      console.log('🏰 ColonyManager: ✅ All initial colonies created successfully')
    } catch (error) {
      console.error('🏰 ColonyManager: ❌ Failed to create initial colonies:', error)
      throw error
    }
  }
} 