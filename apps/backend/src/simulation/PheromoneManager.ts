import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database } from '../types/supabase'
import type { Ant, PheromoneTrail } from '../types/drizzle'

export class PheromoneManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
    console.log('🔬 PheromoneManager: Constructor initialized')
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    console.log('🔬 PheromoneManager: Initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('PheromoneManager not initialized')
    }

    console.log(`🔬 PheromoneManager: Processing tick ${tick}`)

    // Decay all pheromone trails
    const decayedCount = await this.decayPheromoneTrails()

    // Remove expired or very weak trails
    const cleanedCount = await this.cleanupWeakTrails()

    // Create new pheromone trails from ants carrying food
    if (tick % 2 === 0) { // Create trails every 2 ticks
      console.log('🔬 PheromoneManager: Creating new pheromone trails...')
      const newTrailsCount = await this.createPheromoneTrails()
      console.log(`🔬 PheromoneManager: Created ${newTrailsCount} new pheromone trails`)
    }

    console.log(`🔬 PheromoneManager: Tick ${tick} complete - Decayed: ${decayedCount}, Cleaned: ${cleanedCount}`)
  }

  private async decayPheromoneTrails(): Promise<number> {
    // Get all active pheromone trails
    const { data: trails } = await this.supabase
      .from('pheromone_trails')
      .select('*')
      .gt('strength', 0)

    if (!trails || trails.length === 0) {
      console.log('🔬 PheromoneManager: No active trails to decay')
      return 0
    }

    console.log(`🔬 PheromoneManager: Decaying ${trails.length} pheromone trails`)
    let decayedCount = 0

    // Update each trail's strength
    for (const trail of trails) {
      const newStrength = Math.max(0, trail.strength - trail.decay_rate)
      
      if (newStrength !== trail.strength) {
        await this.supabase
          .from('pheromone_trails')
          .update({ strength: newStrength })
          .eq('id', trail.id)
        
        decayedCount++
      }
    }

    console.log(`🔬 PheromoneManager: ✅ Decayed ${decayedCount} trails`)
    return decayedCount
  }

  private async cleanupWeakTrails(): Promise<number> {
    console.log('🔬 PheromoneManager: Cleaning up weak and expired trails...')
    
    // Count trails before cleanup
    const { data: beforeCount } = await this.supabase
      .from('pheromone_trails')
      .select('id', { count: 'exact' })

    // Remove trails with very low strength
    const { count: weakCount } = await this.supabase
      .from('pheromone_trails')
      .delete({ count: 'exact' })
      .lt('strength', 0.01)

    // Remove expired trails
    const now = new Date().toISOString()
    const { count: expiredCount } = await this.supabase
      .from('pheromone_trails')
      .delete({ count: 'exact' })
      .lt('expires_at', now)

    const totalCleaned = (weakCount || 0) + (expiredCount || 0)
    console.log(`🔬 PheromoneManager: ✅ Cleaned up ${totalCleaned} trails (${weakCount || 0} weak, ${expiredCount || 0} expired)`)
    
    return totalCleaned
  }

  private async createPheromoneTrails(): Promise<number> {
    // Get all ants that are carrying food (they should leave food trails)
    const { data: ants } = await this.supabase
      .from('ants')
      .select(`
        *,
        colonies(simulation_id)
      `)
      .eq('state', 'carrying_food')

    if (!ants || ants.length === 0) {
      console.log('🔬 PheromoneManager: No ants carrying food')
      return 0
    }

    // Filter for ants in this simulation
    const simulationAnts = ants.filter(ant => 
      (ant.colonies as { simulation_id: string })?.simulation_id === this.simulationId
    )

    console.log(`🔬 PheromoneManager: Found ${simulationAnts.length} ants carrying food`)
    let newTrailsCreated = 0

    for (const ant of simulationAnts) {
      try {
        const created = await this.createFoodTrail(ant)
        if (created) newTrailsCreated++
      } catch (error) {
        console.warn(`🔬 PheromoneManager: Failed to create trail for ant ${ant.id}:`, error)
      }
    }

    return newTrailsCreated
  }

  private async createFoodTrail(ant: Ant): Promise<boolean> {
    try {
      // Check if there's already a recent trail at this position
      const { data: existingTrail } = await this.supabase
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
      if (existingTrail) {
        const newStrength = Math.min(1.0, existingTrail.strength + 0.1)
        await this.supabase
          .from('pheromone_trails')
          .update({ strength: newStrength })
          .eq('id', existingTrail.id)
        
        console.log(`🔬 Strengthened existing trail (${existingTrail.strength.toFixed(2)} → ${newStrength.toFixed(2)}) near ant ${ant.id}`)
        return false
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

      console.log(`🔬 Created new food trail at (${ant.position_x.toFixed(1)}, ${ant.position_y.toFixed(1)}) for ant ${ant.id}`)
      return true
    } catch (error) {
      // Ignore errors from trail creation (likely duplicate position)
      // This is expected when multiple ants are in the same area
      return false
    }
  }

  async getPheromoneInfluence(x: number, y: number, colonyId: string, radius = 20): Promise<{ direction: number; strength: number }> {
    console.log(`🔬 PheromoneManager: Getting pheromone influence at (${x.toFixed(1)}, ${y.toFixed(1)}) for colony ${colonyId}`)
    
    // Get all pheromone trails within radius
    const { data: trails } = await this.supabase
      .from('pheromone_trails')
      .select('*')
      .eq('colony_id', colonyId)
      .gt('strength', 0.1)

    if (!trails || trails.length === 0) {
      console.log('🔬 PheromoneManager: No trails found for influence calculation')
      return { direction: 0, strength: 0 }
    }

    let totalInfluenceX = 0
    let totalInfluenceY = 0
    let totalStrength = 0
    let influentialTrails = 0

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
        influentialTrails++
      }
    }

    if (totalStrength === 0) {
      console.log('🔬 PheromoneManager: No influential trails found')
      return { direction: 0, strength: 0 }
    }

    // Calculate final direction
    const direction = Math.atan2(totalInfluenceY, totalInfluenceX)
    
    const result = {
      direction,
      strength: totalStrength
    }

    console.log(`🔬 PheromoneManager: Pheromone influence calculated - ${influentialTrails} trails, direction: ${direction.toFixed(2)}, strength: ${totalStrength.toFixed(3)}`)
    return result
  }

  async createDangerTrail(x: number, y: number, colonyId: string): Promise<void> {
    console.log(`🔬 PheromoneManager: Creating danger trail at (${x.toFixed(1)}, ${y.toFixed(1)}) for colony ${colonyId}`)
    
    const expiresAt = new Date()
    expiresAt.setMinutes(expiresAt.getMinutes() + 10) // Danger trails last 10 minutes

    try {
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

      console.log(`🔬 PheromoneManager: ✅ Created danger trail at (${x.toFixed(1)}, ${y.toFixed(1)})`)
    } catch (error) {
      console.error('🔬 PheromoneManager: ❌ Failed to create danger trail:', error)
    }
  }
} 