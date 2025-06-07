import type { SupabaseClient } from '@supabase/supabase-js'
import type { Ant, FoodSource } from '../types/drizzle'
import type { Database } from '../types/supabase'

export class AntBehaviorManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
    console.log('🐜 AntBehaviorManager: Constructor initialized')
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    console.log('🐜 AntBehaviorManager: Initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('AntBehaviorManager not initialized')
    }

    console.log(`🐜 AntBehaviorManager: Processing tick ${tick}`)

    // Get all living ants in the simulation
    const { data: ants } = await this.supabase
      .from('ants')
      .select(`
        *,
        colonies(simulation_id)
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

    // Process each ant's behavior
    let processedCount = 0
    let deadCount = 0
    let errorCount = 0

    for (const ant of simulationAnts) {
      try {
        const result = await this.processAntBehavior(ant, tick)
        if (result === 'dead') {
          deadCount++
        } else {
          processedCount++
        }
      } catch (error) {
        errorCount++
        console.error(`🐜 AntBehaviorManager: Error processing ant ${ant.id}:`, error)
      }
    }

    console.log(`🐜 AntBehaviorManager: Tick ${tick} complete - Processed: ${processedCount}, Died: ${deadCount}, Errors: ${errorCount}`)
  }

  private async processAntBehavior(ant: Ant, tick: number): Promise<string> {
    try {
      console.log(`🐜 Processing ant ${ant.id}: state=${ant.state}, energy=${ant.energy}, position=(${ant.position_x}, ${ant.position_y})`)
      
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

      // Determine ant's next action based on current state
      const nextAction = await this.determineAntAction(ant)
      console.log(`🐜 Ant ${ant.id} determined action: ${nextAction}`)
      
      // Execute the action
      await this.executeAntAction(ant, nextAction)

      // Update ant's basic properties
      await this.supabase
        .from('ants')
        .update({
          age_ticks: newAge,
          energy: newEnergy,
          last_updated: new Date().toISOString()
        })
        .eq('id', ant.id)

      return 'processed'
    } catch (error) {
      console.error(`🐜 Error processing ant ${ant.id} behavior:`, error)
      throw error
    }
  }

  private async determineAntAction(ant: Ant): Promise<string> {
    // Simple state machine for ant behavior
    switch (ant.state) {
      case 'wandering': {
        // Look for food or follow pheromone trails
        const nearbyFood = await this.findNearbyFood(ant.position_x, ant.position_y, 50)
        if (nearbyFood) {
          console.log(`🐜 Ant ${ant.id} found nearby food: ${nearbyFood.food_type} at (${nearbyFood.position_x.toFixed(1)}, ${nearbyFood.position_y.toFixed(1)})`)
          return 'seek_food'
        }
        return 'wander'
      }

      case 'seeking_food': {
        // Check if ant reached its food target
        if (ant.target_id) {
          const distance = await this.getDistanceToTarget(ant, ant.target_id)
          console.log(`🐜 Ant ${ant.id} distance to food target: ${distance.toFixed(1)}`)
          if (distance < 5) {
            return 'collect_food'
          }
        }
        return 'move_to_food'
      }

      case 'carrying_food': {
        // Return to colony
        console.log(`🐜 Ant ${ant.id} carrying food, returning to colony`)
        return 'return_to_colony'
      }

      default:
        console.log(`🐜 Ant ${ant.id} in unknown state: ${ant.state}, defaulting to wander`)
        return 'wander'
    }
  }

  private async executeAntAction(ant: Ant, action: string): Promise<void> {
    console.log(`🐜 Executing action '${action}' for ant ${ant.id}`)
    
    switch (action) {
      case 'wander':
        await this.moveAntRandomly(ant)
        break

      case 'seek_food':
        await this.moveAntTowardsFood(ant)
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

  private async moveAntRandomly(ant: Ant): Promise<void> {
    // Fetch world bounds from simulation
    const { data: simulation } = await this.supabase
      .from('simulations')
      .select('world_width, world_height')
      .eq('id', this.simulationId)
      .single()

    if (!simulation) {
      console.error(`🐜 Cannot find simulation ${this.simulationId} for world bounds`)
      return
    }

    // Generate random movement
    const moveDistance = ant.current_speed
    const randomAngle = Math.random() * 2 * Math.PI
    
    const newX = ant.position_x + Math.cos(randomAngle) * moveDistance
    const newY = ant.position_y + Math.sin(randomAngle) * moveDistance

    // Use dynamic world bounds from simulation and round to integers
    const boundedX = Math.round(Math.max(0, Math.min(simulation.world_width, newX)))
    const boundedY = Math.round(Math.max(0, Math.min(simulation.world_height, newY)))

    // Convert angle from radians to degrees and round to integer
    const angleDegrees = Math.round((randomAngle * 180) / Math.PI)

    console.log(`🐜 Moving ant ${ant.id} randomly from (${ant.position_x}, ${ant.position_y}) to (${boundedX}, ${boundedY}) within bounds (${simulation.world_width}x${simulation.world_height})`)

    await this.supabase
      .from('ants')
      .update({
        position_x: boundedX,
        position_y: boundedY,
        angle: angleDegrees,
        last_updated: new Date().toISOString()
      })
      .eq('id', ant.id)
  }

  private async moveAntTowardsFood(ant: Ant): Promise<void> {
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

  private async moveAntTowardsTarget(ant: Ant): Promise<void> {
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

  private async moveAntTowardsColony(ant: Ant): Promise<void> {
    // Get colony position
    const { data: colony } = await this.supabase
      .from('colonies')
      .select('center_x, center_y, name')
      .eq('id', ant.colony_id)
      .single()

    if (!colony) {
      console.error(`🐜 Ant ${ant.id} cannot find its colony ${ant.colony_id}`)
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

    console.log(`🐜 Moving ant ${ant.id} towards colony '${colony.name}' - distance: ${distance.toFixed(1)}`)

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

  private async collectFood(ant: Ant): Promise<void> {
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

    if (newFoodAmount <= 0) {
      console.log(`🐜 Food source ${foodSource.id} (${foodSource.food_type}) has been depleted`)
    }
  }

  private async depositFood(ant: Ant): Promise<void> {
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

    const { data: foodSources } = await this.supabase
      .from('food_sources')
      .select('*')
      .eq('simulation_id', this.simulationId)
      .gt('amount', 0)

    if (!foodSources) {
      console.log(`🐜 No food sources found in simulation ${this.simulationId}`)
      return null
    }

    // Find closest food within radius
    let closestFood: FoodSource | null = null
    let closestDistance = radius

    for (const food of foodSources) {
      const distance = Math.sqrt(
        (food.position_x - x) ** 2 + (food.position_y - y) ** 2
      )
      
      if (distance < closestDistance) {
        closestDistance = distance
        closestFood = food
      }
    }

    if (closestFood) {
      console.log(`🐜 Found nearby food: ${closestFood.food_type} at distance ${closestDistance.toFixed(1)}`)
    }

    return closestFood
  }

  private async getDistanceToTarget(ant: Ant, targetId: string): Promise<number> {
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
} 