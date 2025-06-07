import { useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema, eq, and, gt } from '@ant-colony-simulator/db-drizzle'
import { AddAntsButton } from './-components/add-ants-button'
import { CreateSimulationButton } from '../-components/create-simulation-button'
import { DeleteSimulationButton } from './-components/delete-simulation-button'
import { AddFoodSourcesButton } from './-components/add-food-sources'
import { ResetAntPositionsButton } from './-components/reset-ant-positions'
import { Button } from '~/lib/components/ui/button'
import type { Simulation } from '~/types/drizzle'

// Define minimal types for the rendered data
type RenderAnt = {
  id: string;
  position_x: number;
  position_y: number;
  angle: number;
  colony_id: string;
  state: string;
  ant_type: {
    id: number;
    name: string;
    role: string;
    color_hue: number;
    base_speed: number;
    base_strength: number;
    base_health: number;
    carrying_capacity: number;
  };
}

type RenderColony = {
  id: string;
  name: string;
  center_x: number;
  center_y: number;
  radius: number;
  color_hue: number;
}

type RenderFoodSource = {
  id: string;
  position_x: number;
  position_y: number;
  food_type: string;
  amount: number;
}

type RenderPheromoneTrail = {
  id: string;
  colony_id: string;
  trail_type: string;
  position_x: number;
  position_y: number;
  strength: number;
}

const getSimulationData = createServerFn({ method: 'GET' })
  .validator((data: { simulationId: string }) => data)
  .handler(async ({ data }) => {
    const { simulationId } = data
    console.log('simulationId', simulationId)
    try {
      const simulations = await postgres_db
        .select()
        .from(schema.simulations)
        .where(eq(schema.simulations.id, simulationId))
        .limit(1)

      if (simulations.length === 0) {
        return { simulation: null, ants: [], colonies: [], foodSources: [], pheromoneTrails: [] }
      }

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
          angle: schema.ants.angle,
          colony_id: schema.ants.colony_id,
          state: schema.ants.state,
          ant_type: {
            id: schema.ant_types.id,
            name: schema.ant_types.name,
            role: schema.ant_types.role,
            color_hue: schema.ant_types.color_hue,
            base_speed: schema.ant_types.base_speed,
            base_strength: schema.ant_types.base_strength,
            base_health: schema.ant_types.base_health,
            carrying_capacity: schema.ant_types.carrying_capacity,
          }
        })
        .from(schema.ants)
        .innerJoin(schema.colonies, eq(schema.ants.colony_id, schema.colonies.id))
        .innerJoin(schema.ant_types, eq(schema.ants.ant_type_id, schema.ant_types.id))
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
        .where(and(
          eq(schema.food_sources.simulation_id, simulationId),
          gt(schema.food_sources.amount, 0)
        ))
        .limit(100)

      // Fetch pheromone trails for colonies in this simulation
      const pheromoneTrails = await postgres_db
        .select({
          id: schema.pheromone_trails.id,
          colony_id: schema.pheromone_trails.colony_id,
          trail_type: schema.pheromone_trails.trail_type,
          position_x: schema.pheromone_trails.position_x,
          position_y: schema.pheromone_trails.position_y,
          strength: schema.pheromone_trails.strength
        })
        .from(schema.pheromone_trails)
        .innerJoin(schema.colonies, eq(schema.pheromone_trails.colony_id, schema.colonies.id))
        .where(and(
          eq(schema.colonies.simulation_id, simulationId),
          gt(schema.pheromone_trails.strength, 0) // Only show trails with meaningful strength
        ))
        .limit(1000)

      return {
        simulation,
        ants,
        colonies,
        foodSources,
        pheromoneTrails
      }
    } catch (error) {
      console.error('Database error:', error)
      return { simulation: null, ants: [], colonies: [], foodSources: [], pheromoneTrails: [] }
    }
  })

const SimulationField = ({ 
  simulation, 
  ants, 
  colonies, 
  foodSources,
  pheromoneTrails
}: { 
  simulation: Simulation | null
  ants: RenderAnt[]
  colonies: RenderColony[]
  foodSources: RenderFoodSource[]
  pheromoneTrails: RenderPheromoneTrail[]
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

  // Group pheromone trails by type for different rendering
  const trailsByType = pheromoneTrails.reduce((acc, trail) => {
    if (!acc[trail.trail_type]) {
      acc[trail.trail_type] = []
    }
    acc[trail.trail_type].push(trail)
    return acc
  }, {} as Record<string, RenderPheromoneTrail[]>)

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

        {/* Pheromone trails - render behind other elements */}
        {Object.entries(trailsByType).map(([trailType, trails]) => (
          <g key={`trails-${trailType}`}>
            {trails.map((trail) => {
              const colony = colonies.find(c => c.id === trail.colony_id);
              const opacity = Math.max(0.1, Math.min(0.8, trail.strength));
              
              // Different colors for different trail types
              let trailColor = '#9333ea'; // purple for default
              if (trailType === 'food') trailColor = '#059669'; // green
              if (trailType === 'danger') trailColor = '#dc2626'; // red
              if (trailType === 'territory') trailColor = '#2563eb'; // blue
              if (trailType === 'recruitment') trailColor = '#ea580c'; // orange
              
              // Use colony color if available, otherwise use trail type color
              const finalColor = colony 
                ? `hsl(${colony.color_hue}, 70%, 50%)`
                : trailColor;

              return (
                <circle
                  key={trail.id}
                  cx={Math.min(Number(trail.position_x), fieldWidth)}
                  cy={Math.min(Number(trail.position_y), fieldHeight)}
                  r={Math.max(1, trail.strength * 3)}
                  fill={finalColor}
                  opacity={opacity}
                  className="animate-pulse"
                >
                  <title>{`${trailType.charAt(0).toUpperCase() + trailType.slice(1)} Trail | Strength: ${Number(trail.strength).toFixed(2)} | Position: (${trail.position_x}, ${trail.position_y}) | Colony: ${colony?.name || 'Unknown'}`}</title>
                </circle>
              );
            })}
          </g>
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
          >
            <title>{`Food Source: ${food.food_type} | Amount: ${Number(food.amount).toFixed(1)} | Position: (${food.position_x}, ${food.position_y})`}</title>
          </circle>
        ))}
        
        {/* Colonies */}
        {colonies.map((colony) => {
          const colonyAnts = ants.filter(ant => ant.colony_id === colony.id);
          return (
            <g key={colony.id}>
              {/* Colony territory circle */}
              <circle
                cx={Math.min(Number(colony.center_x), fieldWidth)}
                cy={Math.min(Number(colony.center_y), fieldHeight)}
                r={Math.min(Number(colony.radius), 50)}
                fill={`hsl(${colony.color_hue}, 50%, 80%)`}
                opacity={0.3}
              >
                <title>{`${colony.name} Territory | Center: (${colony.center_x}, ${colony.center_y}) | Radius: ${colony.radius} | Ants: ${colonyAnts.length}`}</title>
              </circle>
              {/* Colony center */}
              <circle
                cx={Math.min(Number(colony.center_x), fieldWidth)}
                cy={Math.min(Number(colony.center_y), fieldHeight)}
                r={5}
                fill={`hsl(${colony.color_hue}, 70%, 50%)`}
              >
                <title>{`${colony.name} Center | Position: (${colony.center_x}, ${colony.center_y}) | Ants: ${colonyAnts.length}`}</title>
              </circle>
            </g>
          );
        })}
        
        {/* Ants */}
        {ants.map((ant) => {
          const colony = colonies.find(c => c.id === ant.colony_id);
          // Use ant type color hue if available, otherwise fall back to brown
          const antColor = ant.state === 'carrying_food' 
            ? `hsl(${ant.ant_type.color_hue}, 70%, 60%)` 
            : `hsl(${ant.ant_type.color_hue}, 60%, 40%)`;
          
          const imageSize = 8; // Size of the ant image
          
          return (
            <g key={ant.id}>
              {/* Colored background circle for ant type identification */}
              <circle
                cx={Math.min(Number(ant.position_x), fieldWidth)}
                cy={Math.min(Number(ant.position_y), fieldHeight)}
                r={imageSize / 2 + 1}
                fill={antColor}
                opacity={0.3}
                className={ant.state === 'carrying_food' ? 'animate-pulse' : ''}
              />
              {/* Ant image */}
              <image
                href="/ant_sprite.png"
                x={Math.min(Number(ant.position_x), fieldWidth) - imageSize / 2}
                y={Math.min(Number(ant.position_y), fieldHeight) - imageSize / 2}
                width={imageSize}
                height={imageSize}
                transform={`rotate(${ant.angle} ${Math.min(Number(ant.position_x), fieldWidth)} ${Math.min(Number(ant.position_y), fieldHeight)})`}
                className={ant.state === 'carrying_food' ? 'animate-pulse' : ''}
              >
                <title>{`${ant.ant_type.name} (${ant.ant_type.role}) | State: ${ant.state.replace('_', ' ')} | Position: (${ant.position_x}, ${ant.position_y}) | Colony: ${colony?.name || 'Unknown'} | Speed: ${ant.ant_type.base_speed} | Strength: ${ant.ant_type.base_strength} | Health: ${ant.ant_type.base_health}`}</title>
              </image>
            </g>
          );
        })}
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
    // Refresh simulation data every 0.05 seconds
    refetchInterval: 50,
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
              Food Sources: {data.foodSources.length} |
              Pheromone Trails: {data.pheromoneTrails.length}
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
          <DeleteSimulationButton />
          <Button 
            onClick={() => refetch()} 
            variant="outline"
            size="sm"
          >
            Refresh
          </Button>
          {hasSimulation && <AddAntsButton />}
          {hasSimulation && <AddFoodSourcesButton />}
          {hasSimulation && <ResetAntPositionsButton />}
        </div>
      </div>
      
      <div className="space-y-2">
        <h3 className="text-lg font-semibold">Simulation Field</h3>
        <p className="text-sm text-muted-foreground">
          White grid shows coordinates. Brown dots are ants (purple when carrying food), colored circles are colonies, green circles are food sources, colored dots are pheromone trails.
        </p>
      </div>

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
          pheromoneTrails={data?.pheromoneTrails || []}
        />
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
            <h4 className="font-semibold mb-2">Pheromone Trails</h4>
            <div className="space-y-2 text-sm">
              {Object.entries(data.pheromoneTrails.reduce((acc, trail) => {
                if (!acc[trail.trail_type]) {
                  acc[trail.trail_type] = 0
                }
                acc[trail.trail_type]++
                return acc
              }, {} as Record<string, number>)).map(([trailType, count]) => (
                <div key={trailType} className="flex justify-between">
                  <span className="capitalize">{trailType}:</span>
                  <span>{count}</span>
                </div>
              ))}
              {data.pheromoneTrails.length === 0 && (
                <p className="text-gray-500 text-xs">No active trails</p>
              )}
            </div>
          </div>

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Ants</h4>
            <div className="space-y-3 text-sm">
              {Object.entries(data.ants.reduce((acc, ant) => {
                const typeName = ant.ant_type.name;
                if (!acc[typeName]) {
                  acc[typeName] = {
                    count: 0,
                    antType: ant.ant_type
                  };
                }
                acc[typeName].count++;
                return acc;
              }, {} as Record<string, { count: number; antType: RenderAnt['ant_type'] }>)).map(([typeName, { count, antType }]) => (
                <div key={typeName} className="border-l-4 pl-3" style={{ borderColor: `hsl(${antType.color_hue}, 60%, 50%)` }}>
                  <div className="flex justify-between items-center">
                    <span className="font-medium">{typeName}</span>
                    <span className="text-gray-600">{count} ants</span>
                  </div>
                  <div className="text-xs text-gray-500 mt-1">
                    {antType.role} • Speed: {Number(antType.base_speed).toFixed(1)} • Strength: {Number(antType.base_strength).toFixed(1)} • Capacity: {Number(antType.carrying_capacity).toFixed(1)}
                  </div>
                </div>
              ))}
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


