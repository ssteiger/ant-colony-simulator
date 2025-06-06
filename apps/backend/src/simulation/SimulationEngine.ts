import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database } from '../types/ant-colony'

export interface SimulationEngineOptions {
  tickInterval: number // milliseconds between ticks
  onTick: (tick: number) => Promise<void>
}

export class SimulationEngine {
  private supabase: SupabaseClient<Database>
  private intervalId: NodeJS.Timeout | null = null
  private currentTick = 0
  private isRunning = false
  private simulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
    console.log('⚙️ SimulationEngine: Constructor initialized')
  }

  async start(simulationId: string, options: SimulationEngineOptions): Promise<void> {
    if (this.isRunning) {
      console.warn('⚙️ SimulationEngine: Engine is already running')
      throw new Error('Simulation engine is already running')
    }

    console.log(`⚙️ SimulationEngine: Starting engine for simulation ${simulationId}`)
    console.log(`⚙️ SimulationEngine: Tick interval: ${options.tickInterval}ms`)

    this.simulationId = simulationId
    
    // Get current tick from database
    const { data: simulation } = await this.supabase
      .from('simulations')
      .select('current_tick')
      .eq('id', simulationId)
      .single()

    this.currentTick = simulation?.current_tick || 0
    this.isRunning = true

    console.log(`⚙️ SimulationEngine: Starting at tick ${this.currentTick}`)

    // Start the simulation loop
    this.intervalId = setInterval(async () => {
      const tickStartTime = Date.now()
      try {
        this.currentTick++
        console.log(`⚙️ SimulationEngine: ⏰ Tick ${this.currentTick} starting...`)
        
        await options.onTick(this.currentTick)
        
        const tickDuration = Date.now() - tickStartTime
        if (this.currentTick % 10 === 0) {
          console.log(`⚙️ SimulationEngine: ✅ Tick ${this.currentTick} completed in ${tickDuration}ms`)
        }

        // Performance warning if tick takes too long
        if (tickDuration > options.tickInterval * 0.8) {
          console.warn(`⚙️ SimulationEngine: ⚠️ Tick ${this.currentTick} took ${tickDuration}ms (${((tickDuration / options.tickInterval) * 100).toFixed(1)}% of interval)`)
        }
      } catch (error) {
        const tickDuration = Date.now() - tickStartTime
        console.error(`⚙️ SimulationEngine: ❌ Error in tick ${this.currentTick} after ${tickDuration}ms:`, error)
      }
    }, options.tickInterval)

    console.log('⚙️ SimulationEngine: ✅ Engine started successfully')
  }

  async stop(): Promise<void> {
    if (!this.isRunning) {
      console.log('⚙️ SimulationEngine: Engine is not running')
      return
    }

    console.log(`⚙️ SimulationEngine: Stopping engine at tick ${this.currentTick}...`)

    if (this.intervalId) {
      clearInterval(this.intervalId)
      this.intervalId = null
      console.log('⚙️ SimulationEngine: Interval cleared')
    }

    this.isRunning = false
    console.log(`⚙️ SimulationEngine: ✅ Engine stopped successfully at tick ${this.currentTick}`)
  }

  getCurrentTick(): number {
    console.log(`⚙️ SimulationEngine: Current tick requested: ${this.currentTick}`)
    return this.currentTick
  }

  isSimulationRunning(): boolean {
    console.log(`⚙️ SimulationEngine: Running status requested: ${this.isRunning}`)
    return this.isRunning
  }
} 