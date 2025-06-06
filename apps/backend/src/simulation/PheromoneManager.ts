import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database } from '../types/ant-colony'

export class PheromoneManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    console.log('PheromoneManager initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('PheromoneManager not initialized')
    }

    // Decay all pheromone trails
    await this.decayPheromoneTrails()

    // Remove expired or very weak trails
    await this.cleanupWeakTrails()

    // Create new pheromone trails from ants carrying food
    if (tick % 5 === 0) { // Create trails every 5 ticks
      await this.createPheromoneTrails()
    }
  }

  private async decayPheromoneTrails(): Promise<void> {
    // Get all active pheromone trails
    const { data: trails } = await this.supabase
      .from('pheromone_trails')
      .select('*')
      .gt('strength', 0)

    if (!trails) return

    // Update each trail's strength
    for (const trail of trails) {
      const newStrength = Math.max(0, trail.strength - trail.decay_rate)
      
      await this.supabase
        .from('pheromone_trails')
        .update({ strength: newStrength })
        .eq('id', trail.id)
    }
  }

  private async cleanupWeakTrails(): Promise<void> {
    // Remove trails with very low strength
    await this.supabase
      .from('pheromone_trails')
      .delete()
      .lt('strength', 0.01)

    // Remove expired trails
    const now = new Date().toISOString()
    await this.supabase
      .from('pheromone_trails')
      .delete()
      .lt('expires_at', now)
  }

  private async createPheromoneTrails(): Promise<void> {
    // Get all ants that are carrying food (they should leave food trails)
    const { data: ants } = await this.supabase
      .from('ants')
      .select(`
        *,
        colonies(simulation_id)
      `)
      .eq('state', 'carrying_food')

    if (!ants) return

    // Filter for ants in this simulation
    const simulationAnts = ants.filter(ant => 
      (ant.colonies as any)?.simulation_id === this.simulationId
    )

    for (const ant of simulationAnts) {
      await this.createFoodTrail(ant)
    }
  }

  private async createFoodTrail(ant: any): Promise<void> {
    try {
      // Check if there's already a recent trail at this position
      const existingTrail = await this.supabase
        .from('pheromone_trails')
        .select('*')
        .eq('colony_id', ant.colony_id)
        .eq('trail_type', 'food')
        .gte('position_x', ant.position_x - 5)
        .lte('position_x', ant.position_x + 5)
        .gte('position_y', ant.position_y - 5)
        .lte('position_y', ant.position_y + 5)
        .gt('strength', 0.5)
        .limit(1)
        .single()

      // If there's already a strong trail nearby, strengthen it instead of creating new one
      if (existingTrail.data) {
        const newStrength = Math.min(1.0, existingTrail.data.strength + 0.1)
        await this.supabase
          .from('pheromone_trails')
          .update({ strength: newStrength })
          .eq('id', existingTrail.data.id)
        return
      }

      // Create new pheromone trail
      const expiresAt = new Date()
      expiresAt.setMinutes(expiresAt.getMinutes() + 30) // Trail lasts 30 minutes

      await this.supabase
        .from('pheromone_trails')
        .insert({
          colony_id: ant.colony_id,
          trail_type: 'food',
          position_x: ant.position_x,
          position_y: ant.position_y,
          strength: 0.8,
          decay_rate: 0.002,
          expires_at: expiresAt.toISOString(),
          source_ant_id: ant.id
        })
    } catch (error) {
      // Ignore errors from trail creation (likely duplicate position)
      // This is expected when multiple ants are in the same area
    }
  }

  async getPheromoneInfluence(x: number, y: number, colonyId: string, radius: number = 20) {
    // Get all pheromone trails within radius
    const { data: trails } = await this.supabase
      .from('pheromone_trails')
      .select('*')
      .eq('colony_id', colonyId)
      .gt('strength', 0.1)

    if (!trails) return { direction: 0, strength: 0 }

    let totalInfluenceX = 0
    let totalInfluenceY = 0
    let totalStrength = 0

    for (const trail of trails) {
      const distance = Math.sqrt(
        (trail.position_x - x) ** 2 + (trail.position_y - y) ** 2
      )

      if (distance <= radius && distance > 0) {
        // Calculate influence based on distance and trail strength
        const influence = trail.strength / (1 + distance * 0.1)
        
        // Direction vector from current position to trail
        const dirX = (trail.position_x - x) / distance
        const dirY = (trail.position_y - y) / distance

        totalInfluenceX += dirX * influence
        totalInfluenceY += dirY * influence
        totalStrength += influence
      }
    }

    if (totalStrength === 0) {
      return { direction: 0, strength: 0 }
    }

    // Calculate final direction
    const direction = Math.atan2(totalInfluenceY, totalInfluenceX)
    
    return {
      direction,
      strength: totalStrength
    }
  }

  async createDangerTrail(x: number, y: number, colonyId: string): Promise<void> {
    const expiresAt = new Date()
    expiresAt.setMinutes(expiresAt.getMinutes() + 10) // Danger trails last 10 minutes

    await this.supabase
      .from('pheromone_trails')
      .insert({
        colony_id: colonyId,
        trail_type: 'danger',
        position_x: x,
        position_y: y,
        strength: 1.0,
        decay_rate: 0.01, // Decay faster than food trails
        expires_at: expiresAt.toISOString()
      })
  }
} 