import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database, Ant, FoodSource } from '../types/ant-colony'

export class AntBehaviorManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    console.log('AntBehaviorManager initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('AntBehaviorManager not initialized')
    }

    // Get all living ants in the simulation
    const { data: ants } = await this.supabase
      .from('ants')
      .select(`
        *,
        colonies(simulation_id)
      `)
      .neq('state', 'dead')

    if (!ants) return

    // Filter ants for this simulation
    const simulationAnts = ants.filter(ant => 
      (ant.colonies as any)?.simulation_id === this.simulationId
    )

    // Process each ant's behavior
    for (const ant of simulationAnts) {
      await this.processAntBehavior(ant, tick)
    }
  }

  private async processAntBehavior(ant: Ant, tick: number): Promise<void> {
    try {
      // Age the ant
      const newAge = ant.age_ticks + 1
      
      // Check if ant should die of old age
      if (newAge > 10000) { // Simplified lifespan
        await this.killAnt(ant.id, 'old_age')
        return
      }

      // Decrease energy over time
      const energyDecay = 1
      const newEnergy = Math.max(0, ant.energy - energyDecay)

      // If energy is too low, ant dies
      if (newEnergy <= 0) {
        await this.killAnt(ant.id, 'starvation')
        return
      }

      // Determine ant's next action based on current state
      const nextAction = await this.determineAntAction(ant)
      
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

    } catch (error) {
      console.error(`Error processing ant ${ant.id} behavior:`, error)
    }
  }

  private async determineAntAction(ant: Ant): Promise<string> {
    // Simple state machine for ant behavior
    switch (ant.state) {
      case 'wandering':
        // Look for food or follow pheromone trails
        const nearbyFood = await this.findNearbyFood(ant.position_x, ant.position_y, 50)
        if (nearbyFood) {
          return 'seek_food'
        }
        return 'wander'

      case 'seeking_food':
        // Check if ant reached its food target
        if (ant.target_id) {
          const distance = await this.getDistanceToTarget(ant, ant.target_id)
          if (distance < 5) {
            return 'collect_food'
          }
        }
        return 'move_to_food'

      case 'carrying_food':
        // Return to colony
        return 'return_to_colony'

      default:
        return 'wander'
    }
  }

  private async executeAntAction(ant: Ant, action: string): Promise<void> {
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
    }
  }

  private async moveAntRandomly(ant: Ant): Promise<void> {
    // Generate random movement
    const moveDistance = ant.current_speed
    const randomAngle = Math.random() * 2 * Math.PI
    
    const newX = ant.position_x + Math.cos(randomAngle) * moveDistance
    const newY = ant.position_y + Math.sin(randomAngle) * moveDistance

    // Simple boundary checking (keep ants in world bounds)
    const boundedX = Math.max(0, Math.min(1200, newX))
    const boundedY = Math.max(0, Math.min(800, newY))

    await this.supabase
      .from('ants')
      .update({
        position_x: boundedX,
        position_y: boundedY,
        angle: randomAngle
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

      await this.supabase
        .from('ants')
        .update({
          position_x: newX,
          position_y: newY,
          angle,
          state: 'seeking_food',
          target_x: nearbyFood.position_x,
          target_y: nearbyFood.position_y,
          target_id: nearbyFood.id,
          target_type: 'food_source'
        })
        .eq('id', ant.id)
    }
  }

  private async moveAntTowardsTarget(ant: Ant): Promise<void> {
    if (!ant.target_x || !ant.target_y) return

    const angle = Math.atan2(ant.target_y - ant.position_y, ant.target_x - ant.position_x)
    const newX = ant.position_x + Math.cos(angle) * ant.current_speed
    const newY = ant.position_y + Math.sin(angle) * ant.current_speed

    await this.supabase
      .from('ants')
      .update({
        position_x: newX,
        position_y: newY,
        angle
      })
      .eq('id', ant.id)
  }

  private async moveAntTowardsColony(ant: Ant): Promise<void> {
    // Get colony position
    const { data: colony } = await this.supabase
      .from('colonies')
      .select('center_x, center_y')
      .eq('id', ant.colony_id)
      .single()

    if (!colony) return

    const angle = Math.atan2(colony.center_y - ant.position_y, colony.center_x - ant.position_x)
    const newX = ant.position_x + Math.cos(angle) * ant.current_speed
    const newY = ant.position_y + Math.sin(angle) * ant.current_speed

    // Check if ant reached colony
    const distance = Math.sqrt(
      Math.pow(colony.center_x - newX, 2) + Math.pow(colony.center_y - newY, 2)
    )

    if (distance < 15) {
      // Ant reached colony - deposit food and change state
      await this.depositFood(ant)
    } else {
      await this.supabase
        .from('ants')
        .update({
          position_x: newX,
          position_y: newY,
          angle
        })
        .eq('id', ant.id)
    }
  }

  private async collectFood(ant: Ant): Promise<void> {
    if (!ant.target_id) return

    // Get food source
    const { data: foodSource } = await this.supabase
      .from('food_sources')
      .select('*')
      .eq('id', ant.target_id)
      .single()

    if (!foodSource || foodSource.amount <= 0) {
      // Food is gone, switch to wandering
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

    // Update food source
    await this.supabase
      .from('food_sources')
      .update({ amount: newFoodAmount })
      .eq('id', foodSource.id)

    // Update ant to carry food
    await this.supabase
      .from('ants')
      .update({
        state: 'carrying_food',
        carried_resources: { [foodSource.food_type]: collectionAmount },
        target_id: null,
        target_x: null,
        target_y: null,
        target_type: null
      })
      .eq('id', ant.id)
  }

  private async depositFood(ant: Ant): Promise<void> {
    const carriedResources = (ant.carried_resources as any) || {}
    
    if (Object.keys(carriedResources).length === 0) {
      // No food to deposit, start wandering
      await this.supabase
        .from('ants')
        .update({ state: 'wandering' })
        .eq('id', ant.id)
      return
    }

    // Get colony current resources
    const { data: colony } = await this.supabase
      .from('colonies')
      .select('resources')
      .eq('id', ant.colony_id)
      .single()

    if (!colony) return

    const colonyResources = (colony.resources as any) || {}
    
    // Add carried resources to colony
    const updatedResources = { ...colonyResources }
    for (const [foodType, amount] of Object.entries(carriedResources)) {
      updatedResources[foodType] = (updatedResources[foodType] || 0) + (amount as number)
    }

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
  }

  private async findNearbyFood(x: number, y: number, radius: number): Promise<FoodSource | null> {
    const { data: foodSources } = await this.supabase
      .from('food_sources')
      .select('*')
      .eq('simulation_id', this.simulationId!)
      .gt('amount', 0)

    if (!foodSources) return null

    // Find closest food within radius
    let closestFood: FoodSource | null = null
    let closestDistance = radius

    for (const food of foodSources) {
      const distance = Math.sqrt(
        Math.pow(food.position_x - x, 2) + Math.pow(food.position_y - y, 2)
      )
      
      if (distance < closestDistance) {
        closestDistance = distance
        closestFood = food
      }
    }

    return closestFood
  }

  private async getDistanceToTarget(ant: Ant, targetId: string): Promise<number> {
    const { data: target } = await this.supabase
      .from('food_sources')
      .select('position_x, position_y')
      .eq('id', targetId)
      .single()

    if (!target) return Infinity

    return Math.sqrt(
      Math.pow(target.position_x - ant.position_x, 2) + 
      Math.pow(target.position_y - ant.position_y, 2)
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

    console.log(`Ant ${antId} died from ${cause}`)
  }
} 