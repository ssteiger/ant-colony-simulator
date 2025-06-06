import type { SupabaseClient } from '@supabase/supabase-js'
import type { Database } from '../types/ant-colony'
import { SimulationEngine } from './SimulationEngine'
import { ColonyManager } from './ColonyManager'
import { AntBehaviorManager } from './AntBehaviorManager'
import { EnvironmentManager } from './EnvironmentManager'
import { PheromoneManager } from './PheromoneManager'

export interface SimulationStats {
  totalAnts: number
  activeColonies: number
  totalFoodCollected: number
  activePheromoneTrails: number
  currentTick: number
}

export class AntColonySimulator {
  private supabase: SupabaseClient<Database>
  private simulationEngine: SimulationEngine
  private colonyManager: ColonyManager
  private antBehaviorManager: AntBehaviorManager
  private environmentManager: EnvironmentManager
  private pheromoneManager: PheromoneManager
  private isRunning = false
  private currentSimulationId: string | null = null

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase
    this.simulationEngine = new SimulationEngine(supabase)
    this.colonyManager = new ColonyManager(supabase)
    this.antBehaviorManager = new AntBehaviorManager(supabase)
    this.environmentManager = new EnvironmentManager(supabase)
    this.pheromoneManager = new PheromoneManager(supabase)
  }

  async start(): Promise<void> {
    if (this.isRunning) {
      console.warn('Simulation is already running')
      return
    }

    try {
      // Get or create an active simulation
      const simulation = await this.getOrCreateActiveSimulation()
      this.currentSimulationId = simulation.id

      console.log(`Starting simulation: ${simulation.name} (ID: ${simulation.id})`)

      // Initialize all managers with the simulation
      await this.colonyManager.initialize(simulation.id)
      await this.antBehaviorManager.initialize(simulation.id)
      await this.environmentManager.initialize(simulation.id)
      await this.pheromoneManager.initialize(simulation.id)

      // Start the simulation engine
      await this.simulationEngine.start(simulation.id, {
        tickInterval: 100, // 100ms between ticks
        onTick: this.handleTick.bind(this)
      })

      this.isRunning = true
      console.log('Ant Colony Simulator started successfully')
    } catch (error) {
      console.error('Failed to start simulation:', error)
      throw error
    }
  }

  async stop(): Promise<void> {
    if (!this.isRunning) {
      return
    }

    try {
      await this.simulationEngine.stop()
      this.isRunning = false
      console.log('Ant Colony Simulator stopped')
    } catch (error) {
      console.error('Error stopping simulation:', error)
      throw error
    }
  }

  async getSimulationStats(): Promise<SimulationStats> {
    if (!this.currentSimulationId) {
      throw new Error('No active simulation')
    }

    const [antsResult, coloniesResult, pheromoneResult, simulationResult] = await Promise.all([
      this.supabase
        .from('ants')
        .select(`
          id,
          colonies!inner(simulation_id)
        `)
        .neq('state', 'dead'),
      
      this.supabase
        .from('colonies')
        .select('id, resources')
        .eq('simulation_id', this.currentSimulationId)
        .eq('is_active', true),
      
      this.supabase
        .from('pheromone_trails')
        .select('id')
        .gt('strength', 0.1),
      
      this.supabase
        .from('simulations')
        .select('current_tick')
        .eq('id', this.currentSimulationId)
        .single()
    ])

    // Filter ants for this simulation
    const simulationAnts = antsResult.data?.filter(ant => 
      (ant.colonies as any)?.simulation_id === this.currentSimulationId
    ) || []

    const totalFoodCollected = coloniesResult.data?.reduce((total, colony) => {
      const resources = colony.resources as Record<string, number>
      return total + (resources?.seeds || 0) + (resources?.sugar || 0) + (resources?.protein || 0)
    }, 0) || 0

    return {
      totalAnts: simulationAnts.length,
      activeColonies: coloniesResult.data?.length || 0,
      totalFoodCollected,
      activePheromoneTrails: pheromoneResult.data?.length || 0,
      currentTick: simulationResult.data?.current_tick || 0
    }
  }

  private async getOrCreateActiveSimulation() {
    // First, try to get an active simulation
    const { data: existingSimulation } = await this.supabase
      .from('simulations')
      .select('*')
      .eq('is_active', true)
      .single()

    if (existingSimulation) {
      return existingSimulation
    }

    // Create a new simulation if none exists
    const { data: newSimulation, error } = await this.supabase
      .from('simulations')
      .insert({
        name: 'Ant Colony Simulation',
        description: 'A complex ant colony ecosystem simulation',
        world_width: 1200,
        world_height: 800,
        is_active: true,
        simulation_speed: 1.0,
        current_tick: 0
      })
      .select()
      .single()

    if (error || !newSimulation) {
      throw new Error(`Failed to create simulation: ${error?.message}`)
    }

    return newSimulation
  }

  private async handleTick(tick: number): Promise<void> {
    try {
      // Update simulation tick
      await this.supabase
        .from('simulations')
        .update({ current_tick: tick })
        .eq('id', this.currentSimulationId!)

      // Process ant behaviors
      await this.antBehaviorManager.processTick(tick)

      // Update pheromone trails
      await this.pheromoneManager.processTick(tick)

      // Process environment changes
      await this.environmentManager.processTick(tick)

      // Update colony states
      await this.colonyManager.processTick(tick)

      // Log progress every 100 ticks
      if (tick % 100 === 0) {
        console.log(`Simulation tick: ${tick}`)
      }
    } catch (error) {
      console.error(`Error processing tick ${tick}:`, error)
    }
  }
} 