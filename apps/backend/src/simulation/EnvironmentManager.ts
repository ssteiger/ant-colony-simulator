import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database } from '../types/ant-colony'

export class EnvironmentManager {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    console.log('EnvironmentManager initialized for simulation:', simulationId)
  }

  async processTick(tick: number): Promise<void> {
    if (!this.simulationId) {
      throw new Error('EnvironmentManager not initialized')
    }

    // Regenerate food sources
    await this.regenerateFoodSources()

    // Update weather occasionally
    if (tick % 1000 === 0) {
      await this.updateWeather()
    }

    // Spawn new food sources occasionally
    if (tick % 500 === 0) {
      await this.spawnNewFoodSources()
    }
  }

  private async regenerateFoodSources(): Promise<void> {
    const { data: foodSources } = await this.supabase
      .from('food_sources')
      .select('*')
      .eq('simulation_id', this.simulationId!)
      .eq('is_renewable', true)
      .lt('amount', 'max_amount')

    if (!foodSources) return

    for (const foodSource of foodSources) {
      if (foodSource.regeneration_rate > 0) {
        const newAmount = Math.min(
          foodSource.max_amount,
          foodSource.amount + foodSource.regeneration_rate
        )

        await this.supabase
          .from('food_sources')
          .update({ amount: newAmount })
          .eq('id', foodSource.id)
      }
    }
  }

  private async updateWeather(): Promise<void> {
    const weatherTypes = ['clear', 'rain', 'wind', 'storm']
    const randomWeather = weatherTypes[Math.floor(Math.random() * weatherTypes.length)]
    const randomIntensity = Math.random()

    await this.supabase
      .from('simulations')
      .update({
        weather_type: randomWeather,
        weather_intensity: randomIntensity
      })
      .eq('id', this.simulationId!)

    console.log(`Weather updated to ${randomWeather} with intensity ${randomIntensity.toFixed(2)}`)
  }

  private async spawnNewFoodSources(): Promise<void> {
    // Randomly spawn new food sources if there are too few
    const { data: existingFood } = await this.supabase
      .from('food_sources')
      .select('id')
      .eq('simulation_id', this.simulationId!)
      .gt('amount', 0)

    const foodCount = existingFood?.length || 0

    // Maintain at least 10 food sources
    if (foodCount < 10) {
      const foodTypes = ['seeds', 'sugar', 'protein', 'fruit']
      const randomFoodType = foodTypes[Math.floor(Math.random() * foodTypes.length)]

      // Random position in the world
      const x = Math.random() * 1200
      const y = Math.random() * 800
      const amount = 20 + Math.random() * 30 // 20-50 units

      await this.supabase
        .from('food_sources')
        .insert({
          simulation_id: this.simulationId!,
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

      console.log(`Spawned new ${randomFoodType} food source at (${x.toFixed(1)}, ${y.toFixed(1)})`)
    }
  }

  async getEnvironmentalEffects(x: number, y: number) {
    // Get current weather
    const { data: simulation } = await this.supabase
      .from('simulations')
      .select('weather_type, weather_intensity, season, time_of_day')
      .eq('id', this.simulationId!)
      .single()

    if (!simulation) return { speedModifier: 1.0, visibilityModifier: 1.0 }

    let speedModifier = 1.0
    let visibilityModifier = 1.0

    // Weather effects
    switch (simulation.weather_type) {
      case 'rain':
        speedModifier *= (1.0 - simulation.weather_intensity * 0.3)
        visibilityModifier *= (1.0 - simulation.weather_intensity * 0.2)
        break
      case 'wind':
        speedModifier *= (1.0 + simulation.weather_intensity * 0.1) // Wind can help
        break
      case 'storm':
        speedModifier *= (1.0 - simulation.weather_intensity * 0.5)
        visibilityModifier *= (1.0 - simulation.weather_intensity * 0.4)
        break
    }

    // Time of day effects (darkness reduces visibility)
    const hour = Math.floor(simulation.time_of_day / 60)
    if (hour < 6 || hour > 20) { // Night time
      visibilityModifier *= 0.7
    }

    return { speedModifier, visibilityModifier }
  }
} 