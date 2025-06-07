import type { SupabaseClient } from '@supabase/supabase-js'

import type { Database } from '../types/supabase'
import type { SimulationStats } from '../types/drizzle'

import { SimulationEngine } from './SimulationEngine'
import { ColonyManager } from './ColonyManager'
import { AntBehaviorManager } from './AntBehaviorManager'
import { EnvironmentManager } from './EnvironmentManager'
import { PheromoneManager } from './PheromoneManager'

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
    console.log('🎮 AntColonySimulator: Constructor starting')
    this.supabase = supabase
    this.simulationEngine = new SimulationEngine(supabase)
    this.colonyManager = new ColonyManager(supabase)
    this.antBehaviorManager = new AntBehaviorManager(supabase)
    this.environmentManager = new EnvironmentManager(supabase)
    this.pheromoneManager = new PheromoneManager(supabase)
    console.log('🎮 AntColonySimulator: All managers created successfully')
  }

  async start(): Promise<void> {
    if (this.isRunning) {
      console.warn('🎮 AntColonySimulator: Simulation is already running')
      return
    }

    console.log('🎮 AntColonySimulator: Starting simulation process...')

    try {
      // Get an active simulation
      console.log('🎮 AntColonySimulator: Getting active simulation...')
      const simulation = await this.getActiveSimulation()
      this.currentSimulationId = simulation.id

      console.log(`🎮 AntColonySimulator: Starting simulation: ${simulation.name} (ID: ${simulation.id})`)
      console.log(`🎮 AntColonySimulator: World dimensions: ${simulation.world_width}x${simulation.world_height}`)
      console.log(`🎮 AntColonySimulator: Current tick: ${simulation.current_tick}`)

      // Initialize all managers with the simulation
      console.log('🎮 AntColonySimulator: Initializing managers...')
      await this.colonyManager.initialize(simulation.id)
      console.log('🎮 AntColonySimulator: ✓ ColonyManager initialized')
      
      await this.antBehaviorManager.initialize(simulation.id)
      console.log('🎮 AntColonySimulator: ✓ AntBehaviorManager initialized')
      
      await this.environmentManager.initialize(simulation.id)
      console.log('🎮 AntColonySimulator: ✓ EnvironmentManager initialized')
      
      await this.pheromoneManager.initialize(simulation.id)
      console.log('🎮 AntColonySimulator: ✓ PheromoneManager initialized')

      // Start the simulation engine
      console.log('🎮 AntColonySimulator: Starting simulation engine...')
      await this.simulationEngine.start(simulation.id, {
        tickInterval: 100, // 100ms between ticks
        onTick: this.handleTick.bind(this)
      })

      this.isRunning = true
      console.log('🎮 AntColonySimulator: ✅ Simulation started successfully!')
      console.log('🎮 AntColonySimulator: Tick interval: 100ms')
    } catch (error) {
      console.error('🎮 AntColonySimulator: ❌ Failed to start simulation:', error)
      throw error
    }
  }

  async stop(): Promise<void> {
    if (!this.isRunning) {
      console.log('🎮 AntColonySimulator: Simulation is not running')
      return
    }

    console.log('🎮 AntColonySimulator: Stopping simulation...')

    try {
      await this.simulationEngine.stop()
      this.isRunning = false
      console.log('🎮 AntColonySimulator: ✅ Simulation stopped successfully')
    } catch (error) {
      console.error('🎮 AntColonySimulator: ❌ Error stopping simulation:', error)
      throw error
    }
  }

  async getSimulationStats(): Promise<SimulationStats> {
    if (!this.currentSimulationId) {
      throw new Error('No active simulation')
    }

    console.log('🎮 AntColonySimulator: Gathering simulation statistics...')

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
      (ant.colonies as unknown as { simulation_id: string })?.simulation_id === this.currentSimulationId
    ) || []

    const totalFoodCollected = coloniesResult.data?.reduce((total, colony) => {
      const resources = colony.resources as Record<string, number>
      return total + (resources?.seeds || 0) + (resources?.sugar || 0) + (resources?.protein || 0)
    }, 0) || 0

    const stats = {
      totalAnts: simulationAnts.length,
      activeColonies: coloniesResult.data?.length || 0,
      totalFoodCollected,
      activePheromoneTrails: pheromoneResult.data?.length || 0,
      currentTick: simulationResult.data?.current_tick || 0
    }

    console.log('🎮 AntColonySimulator: Statistics gathered:', stats)
    return stats
  }

  private async getActiveSimulation() {
    console.log('🎮 AntColonySimulator: Looking for existing active simulation...')
    
    // Try to get an active simulation
    const { data: existingSimulation, error } = await this.supabase
      .from('simulations')
      .select('*')
      .eq('is_active', true)
      .single()

    if (error || !existingSimulation) {
      console.error('🎮 AntColonySimulator: No active simulation found')
      throw new Error('No active simulation found. Please create a simulation first.')
    }

    console.log(`🎮 AntColonySimulator: Found existing simulation: ${existingSimulation.name} (ID: ${existingSimulation.id})`)
    return existingSimulation
  }

  private async handleTick(tick: number): Promise<void> {
    const tickStartTime = Date.now()
    
    try {
      console.log(`🎮 AntColonySimulator: ⏰ Starting tick ${tick}`)

      // Update simulation tick
      if (this.currentSimulationId) {
        await this.supabase
          .from('simulations')
          .update({ current_tick: tick })
          .eq('id', this.currentSimulationId)
      }

      // Process ant behaviors
      console.log('🎮 AntColonySimulator: Processing ant behaviors...')
      const antStartTime = Date.now()
      await this.antBehaviorManager.processTick(tick)
      console.log(`🎮 AntColonySimulator: ✓ Ant behaviors processed (${Date.now() - antStartTime}ms)`)

      // Update pheromone trails
      console.log('🎮 AntColonySimulator: Processing pheromone trails...')
      const pheromoneStartTime = Date.now()
      await this.pheromoneManager.processTick(tick)
      console.log(`🎮 AntColonySimulator: ✓ Pheromone trails processed (${Date.now() - pheromoneStartTime}ms)`)

      // Process environment changes
      console.log('🎮 AntColonySimulator: Processing environment...')
      const envStartTime = Date.now()
      await this.environmentManager.processTick(tick)
      console.log(`🎮 AntColonySimulator: ✓ Environment processed (${Date.now() - envStartTime}ms)`)

      // Update colony states
      console.log('🎮 AntColonySimulator: Processing colonies...')
      const colonyStartTime = Date.now()
      await this.colonyManager.processTick(tick)
      console.log(`🎮 AntColonySimulator: ✓ Colonies processed (${Date.now() - colonyStartTime}ms)`)

      const totalTickTime = Date.now() - tickStartTime
      console.log(`🎮 AntColonySimulator: ✅ Tick ${tick} completed (${totalTickTime}ms)`)

      // Log progress every 10 ticks with stats
      if (tick % 10 === 0) {
        try {
          const stats = await this.getSimulationStats()
          console.log(`🎮 AntColonySimulator: 📊 Tick ${tick} Stats - Ants: ${stats.totalAnts}, Colonies: ${stats.activeColonies}, Food: ${stats.totalFoodCollected}, Pheromones: ${stats.activePheromoneTrails}`)
        } catch (error) {
          console.warn(`🎮 AntColonySimulator: Could not gather stats at tick ${tick}:`, error)
        }
      }
    } catch (error) {
      console.error(`🎮 AntColonySimulator: ❌ Error processing tick ${tick}:`, error)
    }
  }
} 