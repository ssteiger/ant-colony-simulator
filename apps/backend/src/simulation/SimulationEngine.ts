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
  }

  async start(simulationId: string, options: SimulationEngineOptions): Promise<void> {
    if (this.isRunning) {
      throw new Error('Simulation engine is already running')
    }

    this.simulationId = simulationId
    
    // Get current tick from database
    const { data: simulation } = await this.supabase
      .from('simulations')
      .select('current_tick')
      .eq('id', simulationId)
      .single()

    this.currentTick = simulation?.current_tick || 0
    this.isRunning = true

    console.log(`Starting simulation engine at tick ${this.currentTick}`)

    // Start the simulation loop
    this.intervalId = setInterval(async () => {
      try {
        this.currentTick++
        console.log(`Simulation tick ${this.currentTick} starting...`)
        await options.onTick(this.currentTick)
        if (this.currentTick % 10 === 0) {
          console.log(`Completed tick ${this.currentTick}`)
        }
      } catch (error) {
        console.error(`Error in simulation tick ${this.currentTick}:`, error)
      }
    }, options.tickInterval)
  }

  async stop(): Promise<void> {
    if (!this.isRunning) {
      return
    }

    if (this.intervalId) {
      clearInterval(this.intervalId)
      this.intervalId = null
    }

    this.isRunning = false
    console.log(`Simulation engine stopped at tick ${this.currentTick}`)
  }

  getCurrentTick(): number {
    return this.currentTick
  }

  isSimulationRunning(): boolean {
    return this.isRunning
  }
} 