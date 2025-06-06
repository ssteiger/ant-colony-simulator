import { useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema, eq } from '@ant-colony-simulator/db-drizzle'
import { AddAntsButton } from './-components/add-ants-button'
import { CreateSimulationButton } from './-components/create-simulation-button'

interface Simulation {
  id: string
  name: string
  world_width: number
  world_height: number
  current_tick: number | null
  is_active: boolean | null
  season?: string | null
  weather_type?: string | null
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
  .validator((data: { simulationId: string }) => data)
  .handler(async ({ data, context }) => {
    const { simulationId } = data
    console.log('data', data)
    console.log('simulationId', simulationId)
    try {
      const simulations = await postgres_db
        .select()
        .from(schema.simulations)
        .where(eq(schema.simulations.id, simulationId))
        .limit(1)

      if (simulations.length === 0) {
        return { simulation: null, ants: [], colonies: [], foodSources: [] }
      }

      console.log('simulations', simulations)

      const simulation = simulations[0]

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
        .where(eq(schema.colonies.simulation_id, simulationId))
        .limit(100)

      const ants = await postgres_db
        .select({
          id: schema.ants.id,
          position_x: schema.ants.position_x,
          position_y: schema.ants.position_y,
          colony_id: schema.ants.colony_id,
          state: schema.ants.state
        })
        .from(schema.ants)
        .innerJoin(schema.colonies, eq(schema.ants.colony_id, schema.colonies.id))
        .where(eq(schema.colonies.simulation_id, simulationId))
        .limit(500)

      const foodSources = await postgres_db
        .select({
          id: schema.food_sources.id,
          position_x: schema.food_sources.position_x,
          position_y: schema.food_sources.position_y,
          food_type: schema.food_sources.food_type,
          amount: schema.food_sources.amount
        })
        .from(schema.food_sources)
        .where(eq(schema.food_sources.simulation_id, simulationId))
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

const SimulationPage = () => {
  const params = Route.useParams()
  console.log('params', params)
  const simulationId = params.id
  console.log('simulationId', simulationId)

  const {
    data,
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ['simulation-data', simulationId],
    queryFn: () => getSimulationData({ data: { simulationId } }),
    // Refresh simulation data every 5 seconds
    refetchInterval: 5000,
  })

  const hasSimulation = data?.simulation !== null && data?.simulation !== undefined

  return (
    <div className="flex-1 space-y-4 p-4">
      <div className="flex items-center justify-between">
        <div>
          {hasSimulation && data?.simulation && (
            <p className="text-sm text-muted-foreground">
              {data.simulation.name} - Tick: {data.simulation.current_tick || 0} | 
              Ants: {data.ants.length} | 
              Colonies: {data.colonies.length} | 
              Food Sources: {data.foodSources.length}
            </p>
          )}
          {!hasSimulation && (
            <p className="text-sm text-yellow-600">
              No active simulation found. Create one to get started!
            </p>
          )}
        </div>
        <div className="flex gap-2">
          <CreateSimulationButton />
          <button 
            type="button"
            onClick={() => refetch()} 
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
          >
            Refresh
          </button>
          {hasSimulation && <AddAntsButton />}
        </div>
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
      
      {hasSimulation && data?.simulation && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
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
          <div className="bg-gray-50 p-3 rounded">
            <h4 className="font-medium">Environment</h4>
            <p className="capitalize">{data.simulation.season} • {data.simulation.weather_type}</p>
          </div>
        </div>
      )}

      {hasSimulation && data && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Ant Activity</h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Wandering:</span>
                <span>{data.ants.filter(ant => ant.state === 'wandering').length}</span>
              </div>
              <div className="flex justify-between">
                <span>Seeking Food:</span>
                <span>{data.ants.filter(ant => ant.state === 'seeking_food').length}</span>
              </div>
              <div className="flex justify-between">
                <span>Carrying Food:</span>
                <span>{data.ants.filter(ant => ant.state === 'carrying_food').length}</span>
              </div>
              <div className="flex justify-between">
                <span>Other States:</span>
                <span>{data.ants.filter(ant => !['wandering', 'seeking_food', 'carrying_food'].includes(ant.state)).length}</span>
              </div>
            </div>
          </div>

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Colonies</h4>
            <div className="space-y-2 text-sm">
              {data.colonies.map((colony) => (
                <div key={colony.id} className="flex justify-between">
                  <span>{colony.name}:</span>
                  <span>{data.ants.filter(ant => ant.colony_id === colony.id).length} ants</span>
                </div>
              ))}
            </div>
          </div>

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Food Sources</h4>
            <div className="space-y-2 text-sm">
              {data.foodSources.map((food) => (
                <div key={food.id} className="flex justify-between">
                  <span className="capitalize">{food.food_type}:</span>
                  <span>{Number(food.amount).toFixed(1)}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export const Route = createFileRoute('/_authenticated/_app/simulation/$id')({
  component: SimulationPage,
})


