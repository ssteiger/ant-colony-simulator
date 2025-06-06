import { useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema } from '@ant-colony-simulator/db-drizzle'

interface Simulation {
  id: string
  name: string
  world_width: number
  world_height: number
  current_tick: number | null
  is_active: boolean | null
}

interface Ant {
  id: string
  position_x: string
  position_y: string
  colony_id: string
  state: string
}

interface Colony {
  id: string
  name: string
  center_x: string
  center_y: string
  radius: string
  color_hue: number
}

interface FoodSource {
  id: string
  position_x: string
  position_y: string
  food_type: string
  amount: string
}

const getSimulationData = createServerFn({ method: 'GET' })
  .handler(async () => {
    try {
      // Get the first simulation for now (simplified)
      const simulations = await postgres_db
        .select()
        .from(schema.simulations)
        .limit(1)

      if (simulations.length === 0) {
        return { simulation: null, ants: [], colonies: [], foodSources: [] }
      }

      const simulation = simulations[0]

      // Get all colonies (simplified)
      const colonies = await postgres_db
        .select({
          id: schema.colonies.id,
          name: schema.colonies.name,
          center_x: schema.colonies.center_x,
          center_y: schema.colonies.center_y,
          radius: schema.colonies.radius,
          color_hue: schema.colonies.color_hue
        })
        .from(schema.colonies)
        .limit(100)

      // Get all ants (simplified)
      const ants = await postgres_db
        .select({
          id: schema.ants.id,
          position_x: schema.ants.position_x,
          position_y: schema.ants.position_y,
          colony_id: schema.ants.colony_id,
          state: schema.ants.state
        })
        .from(schema.ants)
        .limit(500)

      // Get all food sources (simplified)
      const foodSources = await postgres_db
        .select({
          id: schema.food_sources.id,
          position_x: schema.food_sources.position_x,
          position_y: schema.food_sources.position_y,
          food_type: schema.food_sources.food_type,
          amount: schema.food_sources.amount
        })
        .from(schema.food_sources)
        .limit(100)

      return {
        simulation,
        ants,
        colonies,
        foodSources
      }
    } catch (error) {
      console.error('Database error:', error)
      return { simulation: null, ants: [], colonies: [], foodSources: [] }
    }
  })

const SimulationField = ({ 
  simulation, 
  ants, 
  colonies, 
  foodSources 
}: { 
  simulation: Simulation | null
  ants: Ant[]
  colonies: Colony[]
  foodSources: FoodSource[]
}) => {
  if (!simulation) {
    return (
      <div className="flex items-center justify-center h-96 bg-gray-100 rounded-lg">
        <p className="text-gray-500">No active simulation found</p>
      </div>
    )
  }

  const gridSize = 20 // Size of each grid square in pixels
  const fieldWidth = Math.min(simulation.world_width, 800) // Limit display width
  const fieldHeight = Math.min(simulation.world_height, 600) // Limit display height
  const gridCols = Math.floor(fieldWidth / gridSize)
  const gridRows = Math.floor(fieldHeight / gridSize)

  return (
    <div className="relative border border-gray-300 rounded-lg overflow-hidden" style={{ width: fieldWidth, height: fieldHeight }}>
      {/* Grid background */}
      <svg width={fieldWidth} height={fieldHeight} className="absolute inset-0 bg-white" aria-label="Simulation field with grid">
        <title>Ant Simulation Field</title>
        {/* Vertical grid lines */}
        {Array.from({ length: gridCols + 1 }, (_, i) => (
          <line
            key={`vertical-line-${i}-${gridCols}`}
            x1={i * gridSize}
            y1={0}
            x2={i * gridSize}
            y2={fieldHeight}
            stroke="#e5e7eb"
            strokeWidth={0.5}
          />
        ))}
        {/* Horizontal grid lines */}
        {Array.from({ length: gridRows + 1 }, (_, i) => (
          <line
            key={`horizontal-line-${i}-${gridRows}`}
            x1={0}
            y1={i * gridSize}
            x2={fieldWidth}
            y2={i * gridSize}
            stroke="#e5e7eb"
            strokeWidth={0.5}
          />
        ))}
        
        {/* Food sources */}
        {foodSources.map((food) => (
          <circle
            key={food.id}
            cx={Math.min(Number(food.position_x), fieldWidth)}
            cy={Math.min(Number(food.position_y), fieldHeight)}
            r={Math.max(3, Math.min(Number(food.amount) / 10, 10))}
            fill="#10b981"
            opacity={0.7}
          />
        ))}
        
        {/* Colonies */}
        {colonies.map((colony) => (
          <g key={colony.id}>
            {/* Colony territory circle */}
            <circle
              cx={Math.min(Number(colony.center_x), fieldWidth)}
              cy={Math.min(Number(colony.center_y), fieldHeight)}
              r={Math.min(Number(colony.radius), 50)}
              fill={`hsl(${colony.color_hue}, 50%, 80%)`}
              opacity={0.3}
            />
            {/* Colony center */}
            <circle
              cx={Math.min(Number(colony.center_x), fieldWidth)}
              cy={Math.min(Number(colony.center_y), fieldHeight)}
              r={5}
              fill={`hsl(${colony.color_hue}, 70%, 50%)`}
            />
          </g>
        ))}
        
        {/* Ants */}
        {ants.map((ant) => (
          <circle
            key={ant.id}
            cx={Math.min(Number(ant.position_x), fieldWidth)}
            cy={Math.min(Number(ant.position_y), fieldHeight)}
            r={2}
            fill="#8b4513"
            className={ant.state === 'carrying_food' ? 'animate-pulse' : ''}
          />
        ))}
      </svg>
    </div>
  )
}

const HomePage = () => {
  const {
    data,
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ['simulation-data'],
    queryFn: () => getSimulationData(),
    // Refresh simulation data every 5 seconds
    refetchInterval: 5000,
  })

  return (
    <div className="flex-1 space-y-4 p-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Ant Colony Simulation</h2>
          {data?.simulation && (
            <p className="text-sm text-muted-foreground">
              {data.simulation.name} - Tick: {data.simulation.current_tick || 0} | 
              Ants: {data.ants.length} | 
              Colonies: {data.colonies.length} | 
              Food Sources: {data.foodSources.length}
            </p>
          )}
        </div>
        <button 
          type="button"
          onClick={() => refetch()} 
          className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
        >
          Refresh
        </button>
      </div>
      
      <div className="space-y-2">
        <h3 className="text-lg font-semibold">Simulation Field</h3>
        <p className="text-sm text-muted-foreground">
          White grid shows coordinates. Brown dots are ants, colored circles are colonies, green circles are food sources.
        </p>
      </div>
      
      {isLoading ? (
        <div className="flex items-center justify-center h-96 bg-gray-100 rounded-lg">
          <p className="text-gray-500">Loading simulation...</p>
        </div>
      ) : (
        <SimulationField 
          simulation={data?.simulation || null}
          ants={data?.ants || []}
          colonies={data?.colonies || []}
          foodSources={data?.foodSources || []}
        />
      )}
      
      {data?.simulation && (
        <div className="grid grid-cols-3 gap-4 text-sm">
          <div className="bg-gray-50 p-3 rounded">
            <h4 className="font-medium">World Size</h4>
            <p>{data.simulation.world_width} × {data.simulation.world_height}</p>
          </div>
          <div className="bg-gray-50 p-3 rounded">
            <h4 className="font-medium">Simulation Status</h4>
            <p className={data.simulation.is_active ? "text-green-600" : "text-red-600"}>
              {data.simulation.is_active ? "Active" : "Inactive"}
            </p>
          </div>
          <div className="bg-gray-50 p-3 rounded">
            <h4 className="font-medium">Current Tick</h4>
            <p>{(data.simulation.current_tick || 0).toLocaleString()}</p>
          </div>
        </div>
      )}
    </div>
  )
}

export const Route = createFileRoute('/_authenticated/_app/home/')({
  component: HomePage,
})
