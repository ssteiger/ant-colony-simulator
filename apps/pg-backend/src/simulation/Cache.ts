import type { SupabaseClient } from '@supabase/supabase-js'
import type { FoodSource } from '../types/drizzle'
import type { Database } from '../types/supabase'

export interface WorldBounds {
  width: number
  height: number
}

export interface ColonyInfo {
  center_x: number
  center_y: number
  name: string
}

export class SimulationCache {
  private supabase: SupabaseClient<Database>
  private simulationId: string | null = null
  
  // Cache data
  private worldBounds: WorldBounds | null = null
  private foodSourcesCache: FoodSource[] = []
  private coloniesCache: Map<string, ColonyInfo> = new Map()
  
  // Cache management
  private lastCacheUpdate = 0
  private readonly CACHE_REFRESH_INTERVAL = 10000 // 10 seconds

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
    console.log('🗄️ SimulationCache: Constructor initialized')
  }

  async initialize(simulationId: string): Promise<void> {
    this.simulationId = simulationId
    await this.refreshCache()
    console.log('🗄️ SimulationCache: Initialized for simulation:', simulationId)
  }

  async refreshCacheIfNeeded(): Promise<void> {
    const now = Date.now()
    if (now - this.lastCacheUpdate > this.CACHE_REFRESH_INTERVAL) {
      await this.refreshCache()
    }
  }

  private async refreshCache(): Promise<void> {
    if (!this.simulationId) return

    try {
      console.log('🗄️ SimulationCache: Refreshing cache...')
      
      // Cache world bounds
      const { data: simulation } = await this.supabase
        .from('simulations')
        .select('world_width, world_height')
        .eq('id', this.simulationId)
        .single()

      if (simulation) {
        this.worldBounds = {
          width: simulation.world_width,
          height: simulation.world_height
        }
      }

      // Cache food sources
      const { data: foodSources } = await this.supabase
        .from('food_sources')
        .select('*')
        .eq('simulation_id', this.simulationId)
        .gt('amount', 0)

      this.foodSourcesCache = foodSources || []

      // Cache colony positions
      const { data: colonies } = await this.supabase
        .from('colonies')
        .select('id, center_x, center_y, name')
        .eq('simulation_id', this.simulationId)
        .eq('is_active', true)

      this.coloniesCache.clear()
      if (colonies) {
        for (const colony of colonies) {
          this.coloniesCache.set(colony.id, {
            center_x: colony.center_x,
            center_y: colony.center_y,
            name: colony.name
          })
        }
      }

      this.lastCacheUpdate = Date.now()
      console.log(`🗄️ SimulationCache: Cache refreshed - ${this.foodSourcesCache.length} food sources, ${this.coloniesCache.size} colonies`)
    } catch (error) {
      console.error('🗄️ SimulationCache: Error refreshing cache:', error)
    }
  }

  // Getters for cached data
  getWorldBounds(): WorldBounds | null {
    return this.worldBounds
  }

  getFoodSources(): FoodSource[] {
    return this.foodSourcesCache
  }

  getColony(colonyId: string): ColonyInfo | undefined {
    return this.coloniesCache.get(colonyId)
  }

  getAllColonies(): Map<string, ColonyInfo> {
    return this.coloniesCache
  }

  // Update methods for cache synchronization
  updateFoodSource(foodId: string, newAmount: number): void {
    const foodIndex = this.foodSourcesCache.findIndex(food => food.id === foodId)
    if (foodIndex !== -1) {
      this.foodSourcesCache[foodIndex].amount = newAmount
      // Remove from cache if depleted
      if (newAmount <= 0) {
        this.foodSourcesCache.splice(foodIndex, 1)
        console.log(`🗄️ SimulationCache: Removed depleted food source ${foodId} from cache`)
      }
    }
  }

  addFoodSource(foodSource: FoodSource): void {
    if (foodSource.amount > 0) {
      this.foodSourcesCache.push(foodSource)
      console.log(`🗄️ SimulationCache: Added new food source ${foodSource.id} to cache`)
    }
  }

  removeFoodSource(foodId: string): void {
    const foodIndex = this.foodSourcesCache.findIndex(food => food.id === foodId)
    if (foodIndex !== -1) {
      this.foodSourcesCache.splice(foodIndex, 1)
      console.log(`🗄️ SimulationCache: Removed food source ${foodId} from cache`)
    }
  }

  updateColony(colonyId: string, colonyInfo: ColonyInfo): void {
    this.coloniesCache.set(colonyId, colonyInfo)
    console.log(`🗄️ SimulationCache: Updated colony ${colonyId} in cache`)
  }

  removeColony(colonyId: string): void {
    this.coloniesCache.delete(colonyId)
    console.log(`🗄️ SimulationCache: Removed colony ${colonyId} from cache`)
  }

  // Utility methods
  getCacheStats(): {
    worldBounds: boolean
    foodSourceCount: number
    colonyCount: number
    lastUpdate: number
    cacheAge: number
  } {
    return {
      worldBounds: this.worldBounds !== null,
      foodSourceCount: this.foodSourcesCache.length,
      colonyCount: this.coloniesCache.size,
      lastUpdate: this.lastCacheUpdate,
      cacheAge: Date.now() - this.lastCacheUpdate
    }
  }

  // Force refresh method for external use
  async forceRefresh(): Promise<void> {
    await this.refreshCache()
  }
}
