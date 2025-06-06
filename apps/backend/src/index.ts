import { AntColonySimulator } from './simulation/AntColonySimulator'
import { createSupabaseClient } from './utils/supabase'

async function startServer() {
  console.log('Starting Ant Colony Simulator...')

  try {
    // Create Supabase client
    const supabase = createSupabaseClient()

    // Test connection to Supabase
    const { data, error } = await supabase.auth.getSession()

    if (error) {
      throw new Error(`Failed to connect to Supabase: ${error.message}`)
    }

    console.log('Successfully connected to Supabase')

    // Initialize the ant colony simulator
    const simulator = new AntColonySimulator(supabase)

    // Start the simulation
    await simulator.start()

    console.log('Ant Colony Simulator started successfully')

    // Keep the process running
    process.on('SIGINT', async () => {
      console.log('Shutting down Ant Colony Simulator...')
      await simulator.stop()
      process.exit(0)
    })

    // Log simulation stats every 30 seconds
    setInterval(async () => {
      try {
        const stats = await simulator.getSimulationStats()
        console.log('Simulation Stats:', {
          totalAnts: stats.totalAnts,
          activeColonies: stats.activeColonies,
          foodCollected: stats.totalFoodCollected,
          activePheromoneTrails: stats.activePheromoneTrails,
          currentTick: stats.currentTick
        })
      } catch (error) {
        console.error('Error getting simulation stats:', error)
      }
    }, 30000)

  } catch (error) {
    console.error('Error starting Ant Colony Simulator:', error)
    process.exit(1)
  }
}

// Start the server
startServer()
