import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database } from '../types/ant-colony'

export class EnvironmentManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
    console.log('🌍 EnvironmentManager: Constructor initialized')
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    console.log('🌍 EnvironmentManager: Initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('EnvironmentManager not initialized')
    }

    console.log(`🌍 EnvironmentManager: Processing tick ${tick}`)

    // Regenerate food sources
    await this.regenerateFoodSources()

    // Update weather occasionally
    if (tick % 1000 === 0) {
      console.log(`🌍 EnvironmentManager: Updating weather at tick ${tick}`)
      await this.updateWeather()
    }

    // Spawn new food sources occasionally
    if (tick % 500 === 0) {
      console.log(`🌍 EnvironmentManager: Spawning new food sources at tick ${tick}`)
      await this.spawnNewFoodSources()
    }
  }

  private async regenerateFoodSources(): Promise<void> {
    if (!this.simulationId) return

    const { data: foodSources } = await this.supabase
      .from('food_sources')
      .select('*')
      .eq('simulation_id', this.simulationId)
      .eq('is_renewable', true)
      .lt('amount', 'max_amount')

    if (!foodSources || foodSources.length === 0) {
      return
    }

    console.log(`🌍 EnvironmentManager: Regenerating ${foodSources.length} food sources`)
    let regeneratedCount = 0

    for (const foodSource of foodSources) {
      if (foodSource.regeneration_rate > 0) {
        const newAmount = Math.min(
          foodSource.max_amount,
          foodSource.amount + foodSource.regeneration_rate
        )

        if (newAmount > foodSource.amount) {
          const regenerated = newAmount - foodSource.amount
          console.log(`🌍 Regenerating ${foodSource.food_type} at (${foodSource.position_x.toFixed(1)}, ${foodSource.position_y.toFixed(1)}): +${regenerated.toFixed(2)} (${newAmount.toFixed(2)}/${foodSource.max_amount})`)
          
          await this.supabase
            .from('food_sources')
            .update({ amount: newAmount })
            .eq('id', foodSource.id)
          
          regeneratedCount++
        }
      }
    }

    if (regeneratedCount > 0) {
      console.log(`🌍 EnvironmentManager: Successfully regenerated ${regeneratedCount} food sources`)
    }
  }

  private async updateWeather(): Promise<void> {
    if (!this.simulationId) return

    const weatherTypes = ['clear', 'rain', 'wind', 'storm']
    const randomWeather = weatherTypes[Math.floor(Math.random() * weatherTypes.length)]
    const randomIntensity = Math.random()

    console.log(`🌍 EnvironmentManager: Changing weather to ${randomWeather} with intensity ${randomIntensity.toFixed(2)}`)

    await this.supabase
      .from('simulations')
      .update({
        weather_type: randomWeather,
        weather_intensity: randomIntensity
      })
      .eq('id', this.simulationId)

    console.log(`🌍 EnvironmentManager: ✅ Weather updated to ${randomWeather} (intensity: ${randomIntensity.toFixed(2)})`)
  }

  private async spawnNewFoodSources(): Promise<void> {
    if (!this.simulationId) return

    // Randomly spawn new food sources if there are too few
    const { data: existingFood } = await this.supabase
      .from('food_sources')
      .select('id, food_type, amount')
      .eq('simulation_id', this.simulationId)
      .gt('amount', 0)

    const foodCount = existingFood?.length || 0
    console.log(`🌍 EnvironmentManager: Current food sources: ${foodCount}`)

    // Maintain at least 10 food sources
    if (foodCount < 10) {
      const neededFood = 10 - foodCount
      console.log(`🌍 EnvironmentManager: Spawning ${neededFood} new food sources`)

      for (let i = 0; i < neededFood; i++) {
        const foodTypes = ['seeds', 'sugar', 'protein', 'fruit']
        const randomFoodType = foodTypes[Math.floor(Math.random() * foodTypes.length)]

        // Random position in the world
        const x = Math.random() * 1200
        const y = Math.random() * 800
        const amount = 20 + Math.random() * 30 // 20-50 units

        try {
          await this.supabase
            .from('food_sources')
            .insert({
              simulation_id: this.simulationId,
              food_type: randomFoodType,
              position_x: x,
              position_y: y,
              amount,
              max_amount: amount,
              regeneration_rate: Math.random() * 0.1, // 0-0.1 per tick
              discovery_difficulty: Math.random() * 0.8,
              nutritional_value: 1.0 + Math.random() * 0.5,
              spoilage_rate: Math.random() * 0.001,
              is_renewable: Math.random() > 0.3 // 70% chance of being renewable
            })

          console.log(`🌍 EnvironmentManager: ✅ Spawned ${randomFoodType} food source at (${x.toFixed(1)}, ${y.toFixed(1)}) with ${amount.toFixed(1)} units`)
        } catch (error) {
          console.error(`🌍 EnvironmentManager: ❌ Failed to spawn food source:`, error)
        }
      }
    } else {
      console.log(`🌍 EnvironmentManager: Food sources sufficient (${foodCount}/10)`)
    }
  }

  async getEnvironmentalEffects(x: number, y: number) {
    if (!this.simulationId) {
      console.warn('🌍 EnvironmentManager: Cannot get environmental effects - no simulation ID')
      return { speedModifier: 1.0, visibilityModifier: 1.0 }
    }

    // Get current weather
    const { data: simulation } = await this.supabase
      .from('simulations')
      .select('weather_type, weather_intensity, season, time_of_day')
      .eq('id', this.simulationId)
      .single()

    if (!simulation) {
      console.warn('🌍 EnvironmentManager: No simulation data found')
      return { speedModifier: 1.0, visibilityModifier: 1.0 }
    }

    let speedModifier = 1.0
    let visibilityModifier = 1.0

    console.log(`🌍 EnvironmentManager: Weather effects - Type: ${simulation.weather_type}, Intensity: ${simulation.weather_intensity}`)

    // Weather effects
    switch (simulation.weather_type) {
      case 'rain':
        speedModifier *= (1.0 - simulation.weather_intensity * 0.3)
        visibilityModifier *= (1.0 - simulation.weather_intensity * 0.2)
        console.log(`🌍 Rain effects: Speed -${(simulation.weather_intensity * 30).toFixed(1)}%, Visibility -${(simulation.weather_intensity * 20).toFixed(1)}%`)
        break
      case 'wind':
        speedModifier *= (1.0 + simulation.weather_intensity * 0.1) // Wind can help
        console.log(`🌍 Wind effects: Speed +${(simulation.weather_intensity * 10).toFixed(1)}%`)
        break
      case 'storm':
        speedModifier *= (1.0 - simulation.weather_intensity * 0.5)
        visibilityModifier *= (1.0 - simulation.weather_intensity * 0.4)
        console.log(`🌍 Storm effects: Speed -${(simulation.weather_intensity * 50).toFixed(1)}%, Visibility -${(simulation.weather_intensity * 40).toFixed(1)}%`)
        break
      default:
        console.log('🌍 Clear weather: No effects')
    }

    // Time of day effects (darkness reduces visibility)
    const hour = Math.floor(simulation.time_of_day / 60)
    if (hour < 6 || hour > 20) { // Night time
      visibilityModifier *= 0.7
      console.log('🌍 Night time: Visibility -30%')
    }

    const effects = { speedModifier, visibilityModifier }
    console.log(`🌍 EnvironmentManager: Final effects at (${x.toFixed(1)}, ${y.toFixed(1)}):`, effects)
    return effects
  }
} 