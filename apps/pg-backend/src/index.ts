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
          total_ants: stats.total_ants,
          active_combats: stats.active_combats,
          total_food_collected: stats.total_food_collected,
          pheromone_trail_count: stats.pheromone_trail_count,
          tick_number: stats.tick_number
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
