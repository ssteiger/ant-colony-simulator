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
    console.log('🔬 PheromoneManager: Decaying pheromone trails with bulk update')
    
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
    
    // Prepare bulk updates - convert decay_rate back to decimal for calculation
    const updates = trails
      .map(trail => ({
        ...trail,
        strength: Math.max(0, trail.strength - (trail.decay_rate / 10)) // Convert decay_rate back to decimal scale
      }))
      .filter(trail => trail.strength > 0) // Only update trails that still have strength

    // Execute bulk update
    if (updates.length > 0) {
      const { error } = await this.supabase
        .from('pheromone_trails')
        .upsert(updates)

      if (error) {
        console.error('🔬 PheromoneManager: Error in bulk decay:', error)
        return 0
      }
    }

    console.log(`🔬 PheromoneManager: ✅ Bulk updated ${updates.length} trails`)
    return updates.length
  }

  private async cleanupWeakTrails(): Promise<number> {
    console.log('🔬 PheromoneManager: Cleaning up weak and expired trails...')
    
    // Remove trails with very low strength (less than 1 in integer scale, which is 0.01 in decimal)
    const { count: weakCount } = await this.supabase
      .from('pheromone_trails')
      .delete({ count: 'exact' })
      .lt('strength', 1)

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
        .gt('strength', 50) // Adjusted for integer scale (0.5 * 100)
        .limit(1)
        .single()

      // If there's already a strong trail nearby, strengthen it instead of creating new one
      if (existingTrail) {
        const newStrength = Math.min(100, existingTrail.strength + 10) // Adjusted for integer scale
        await this.supabase
          .from('pheromone_trails')
          .update({ strength: newStrength })
          .eq('id', existingTrail.id)
        
        console.log(`🔬 Strengthened existing trail (${existingTrail.strength} → ${newStrength}) near ant ${ant.id}`)
        return false
      }

      // Create new pheromone trail with shorter lifespan
      const expiresAt = new Date()
      expiresAt.setMinutes(expiresAt.getMinutes() + 5) // Reduced from 30 to 5 minutes

      const { error } = await this.supabase
        .from('pheromone_trails')
        .insert({
          colony_id: ant.colony_id,
          trail_type: 'food',
          position_x: Math.round(ant.position_x),
          position_y: Math.round(ant.position_y),
          strength: 80, // Convert 0.8 to integer scale (0.8 * 100)
          decay_rate: 5, // Increased from 2 to 5 for faster decay
          expires_at: expiresAt.toISOString(),
          source_ant_id: ant.id
        })

      if (error) {
        console.error('🔬 PheromoneManager: ❌ Insert error:', error)
        console.error('🔬 PheromoneManager: Ant data:', {
          colony_id: ant.colony_id,
          position_x: ant.position_x,
          position_y: ant.position_y,
          id: ant.id
        })
        return false
      }

      console.log(`🔬 Created new food trail at (${ant.position_x.toFixed(1)}, ${ant.position_y.toFixed(1)}) for ant ${ant.id}`)
      return true
    } catch (error) {
      console.error('🔬 PheromoneManager: ❌ Exception in createFoodTrail:', error)
      console.error('🔬 PheromoneManager: Ant data:', ant)
      return false
    }
  }

  async getPheromoneInfluence(x: number, y: number, colonyId: string, radius = 20): Promise<{ direction: number; strength: number }> {
    console.log(`🔬 PheromoneManager: Getting pheromone influence at (${x.toFixed(1)}, ${y.toFixed(1)}) for colony ${colonyId}`)
    
    // Add spatial filtering to only get trails within radius + small buffer
    const searchRadius = radius + 5 // Small buffer for edge cases
    const { data: trails } = await this.supabase
      .from('pheromone_trails')
      .select('*')
      .eq('colony_id', colonyId)
      .gt('strength', 10) // Adjusted for integer scale (0.1 * 100)
      .gte('position_x', x - searchRadius)
      .lte('position_x', x + searchRadius)
      .gte('position_y', y - searchRadius)
      .lte('position_y', y + searchRadius)

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

      // Hard cutoff at radius - no influence beyond this distance
      if (distance <= radius && distance > 0) {
        // Much stronger distance decay - exponential falloff
        const normalizedDistance = distance / radius // 0 to 1
        const distanceDecay = Math.exp(-normalizedDistance * 3) // Exponential decay
        const influence = (trail.strength / 100) * distanceDecay
        
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
    expiresAt.setMinutes(expiresAt.getMinutes() + 3) // Reduced from 10 to 3 minutes

    try {
      const { error } = await this.supabase
        .from('pheromone_trails')
        .insert({
          colony_id: colonyId,
          trail_type: 'danger',
          position_x: Math.round(x),
          position_y: Math.round(y),
          strength: 100, // Convert 1.0 to integer scale (1.0 * 100)
          decay_rate: 15, // Increased from 10 to 15 for faster decay
          expires_at: expiresAt.toISOString()
        })

      if (error) {
        console.error('🔬 PheromoneManager: ❌ Failed to create danger trail:', error)
        return
      }

      console.log(`🔬 PheromoneManager: ✅ Created danger trail at (${x.toFixed(1)}, ${y.toFixed(1)})`)
    } catch (error) {
      console.error('🔬 PheromoneManager: ❌ Failed to create danger trail:', error)
    }
  }

  async createFoodTrailAt(x: number, y: number, colonyId: string, strength = 0.8, targetFoodId?: string): Promise<void> {
    console.log(`🔬 PheromoneManager: Creating food trail at (${x.toFixed(1)}, ${y.toFixed(1)}) for colony ${colonyId} with strength ${strength.toFixed(2)}`)
    
    try {
      // Check if there's already a recent trail at this position
      const { data: existingTrail } = await this.supabase
        .from('pheromone_trails')
        .select('*')
        .eq('colony_id', colonyId)
        .eq('trail_type', 'food')
        .gte('position_x', x - 3)
        .lte('position_x', x + 3)
        .gte('position_y', y - 3)
        .lte('position_y', y + 3)
        .gt('strength', 30) // Adjusted for integer scale (0.3 * 100)
        .limit(1)
        .single()

      // If there's already a trail nearby, strengthen it instead
      if (existingTrail) {
        const newStrength = Math.min(100, existingTrail.strength + Math.round(strength * 50)) // Adjusted for integer scale
        const { error } = await this.supabase
          .from('pheromone_trails')
          .update({ strength: newStrength })
          .eq('id', existingTrail.id)
        
        if (error) {
          console.error('🔬 PheromoneManager: ❌ Failed to strengthen food trail:', error)
          return
        }
        
        console.log(`🔬 Strengthened existing food trail (${existingTrail.strength} → ${newStrength})`)
        return
      }

      // Create new pheromone trail
      const expiresAt = new Date()
      expiresAt.setMinutes(expiresAt.getMinutes() + 8) // Reduced from 45 to 8 minutes

      const { error } = await this.supabase
        .from('pheromone_trails')
        .insert({
          colony_id: colonyId,
          trail_type: 'food',
          position_x: Math.round(x),
          position_y: Math.round(y),
          strength: Math.min(100, Math.round(strength * 100)), // Convert to integer scale
          decay_rate: 3, // Increased from 1 to 3 for faster decay
          expires_at: expiresAt.toISOString(),
          target_food_id: targetFoodId
        })

      if (error) {
        console.error('🔬 PheromoneManager: ❌ Failed to create food trail:', error)
        return
      }

      console.log(`🔬 PheromoneManager: ✅ Created food discovery trail at (${x.toFixed(1)}, ${y.toFixed(1)}) with strength ${Math.round(strength * 100)}`)
    } catch (error) {
      console.error('🔬 PheromoneManager: ❌ Failed to create food trail:', error)
    }
  }
} 