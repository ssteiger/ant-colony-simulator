import type { SupabaseClient } from '@supabase/supabase-js'
import type { Ant, FoodSource, AntType } from '../types/drizzle'
import type { Database } from '../types/supabase'
import { PheromoneManager } from './PheromoneManager'
import { SimulationCache } from './Cache'

// Extended ant type with joined relations from Supabase query
type AntWithRelations = Ant & {
  colonies: { simulation_id: string } | null
  ant_types: AntType | null
}

export class AntBehaviorManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null
  private pheromoneManager: PheromoneManager
  private cache: SimulationCache
  private antUpdates: Array<{
    id: string;
    position_x: number;
    position_y: number;
    angle: number;
    state?: string;
    target_x?: number;
    target_y?: number;
    target_id?: string;
    target_type?: string;
  }> = []

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
    this.pheromoneManager = new PheromoneManager(supabase)
    this.cache = new SimulationCache(supabase)
    console.log('🐜 AntBehaviorManager: Constructor initialized')
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    await this.pheromoneManager.initialize(simulationId)
    
    // Initialize cache
    await this.cache.initialize(simulationId)
    
    console.log('🐜 AntBehaviorManager: Initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('AntBehaviorManager not initialized')
    }

    console.log(`🐜 AntBehaviorManager: Processing tick ${tick}`)

    // Refresh cache periodically
    await this.cache.refreshCacheIfNeeded()

    // Get all living ants in the simulation with their ant type information
    const { data: ants } = await this.supabase
      .from('ants')
      .select(`
        *,
        colonies(simulation_id),
        ant_types(role, base_speed, base_strength, carrying_capacity, special_abilities)
      `)
      .neq('state', 'dead')

    if (!ants) {
      console.log('🐜 AntBehaviorManager: No ants found')
      return
    }

    // Filter ants for this simulation
    const simulationAnts = ants.filter(ant => 
      (ant.colonies as { simulation_id: string })?.simulation_id === this.simulationId
    )

    console.log(`🐜 AntBehaviorManager: Processing ${simulationAnts.length} ants at tick ${tick}`)

    // Process ants in parallel batches of 50 for better performance
    const BATCH_SIZE = 50
    const batches = []
    for (let i = 0; i < simulationAnts.length; i += BATCH_SIZE) {
      batches.push(simulationAnts.slice(i, i + BATCH_SIZE))
    }

    let processedCount = 0
    let deadCount = 0
    let errorCount = 0

    // Process batches in parallel
    for (const batch of batches) {
      const batchResults = await Promise.allSettled(
        batch.map(ant => this.processAntBehavior(ant as AntWithRelations))
      )

      // Count results
      for (const result of batchResults) {
        if (result.status === 'fulfilled') {
          if (result.value === 'dead') {
            deadCount++
          } else {
            processedCount++
          }
        } else {
          errorCount++
          console.error('🐜 AntBehaviorManager: Error processing ant:', result.reason)
        }
      }
    }

    // Execute batch updates at the end of the tick for better performance
    await this.batchUpdateAnts()

    console.log(`🐜 AntBehaviorManager: Tick ${tick} complete - Processed: ${processedCount}, Died: ${deadCount}, Errors: ${errorCount}`)
  }

  private async processAntBehavior(ant: AntWithRelations): Promise<string> {
    try {
      // Only log individual ant details every 100 ticks to reduce noise
      if (Math.random() < 0.01) { // Log 1% of ants randomly for debugging
        console.log(`🐜 Processing ant ${ant.id}: state=${ant.state}, energy=${ant.energy}, position=(${ant.position_x}, ${ant.position_y})`)
      }
      
      /*
      // Age the ant
      const newAge = ant.age_ticks + 1
      
      // Check if ant should die of old age
      if (newAge > 10000) { // Simplified lifespan
        console.log(`🐜 Ant ${ant.id} died of old age at ${newAge} ticks`)
        await this.killAnt(ant.id, 'old_age')
        return 'dead'
      }

      // Decrease energy over time
      const energyDecay = 1
      const newEnergy = Math.max(0, ant.energy - energyDecay)

      // If energy is too low, ant dies
      if (newEnergy <= 0) {
        console.log(`🐜 Ant ${ant.id} died of starvation (energy: ${newEnergy})`)
        await this.killAnt(ant.id, 'starvation')
        return 'dead'
      }
      */


      // Determine ant's next action based on current state
      const nextAction = await this.determineAntAction(ant)
      // Only log actions occasionally to reduce noise
      if (Math.random() < 0.01) {
        console.log(`🐜 Ant ${ant.id} determined action: ${nextAction}`)
      }
      
      // Execute the action
      await this.executeAntAction(ant, nextAction)

      // Update ant's basic properties
      await this.supabase
        .from('ants')
        .update({
          //age_ticks: newAge,
          //energy: newEnergy,
          last_updated: new Date().toISOString()
        })
        .eq('id', ant.id)

      return 'processed'
    } catch (error) {
      console.error(`🐜 Error processing ant ${ant.id} behavior:`, error)
      throw error
    }
  }

  private async determineAntAction(ant: AntWithRelations): Promise<string> {
    // Get ant role from type information
    const antType = ant.ant_types as AntType
    const role = antType.role

    // Role-based behavior modifications
    switch (ant.state) {
      case 'wandering': {
        // Look for food or follow pheromone trails
        const nearbyFood = await this.findNearbyFood(ant.position_x, ant.position_y, 50)
        if (nearbyFood) {
          // Workers and scouts prioritize food, soldiers are less interested
          if (role === 'soldier') {
            // Soldiers only collect food if it's very close and they're not patrolling
            const foodDistance = Math.sqrt(
              (nearbyFood.position_x - ant.position_x) ** 2 + 
              (nearbyFood.position_y - ant.position_y) ** 2
            )
            if (foodDistance < 20) {
              console.log(`🐜 Soldier ant ${ant.id} found very nearby food: ${nearbyFood.food_type}`)
              return 'seek_food'
            }
          } else {
            console.log(`🐜 ${role} ant ${ant.id} found nearby food: ${nearbyFood.food_type} at (${nearbyFood.position_x}, ${nearbyFood.position_y})`)
            return 'seek_food'
          }
        }

        // Check for pheromone trails to follow
        const searchRadius = role === 'scout' ? 40 : role === 'worker' ? 30 : 25 // Scouts have wider search
        const pheromoneInfluence = await this.pheromoneManager.getPheromoneInfluence(
          ant.position_x, 
          ant.position_y, 
          ant.colony_id,
          searchRadius
        )

        // Role-based pheromone following behavior
        const followThreshold = role === 'scout' ? 0.05 : role === 'worker' ? 0.1 : 0.15 // Scouts follow weaker trails
        if (pheromoneInfluence.strength > followThreshold) {
          console.log(`🐜 ${role} ant ${ant.id} detected pheromone trail (strength: ${pheromoneInfluence.strength})`)
          return 'follow_pheromone_trail'
        }

        // Scouts wander more aggressively (longer distances), soldiers patrol more systematically
        return role === 'scout' ? 'scout_explore' : role === 'soldier' ? 'patrol' : 'wander'
      }

      case 'seeking_food': {
        // Check if ant reached its food target
        if (ant.target_id) {
          const distance = await this.getDistanceToTarget(ant, ant.target_id)
          console.log(`🐜 ${role} ant ${ant.id} distance to food target: ${distance}`)
          if (distance < 5) {
            return 'collect_food'
          }
        }
        return 'move_to_food'
      }

      case 'carrying_food': {
        // Return to colony
        console.log(`🐜 ${role} ant ${ant.id} carrying food, returning to colony`)
        return 'return_to_colony'
      }

      default:
        console.log(`🐜 ${role} ant ${ant.id} in unknown state: ${ant.state}, defaulting to wander`)
        return 'wander'
    }
  }

  private async executeAntAction(ant: AntWithRelations, action: string): Promise<void> {
    // Only log actions occasionally to reduce noise
    if (Math.random() < 0.01) {
      console.log(`🐜 Executing action '${action}' for ant ${ant.id}`)
    }
    
    switch (action) {
      case 'wander':
        await this.moveAntRandomly(ant)
        break

      case 'scout_explore':
        await this.scoutExplore(ant)
        break

      case 'patrol':
        await this.soldierPatrol(ant)
        break

      case 'seek_food':
        await this.moveAntTowardsFood(ant)
        break

      case 'follow_pheromone_trail':
        await this.followPheromoneTrail(ant)
        break

      case 'move_to_food':
        await this.moveAntTowardsTarget(ant)
        break

      case 'collect_food':
        await this.collectFood(ant)
        break

      case 'return_to_colony':
        await this.moveAntTowardsColony(ant)
        break

      default:
        console.warn(`🐜 Unknown action '${action}' for ant ${ant.id}`)
    }
  }

  private async moveAntRandomly(ant: AntWithRelations): Promise<void> {
    // Use cached world bounds instead of querying database
    const worldBounds = this.cache.getWorldBounds()
    if (!worldBounds) {
      console.error(`🐜 World bounds not cached for simulation ${this.simulationId}`)
      return
    }

    // Convert current angle from degrees to radians
    let currentAngle = (ant.angle * Math.PI) / 180

    // 5% chance to randomly change direction by up to 15%
    if (Math.random() < 0.05) {
      const maxAngleChange = 0.15 * Math.PI // 15% of π radians (about 27 degrees)
      const angleChange = (Math.random() - 0.5) * 2 * maxAngleChange // Random change between -15% and +15%
      currentAngle += angleChange
      console.log(`🐜 Ant ${ant.id} randomly adjusted direction by ${(angleChange * 180 / Math.PI)}°`)
    }

    // Always move forward in the current direction
    const moveDistance = ant.current_speed

    // Calculate new position
    const newX = ant.position_x + Math.cos(currentAngle) * moveDistance
    const newY = ant.position_y + Math.sin(currentAngle) * moveDistance

    // Handle boundary conditions with reflection
    let boundedX = newX
    let boundedY = newY
    let finalAngle = currentAngle

    // Boundary collision detection with reflection
    if (newX < 0) {
      boundedX = Math.abs(newX) // Reflect off left boundary
      finalAngle = Math.PI - currentAngle
    } else if (newX > worldBounds.width) {
      boundedX = worldBounds.width - (newX - worldBounds.width)
      finalAngle = Math.PI - currentAngle
    }

    if (newY < 0) {
      boundedY = Math.abs(newY) // Reflect off top boundary
      finalAngle = -currentAngle
    } else if (newY > worldBounds.height) {
      boundedY = worldBounds.height - (newY - worldBounds.height)
      finalAngle = -currentAngle
    }

    // Ensure we stay within bounds after reflection
    boundedX = Math.max(0, Math.min(worldBounds.width, boundedX))
    boundedY = Math.max(0, Math.min(worldBounds.height, boundedY))

    // Normalize angle to [0, 2π] range
    finalAngle = ((finalAngle % (2 * Math.PI)) + (2 * Math.PI)) % (2 * Math.PI)

    // Round positions to integers
    const roundedX = Math.round(boundedX)
    const roundedY = Math.round(boundedY)

    // Convert angle from radians to degrees and round to integer
    const angleDegrees = Math.round((finalAngle * 180) / Math.PI)

    // Calculate actual distance moved for logging
    const actualDistance = Math.sqrt(
      (roundedX - ant.position_x) ** 2 + (roundedY - ant.position_y) ** 2
    )

    console.log(`🐜 Ant ${ant.id} moved forward ${actualDistance} units from (${ant.position_x}, ${ant.position_y}) to (${roundedX}, ${roundedY}) at ${angleDegrees}°`)

    await this.supabase
      .from('ants')
      .update({
        position_x: roundedX,
        position_y: roundedY,
        angle: angleDegrees,
        last_updated: new Date().toISOString()
      })
      .eq('id', ant.id)
  }

  private async moveAntTowardsFood(ant: AntWithRelations): Promise<void> {
    const nearbyFood = await this.findNearbyFood(ant.position_x, ant.position_y, 100)
    
    if (nearbyFood) {
      const angle = Math.atan2(
        nearbyFood.position_y - ant.position_y,
        nearbyFood.position_x - ant.position_x
      )
      
      const newX = ant.position_x + Math.cos(angle) * ant.current_speed
      const newY = ant.position_y + Math.sin(angle) * ant.current_speed

      // Round positions to integers
      const roundedX = Math.round(newX)
      const roundedY = Math.round(newY)

      // Convert angle from radians to degrees and round to integer
      const angleDegrees = Math.round((angle * 180) / Math.PI)

      console.log(`🐜 Moving ant ${ant.id} towards food ${nearbyFood.food_type} from (${ant.position_x}, ${ant.position_y}) to (${roundedX}, ${roundedY})`)

      await this.supabase
        .from('ants')
        .update({
          position_x: roundedX,
          position_y: roundedY,
          angle: angleDegrees,
          state: 'seeking_food',
          target_x: nearbyFood.position_x,
          target_y: nearbyFood.position_y,
          target_id: nearbyFood.id,
          target_type: 'food_source'
        })
        .eq('id', ant.id)
    } else {
      console.log(`🐜 Ant ${ant.id} lost sight of food, switching to wandering`)
      await this.supabase
        .from('ants')
        .update({ state: 'wandering' })
        .eq('id', ant.id)
    }
  }

  private async moveAntTowardsTarget(ant: AntWithRelations): Promise<void> {
    if (!ant.target_x || !ant.target_y) {
      console.warn(`🐜 Ant ${ant.id} has no target coordinates`)
      return
    }

    const angle = Math.atan2(ant.target_y - ant.position_y, ant.target_x - ant.position_x)
    const newX = ant.position_x + Math.cos(angle) * ant.current_speed
    const newY = ant.position_y + Math.sin(angle) * ant.current_speed

    // Round positions to integers
    const roundedX = Math.round(newX)
    const roundedY = Math.round(newY)

    // Convert angle from radians to degrees and round to integer
    const angleDegrees = Math.round((angle * 180) / Math.PI)

    console.log(`🐜 Moving ant ${ant.id} towards target from (${ant.position_x}, ${ant.position_y}) to (${roundedX}, ${roundedY})`)

    await this.supabase
      .from('ants')
      .update({
        position_x: roundedX,
        position_y: roundedY,
        angle: angleDegrees
      })
      .eq('id', ant.id)
  }

  private async moveAntTowardsColony(ant: AntWithRelations): Promise<void> {
    // Use cached colony position instead of querying database
    const colony = this.cache.getColony(ant.colony_id)

    if (!colony) {
      console.error(`🐜 Ant ${ant.id} cannot find colony ${ant.colony_id} in cache`)
      return
    }

    const angle = Math.atan2(colony.center_y - ant.position_y, colony.center_x - ant.position_x)
    const newX = ant.position_x + Math.cos(angle) * ant.current_speed
    const newY = ant.position_y + Math.sin(angle) * ant.current_speed

    // Round positions to integers
    const roundedX = Math.round(newX)
    const roundedY = Math.round(newY)

    // Check if ant reached colony
    const distance = Math.sqrt(
      (colony.center_x - roundedX) ** 2 + (colony.center_y - roundedY) ** 2
    )

    console.log(`🐜 Moving ant ${ant.id} towards colony '${colony.name}' - distance: ${distance}`)

    // Ants carrying food lay pheromone trails on their way back to colony
    // This is the classic behavior that creates reinforcement of successful paths
    if (ant.state === 'carrying_food') {
      try {
        await this.pheromoneManager.createFoodTrailAt(
          ant.position_x,
          ant.position_y,
          ant.colony_id,
          0.6 // Moderate strength - not as strong as discovery trails
        )
        console.log(`🐜 Ant ${ant.id} laid pheromone trail while carrying food home`)
      } catch (error) {
        // Don't log trail laying errors - they're common and expected
      }
    }

    if (distance < 15) {
      // Ant reached colony - deposit food and change state
      console.log(`🐜 Ant ${ant.id} reached colony '${colony.name}', depositing food`)
      await this.depositFood(ant)
    } else {
      // Convert angle from radians to degrees and round to integer
      const angleDegrees = Math.round((angle * 180) / Math.PI)

      await this.supabase
        .from('ants')
        .update({
          position_x: roundedX,
          position_y: roundedY,
          angle: angleDegrees
        })
        .eq('id', ant.id)
    }
  }

  private async collectFood(ant: AntWithRelations): Promise<void> {
    if (!ant.target_id) {
      console.warn(`🐜 Ant ${ant.id} trying to collect food but has no target_id`)
      return
    }

    // Get food source
    const { data: foodSource } = await this.supabase
      .from('food_sources')
      .select('*')
      .eq('id', ant.target_id)
      .single()

    if (!foodSource || foodSource.amount <= 0) {
      // Food is gone, switch to wandering
      console.log(`🐜 Ant ${ant.id} found no food at target location, switching to wandering`)
      await this.supabase
        .from('ants')
        .update({
          state: 'wandering',
          target_id: null,
          target_x: null,
          target_y: null,
          target_type: null
        })
        .eq('id', ant.id)
      return
    }

    // Collect food
    const collectionAmount = Math.min(1, foodSource.amount) // Ant can carry 1 unit
    const newFoodAmount = foodSource.amount - collectionAmount

    console.log(`🐜 Ant ${ant.id} collecting ${collectionAmount} unit(s) of ${foodSource.food_type}. Remaining: ${newFoodAmount}`)

    // Update food source
    await this.supabase
      .from('food_sources')
      .update({ amount: newFoodAmount })
      .eq('id', foodSource.id)

    // Update cached food source amount
    this.cache.updateFoodSource(foodSource.id, newFoodAmount)

    // Update ant to carry food
    const carriedResources: Record<string, number> = { [foodSource.food_type]: collectionAmount }
    
    await this.supabase
      .from('ants')
      .update({
        state: 'carrying_food',
        carried_resources: carriedResources,
        target_id: null,
        target_x: null,
        target_y: null,
        target_type: null
      })
      .eq('id', ant.id)

    // Create a strong pheromone marker at the food source to attract other ants
    // This represents the key discovery behavior - successful foragers mark good spots
    try {
      await this.pheromoneManager.createFoodTrailAt(
        ant.position_x,
        ant.position_y,
        ant.colony_id,
        1.0, // Strong initial strength
        foodSource.id
      )
      console.log(`🐜 Ant ${ant.id} marked food location with strong pheromone trail`)
    } catch (error) {
      console.warn('Failed to create pheromone trail at food source:', error)
    }

    if (newFoodAmount <= 0) {
      console.log(`🐜 Food source ${foodSource.id} (${foodSource.food_type}) has been depleted`)
    }
  }

  private async depositFood(ant: AntWithRelations): Promise<void> {
    const carriedResources = (ant.carried_resources as Record<string, number>) || {}
    
    if (Object.keys(carriedResources).length === 0) {
      // No food to deposit, start wandering
      console.log(`🐜 Ant ${ant.id} has no food to deposit, switching to wandering`)
      await this.supabase
        .from('ants')
        .update({ state: 'wandering' })
        .eq('id', ant.id)
      return
    }

    // Get colony current resources
    const { data: colony } = await this.supabase
      .from('colonies')
      .select('resources, name')
      .eq('id', ant.colony_id)
      .single()

    if (!colony) {
      console.error(`🐜 Ant ${ant.id} cannot find colony ${ant.colony_id} to deposit food`)
      return
    }

    const colonyResources = (colony.resources as Record<string, number>) || {}
    
    // Add carried resources to colony
    const updatedResources = { ...colonyResources }
    const depositedItems: string[] = []
    
    for (const [foodType, amount] of Object.entries(carriedResources)) {
      updatedResources[foodType] = (updatedResources[foodType] || 0) + amount
      depositedItems.push(`${amount} ${foodType}`)
    }

    console.log(`🐜 Ant ${ant.id} depositing ${depositedItems.join(', ')} to colony '${colony.name}'`)

    // Update colony resources
    await this.supabase
      .from('colonies')
      .update({ resources: updatedResources })
      .eq('id', ant.colony_id)

    // Clear ant's carried resources and start wandering
    await this.supabase
      .from('ants')
      .update({
        state: 'wandering',
        carried_resources: {},
        energy: Math.min(100, ant.energy + 10) // Restore some energy
      })
      .eq('id', ant.id)

    console.log(`🐜 Ant ${ant.id} successfully deposited food and gained energy (new energy: ${Math.min(100, ant.energy + 10)})`)
  }

  private async findNearbyFood(x: number, y: number, radius: number): Promise<FoodSource | null> {
    if (!this.simulationId) {
      console.error('🐜 Cannot find nearby food: simulationId is null')
      return null
    }

    // Use cached food sources instead of querying database
    const foodSources = this.cache.getFoodSources()
    if (foodSources.length === 0) {
      console.log(`🐜 No food sources in cache for simulation ${this.simulationId}`)
      return null
    }

    // Find closest food within radius from cache
    let closestFood: FoodSource | null = null
    let closestDistance = radius

    for (const food of foodSources) {
      // Only consider food sources with amount > 0
      if (food.amount <= 0) continue
      
      const distance = Math.sqrt(
        (food.position_x - x) ** 2 + (food.position_y - y) ** 2
      )
      
      if (distance < closestDistance) {
        closestDistance = distance
        closestFood = food
      }
    }

    if (closestFood) {
      console.log(`🐜 Found nearby food: ${closestFood.food_type} at distance ${closestDistance}`)
    }

    return closestFood
  }

  private async getDistanceToTarget(ant: AntWithRelations, targetId: string): Promise<number> {
    const { data: target } = await this.supabase
      .from('food_sources')
      .select('position_x, position_y')
      .eq('id', targetId)
      .single()

    if (!target) return Number.POSITIVE_INFINITY

    return Math.sqrt(
      (target.position_x - ant.position_x) ** 2 + 
      (target.position_y - ant.position_y) ** 2
    )
  }

  private async killAnt(antId: string, cause: string): Promise<void> {
    await this.supabase
      .from('ants')
      .update({
        state: 'dead',
        health: 0,
        energy: 0
      })
      .eq('id', antId)

    console.log(`💀 Ant ${antId} died from ${cause}`)
  }

  private async followPheromoneTrail(ant: AntWithRelations): Promise<void> {
    // Get pheromone influence at current position
    const pheromoneInfluence = await this.pheromoneManager.getPheromoneInfluence(
      ant.position_x, 
      ant.position_y, 
      ant.colony_id,
      25
    )

    if (pheromoneInfluence.strength === 0) {
      // No pheromone trail found, fall back to random movement
      console.log(`🐜 Ant ${ant.id} lost pheromone trail, switching to random movement`)
      await this.moveAntRandomly(ant)
      return
    }

    // Combine pheromone direction with some randomness (realistic behavior)
    // Stronger pheromone trails are followed more precisely
    const pheromoneWeight = Math.min(0.8, pheromoneInfluence.strength * 2) // Max 80% influence
    const randomWeight = 1 - pheromoneWeight

    // Generate some randomness for realistic movement
    const randomAngle = (Math.random() - 0.5) * Math.PI * 0.5 // ±45 degrees random variation
    const combinedDirection = pheromoneInfluence.direction + (randomAngle * randomWeight)

    // Calculate movement distance - ants move more confidently on strong trails
    const baseSpeed = ant.current_speed
    const speedMultiplier = 1 + (pheromoneInfluence.strength * 0.5) // Up to 50% speed boost on strong trails
    const moveDistance = baseSpeed * speedMultiplier

    // Calculate new position
    const newX = ant.position_x + Math.cos(combinedDirection) * moveDistance
    const newY = ant.position_y + Math.sin(combinedDirection) * moveDistance

    // Ensure position stays within world bounds using cached bounds
    const worldBounds = this.cache.getWorldBounds()
    if (!worldBounds) {
      console.error(`🐜 World bounds not cached for simulation ${this.simulationId}`)
      return
    }

    const boundedX = Math.max(0, Math.min(worldBounds.width, newX))
    const boundedY = Math.max(0, Math.min(worldBounds.height, newY))

    // Round positions to integers
    const roundedX = Math.round(boundedX)
    const roundedY = Math.round(boundedY)

    // Convert angle from radians to degrees
    const angleDegrees = Math.round((combinedDirection * 180) / Math.PI)

    console.log(`🐜 Ant ${ant.id} following pheromone trail (strength: ${pheromoneInfluence.strength.toFixed(3)}, weight: ${pheromoneWeight.toFixed(2)}) from (${ant.position_x}, ${ant.position_y}) to (${roundedX}, ${roundedY})`)

    // Check if we found food while following the trail
    const nearbyFood = await this.findNearbyFood(roundedX, roundedY, 15)
    let newState = ant.state
    let targetId = ant.target_id
    let targetX = ant.target_x
    let targetY = ant.target_y
    let targetType = ant.target_type

    if (nearbyFood) {
      console.log(`🐜 Ant ${ant.id} found food while following pheromone trail: ${nearbyFood.food_type}`)
      newState = 'seeking_food'
      targetId = nearbyFood.id
      targetX = nearbyFood.position_x
      targetY = nearbyFood.position_y
      targetType = 'food_source'
    }

    await this.supabase
      .from('ants')
      .update({
        position_x: roundedX,
        position_y: roundedY,
        angle: angleDegrees,
        state: newState,
        target_id: targetId,
        target_x: targetX,
        target_y: targetY,
        target_type: targetType,
        last_updated: new Date().toISOString()
      })
      .eq('id', ant.id)
  }

  private async scoutExplore(ant: AntWithRelations): Promise<void> {
    // Scouts move with longer, more aggressive exploration patterns
    // They're designed to find new food sources and create initial trails
    
    // Use cached world bounds instead of querying database
    const worldBounds = this.cache.getWorldBounds()
    if (!worldBounds) {
      console.error(`🐜 World bounds not cached for simulation ${this.simulationId}`)
      return
    }

    // Convert current angle from degrees to radians
    let currentAngle = (ant.angle * Math.PI) / 180

    // Get colony position for exploration bias
    const colony = this.cache.getColony(ant.colony_id)

    // 20% chance to randomly change direction by up to 30% (more aggressive than regular ants)
    if (Math.random() < 0.2) {
      const maxAngleChange = 0.3 * Math.PI // 30% of π radians (about 54 degrees)
      let angleChange = (Math.random() - 0.5) * 2 * maxAngleChange

      // Scouts prefer to explore away from colony - bias the angle change
      if (colony) {
        const distanceFromColony = Math.sqrt(
          (ant.position_x - colony.center_x) ** 2 + 
          (ant.position_y - colony.center_y) ** 2
        )
        
        // If close to colony, bias movement away from it
        if (distanceFromColony < 100) {
          const awayFromColony = Math.atan2(
            ant.position_y - colony.center_y,
            ant.position_x - colony.center_x
          )
          const angleDifference = currentAngle - awayFromColony
          // If ant is facing towards colony, encourage turning away
          if (Math.abs(angleDifference) > Math.PI / 2) {
            angleChange *= 1.5 // Amplify the turn
          }
        }
      }

      currentAngle += angleChange
      console.log(`🔍 Scout ant ${ant.id} randomly adjusted exploration direction by ${(angleChange * 180 / Math.PI).toFixed(1)}°`)
    }

    // Scouts move faster than regular ants
    const scoutSpeedBonus = 1.3 // 30% speed bonus
    const moveDistance = ant.current_speed * scoutSpeedBonus

    // Calculate new position
    const newX = ant.position_x + Math.cos(currentAngle) * moveDistance
    const newY = ant.position_y + Math.sin(currentAngle) * moveDistance

    // Handle boundaries with reflection using cached world bounds
    let boundedX = newX
    let boundedY = newY
    let finalAngle = currentAngle

    if (newX < 0) {
      boundedX = Math.abs(newX)
      finalAngle = Math.PI - currentAngle
    } else if (newX > worldBounds.width) {
      boundedX = worldBounds.width - (newX - worldBounds.width)
      finalAngle = Math.PI - currentAngle
    }

    if (newY < 0) {
      boundedY = Math.abs(newY)
      finalAngle = -currentAngle
    } else if (newY > worldBounds.height) {
      boundedY = worldBounds.height - (newY - worldBounds.height)
      finalAngle = -currentAngle
    }

    // Ensure we stay within bounds after reflection
    boundedX = Math.max(0, Math.min(worldBounds.width, boundedX))
    boundedY = Math.max(0, Math.min(worldBounds.height, boundedY))

    // Normalize angle to [0, 2π] range
    finalAngle = ((finalAngle % (2 * Math.PI)) + (2 * Math.PI)) % (2 * Math.PI)

    const roundedX = Math.round(boundedX)
    const roundedY = Math.round(boundedY)
    const angleDegrees = Math.round((finalAngle * 180) / Math.PI)

    // Calculate actual distance moved for logging
    const actualDistance = Math.sqrt(
      (roundedX - ant.position_x) ** 2 + (roundedY - ant.position_y) ** 2
    )

    console.log(`🔍 Scout ant ${ant.id} exploring: moved forward ${actualDistance} units to (${roundedX}, ${roundedY})`)

    // Scouts are more likely to detect food from further away
    const nearbyFood = await this.findNearbyFood(roundedX, roundedY, 80) // Larger detection radius
    if (nearbyFood) {
      console.log(`🔍 Scout ant ${ant.id} discovered food: ${nearbyFood.food_type}!`)
      // Create a strong discovery trail immediately
      try {
        await this.pheromoneManager.createFoodTrailAt(
          roundedX,
          roundedY,
          ant.colony_id,
          0.9, // Strong discovery trail
          nearbyFood.id
        )
      } catch (error) {
        // Ignore trail creation errors
      }
    }

    await this.supabase
      .from('ants')
      .update({
        position_x: roundedX,
        position_y: roundedY,
        angle: angleDegrees,
        last_updated: new Date().toISOString()
      })
      .eq('id', ant.id)
  }

  private async soldierPatrol(ant: AntWithRelations): Promise<void> {
    // Soldiers patrol in a more systematic pattern around the colony
    // They move in expanding circles or follow defensive patterns
    
    // Use cached colony position instead of querying database
    const colony = this.cache.getColony(ant.colony_id)

    if (!colony) {
      console.error(`🐜 Soldier ant ${ant.id} cannot find colony ${ant.colony_id} in cache`)
      await this.moveAntRandomly(ant)
      return
    }

    // Use cached world bounds instead of querying database
    const worldBounds = this.cache.getWorldBounds()
    if (!worldBounds) {
      console.error(`🐜 World bounds not cached for simulation ${this.simulationId}`)
      return
    }

    // Calculate distance from colony
    const distanceFromColony = Math.sqrt(
      (ant.position_x - colony.center_x) ** 2 + 
      (ant.position_y - colony.center_y) ** 2
    )

    // Soldiers patrol within a certain radius of the colony (defensive perimeter)
    const patrolRadius = 60
    const baseSpeed = ant.current_speed
    
    let direction: number
    const moveDistance = baseSpeed

    if (distanceFromColony > patrolRadius) {
      // Too far from colony, move back towards it
      direction = Math.atan2(
        colony.center_y - ant.position_y,
        colony.center_x - ant.position_x
      )
      console.log(`⚔️ Soldier ant ${ant.id} returning to patrol perimeter (distance: ${distanceFromColony.toFixed(1)})`)
    } else {
      // Within patrol area, move in circular pattern around colony
      const angleToColony = Math.atan2(
        ant.position_y - colony.center_y,
        ant.position_x - colony.center_x
      )
      
      // Add some angular movement to create circular patrol
      const patrolDirection = angleToColony + (Math.PI / 3) // 60 degrees ahead
      direction = patrolDirection + (Math.random() - 0.5) * 0.5 // Small random variation
      
      console.log(`⚔️ Soldier ant ${ant.id} patrolling around colony '${colony.name}'`)
    }

    // Calculate new position
    const newX = ant.position_x + Math.cos(direction) * moveDistance
    const newY = ant.position_y + Math.sin(direction) * moveDistance

    // Ensure position stays within world bounds using cached bounds
    const boundedX = Math.max(0, Math.min(worldBounds.width, newX))
    const boundedY = Math.max(0, Math.min(worldBounds.height, newY))

    const roundedX = Math.round(boundedX)
    const roundedY = Math.round(boundedY)
    const angleDegrees = Math.round((direction * 180) / Math.PI)

    await this.supabase
      .from('ants')
      .update({
        position_x: roundedX,
        position_y: roundedY,
        angle: angleDegrees,
        last_updated: new Date().toISOString()
      })
      .eq('id', ant.id)
  }

  private async batchUpdateAnts(): Promise<void> {
    if (this.antUpdates.length === 0) return

    try {
      // Execute batch updates in parallel
      const updatePromises = this.antUpdates.map(update => 
        this.supabase
          .from('ants')
          .update({
            position_x: update.position_x,
            position_y: update.position_y,
            angle: update.angle,
            ...(update.state && { state: update.state }),
            ...(update.target_x !== undefined && { target_x: update.target_x }),
            ...(update.target_y !== undefined && { target_y: update.target_y }),
            ...(update.target_id && { target_id: update.target_id }),
            ...(update.target_type && { target_type: update.target_type }),
            last_updated: new Date().toISOString()
          })
          .eq('id', update.id)
      )

      await Promise.all(updatePromises)
      
      if (this.antUpdates.length > 50) {
        console.log(`🐜 AntBehaviorManager: Bulk updated ${this.antUpdates.length} ants`)
      }
    } catch (error) {
      console.error('🐜 AntBehaviorManager: Failed to bulk update ants:', error)
    }

    // Clear the updates array
    this.antUpdates = []
  }

  private queueAntUpdate(update: {
    id: string;
    position_x: number;
    position_y: number;
    angle: number;
    state?: string;
    target_x?: number;
    target_y?: number;
    target_id?: string;
    target_type?: string;
  }): void {
    this.antUpdates.push(update)
  }
}