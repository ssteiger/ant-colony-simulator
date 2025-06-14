import { createFileRoute } from '@tanstack/react-router'
import { AddAntsButton } from './-components/add-ants-button'
import { CreateSimulationButton } from '../-components/create-simulation-button'
import { DeleteSimulationButton } from './-components/delete-simulation-button'
import { AddFoodSourcesButton } from './-components/add-food-sources'
import { ResetAntPositionsButton } from './-components/reset-ant-positions'
import { Button } from '~/lib/components/ui/button'
import { useSimulationWebSocket } from '~/lib/hooks/useSimulationWebSocket'
import type { Simulation } from '~/types/drizzle'
import React from 'react'

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
  resources: Record<string, number> | null;
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

// Connection status indicator component
const ConnectionStatus = ({ 
  connectionState, 
  error, 
  lastUpdateTime,
  onReconnect,
  simulationId,
  wsData
}: { 
  connectionState: string
  error: string | null
  lastUpdateTime: Date | null
  onReconnect: () => void
  simulationId: string
  wsData: {
    simulation_id: number | null
    current_tick: number
    ants: RenderAnt[]
    colonies: RenderColony[]
    foodSources: RenderFoodSource[]
    pheromoneTrails: RenderPheromoneTrail[]
  } | null
}) => {
  const getStatusColor = () => {
    switch (connectionState) {
      case 'connected': return 'text-green-600'
      case 'connecting': return 'text-yellow-600'
      case 'disconnected': return 'text-gray-600'
      case 'error': return 'text-red-600'
      default: return 'text-gray-600'
    }
  }

  const getStatusIcon = () => {
    switch (connectionState) {
      case 'connected': return '🟢'
      case 'connecting': return '🟡'
      case 'disconnected': return '⚪'
      case 'error': return '🔴'
      default: return '⚪'
    }
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-sm">
        <span className={getStatusColor()}>
          {getStatusIcon()} {connectionState.charAt(0).toUpperCase() + connectionState.slice(1)}
        </span>
        {lastUpdateTime && (
          <span className="text-gray-500">
            Last update: {lastUpdateTime.toLocaleTimeString()}
          </span>
        )}
        {error && (
          <div className="flex items-center gap-2">
            <span className="text-red-600 text-xs">{error}</span>
            <Button onClick={onReconnect} variant="outline" size="sm">
              Retry
            </Button>
          </div>
        )}
      </div>
      
      {/* Debug information */}
      {connectionState === 'connecting' && (
        <div className="text-xs text-gray-500 bg-yellow-50 p-2 rounded border">
          <p className="font-medium">Debug Info:</p>
          <p>If stuck here for more than 10 seconds, check browser console for WebSocket logs.</p>
          <p>Server shows connection but client state is stuck - this suggests the onopen event isn't firing.</p>
        </div>
      )}
      
      {connectionState === 'connected' && !wsData && (
        <div className="text-xs text-gray-500 bg-blue-50 p-2 rounded border">
          <p className="font-medium">WebSocket Connected - Waiting for Data:</p>
          <p>• Simulation ID: {simulationId}</p>
          <p>• Connection: Established</p>
          <p>• Subscription: Sent</p>
          <p>• Status: Waiting for server response...</p>
          <p className="text-orange-600">If no data appears within 10 seconds, check:</p>
          <p className="ml-2">- Does simulation {simulationId} exist?</p>
          <p className="ml-2">- Is the simulation server running?</p>
          <p className="ml-2">- Check browser console for detailed logs</p>
        </div>
      )}
    </div>
  )
}

const SimulationField = ({ 
  simulation, 
  ants, 
  colonies, 
  foodSources,
  pheromoneTrails,
  onFieldSizeChange,
  onMouseMove,
  onMouseLeave
}: { 
  simulation: Simulation | null
  ants: RenderAnt[]
  colonies: RenderColony[]
  foodSources: RenderFoodSource[]
  pheromoneTrails: RenderPheromoneTrail[]
  onFieldSizeChange: (width: number, height: number) => void
  onMouseMove: (e: React.MouseEvent<HTMLDivElement>) => void
  onMouseLeave: () => void
}) => {
  const gridSize = 4 // Size of each grid square in pixels
  const fieldWidth = simulation ? simulation.world_width : 0
  const fieldHeight = simulation ? simulation.world_height : 0
  const gridCols = Math.floor(fieldWidth / gridSize)
  const gridRows = Math.floor(fieldHeight / gridSize)

  // Notify parent of field size changes
  React.useEffect(() => {
    if (simulation) {
      onFieldSizeChange(fieldWidth, fieldHeight)
    }
  }, [fieldWidth, fieldHeight, onFieldSizeChange, simulation])

  if (!simulation) {
    return (
      <div className="flex items-center justify-center h-96 bg-gray-100 rounded-lg">
        <p className="text-gray-500">No active simulation found</p>
      </div>
    )
  }

  // Group pheromone trails by type for different rendering
  const trailsByType = pheromoneTrails.reduce((acc, trail) => {
    if (!acc[trail.trail_type]) {
      acc[trail.trail_type] = []
    }
    acc[trail.trail_type].push(trail)
    return acc
  }, {} as Record<string, RenderPheromoneTrail[]>)

  return (
    <div 
      className="relative border border-gray-300 rounded-lg overflow-hidden" 
      style={{ width: fieldWidth, height: fieldHeight }}
      onMouseMove={onMouseMove}
      onMouseLeave={onMouseLeave}
    >
      
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

              console.log({ trail_type: trail.trail_type })
              
              // Get color based on trail type
              let trailColor: string;
              switch (trail.trail_type) {
                case 'food':
                  trailColor = colony ? `hsl(${colony.color_hue}, 80%, 60%)` : '#059669'; // green for food trails
                  break;
                case 'danger':
                  trailColor = '#dc2626'; // red for danger
                  break;
                case 'home':
                  trailColor = colony ? `hsl(${colony.color_hue}, 70%, 40%)` : '#4b5563'; // darker colony color for home
                  break;
                case 'exploration':
                  trailColor = '#6366f1'; // indigo for exploration
                  break;
                default:
                  trailColor = '#000'; // gray for unknown types
              }
              
              // Calculate size and opacity based on strength
              const baseSize = 3; // Increased from 2
              const size = baseSize + (trail.strength / 50); // Size increases with strength
              const opacity = Math.max(0.2, Math.min(0.8, trail.strength / 100)); // Adjusted opacity range

              return (
                <circle
                  key={trail.id}
                  cx={trail.position_x}
                  cy={trail.position_y}
                  r={size}
                  fill={trailColor}
                  opacity={opacity}
                >
                  <title>{`${trail.trail_type.charAt(0).toUpperCase() + trail.trail_type.slice(1)} Trail | Strength: ${Number(trail.strength).toFixed(1)} | Position: (${trail.position_x.toFixed(2)}, ${trail.position_y.toFixed(2)}) | Colony: ${colony?.name || 'Unknown'}`}</title>
                </circle>
              );
            })}
          </g>
        ))}
        
        {/* Food sources */}
        {foodSources.map((food) => {
          return (
            <circle
              key={food.id}
              cx={food.position_x}
              cy={food.position_y}
              r={Math.max(3, Math.min(Number(food.amount) / 10, 10))}
              fill="#10b981"
              opacity={0.7}
            >
              <title>{`Food Source: ${food.food_type} | Amount: ${Number(food.amount).toFixed(1)} | Position: (${food.position_x.toFixed(2)}, ${food.position_y.toFixed(2)})`}</title>
            </circle>
          );
        })}
        
        {/* Colonies */}
        {colonies.map((colony) => {
          const colonyAnts = ants.filter(ant => ant.colony_id === colony.id);
          
          return (
            <g key={colony.id}>
              {/* Colony territory circle */}
              <circle
                cx={colony.center_x}
                cy={colony.center_y}
                r={Math.min(Number(colony.radius), 50)}
                fill={`hsl(${colony.color_hue}, 50%, 80%)`}
                opacity={0.3}
              >
                <title>{`${colony.name} Territory | Center: (${colony.center_x.toFixed(2)}, ${colony.center_y.toFixed(2)}) | Radius: ${colony.radius} | Ants: ${colonyAnts.length}`}</title>
              </circle>
              {/* Colony center */}
              <circle
                cx={colony.center_x}
                cy={colony.center_y}
                r={5}
                fill={`hsl(${colony.color_hue}, 70%, 50%)`}
              >
                <title>{`${colony.name} Center | Position: (${colony.center_x.toFixed(2)}, ${colony.center_y.toFixed(2)}) | Ants: ${colonyAnts.length}`}</title>
              </circle>
            </g>
          );
        })}
        
        {/* Ants */}
        {ants.map((ant) => (
          <g
            key={ant.id}
            transform={`translate(${ant.position_x}, ${ant.position_y}) rotate(${ant.angle * (180 / Math.PI)} 0 0)`}
          >
            <image
              href="/ant_sprite.png"
              x={-8} // Center the sprite (assuming 16x16 sprite)
              y={-8}
              width={16}
              height={16}
              style={{
                filter: `hue-rotate(${ant.ant_type.color_hue}deg)`,
                opacity: 0.9
              }}
            >
              <title>{`${ant.ant_type.name} | State: ${ant.state} | Position: (${ant.position_x.toFixed(2)}, ${ant.position_y.toFixed(2)}) | Angle: ${ant.angle.toFixed(1)}°`}</title>
            </image>
          </g>
        ))}
      </svg>
    </div>
  )
}

const SimulationPage = () => {
  const params = Route.useParams()
  const simulationId = params.id
  const [fieldSize, setFieldSize] = React.useState({ width: 0, height: 0 })
  const [mouseCoords, setMouseCoords] = React.useState<{ x: number; y: number } | null>(null)

  // Use WebSocket hook for real-time updates
  const { 
    data: wsData, 
    connectionState, 
    isLoading, 
    error, 
    connect,
    lastUpdateTime 
  } = useSimulationWebSocket(simulationId)

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top
    
    // Convert to simulation coordinates
    const simX = (x / fieldSize.width) * (wsData?.simulation_id ? 800 : 0)
    const simY = (y / fieldSize.height) * (wsData?.simulation_id ? 600 : 0)
    
    setMouseCoords({ x: simX, y: simY })
  }

  const handleMouseLeave = () => {
    setMouseCoords(null)
  }

  // Fallback to get simulation metadata if needed
  // This could be enhanced to fetch initial simulation data when WebSocket is not available
  // For now, we'll rely on the WebSocket FullState message

  const hasSimulation = wsData !== null

  return (
    <div className="flex-1 space-y-4 p-4">
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          {hasSimulation && wsData && (
            <p className="text-sm text-muted-foreground">
              Simulation {wsData.simulation_id} - Tick: {wsData.current_tick} | 
              Ants: {wsData.ants.length} | 
              Colonies: {wsData.colonies.length} | 
              Food Sources: {wsData.foodSources.length} |
              Pheromone Trails: {wsData.pheromoneTrails.length}
            </p>
          )}
          {!hasSimulation && !isLoading && (
            <p className="text-sm text-yellow-600">
              No active simulation found. Create one to get started!
            </p>
          )}
          <ConnectionStatus 
            connectionState={connectionState}
            error={error}
            lastUpdateTime={lastUpdateTime}
            onReconnect={connect}
            simulationId={simulationId}
            wsData={wsData}
          />
        </div>
        <div className="flex gap-2">
          <CreateSimulationButton />
          <DeleteSimulationButton />
          {hasSimulation && <AddAntsButton />}
          {hasSimulation && <AddFoodSourcesButton />}
          {hasSimulation && <ResetAntPositionsButton />}
        </div>
      </div>
      
      <div className="space-y-2">
        <h3 className="text-lg font-semibold">Simulation Field ({fieldSize.width}x{fieldSize.height})</h3>
        {mouseCoords && (
          <div className="fixed bottom-14 right-4 bg-black/70 text-white px-2 py-1 rounded text-xs z-50">
            ({mouseCoords.x.toFixed(1)}, {mouseCoords.y.toFixed(1)})
          </div>
        )}
        <p className="text-sm text-muted-foreground">
          Real-time WebSocket updates. Colored dots are ants, colored circles are colonies, green circles are food sources, small colored dots show food trails.
        </p>
      </div>

      {hasSimulation && wsData && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div className="bg-gray-50 p-3 rounded">
            <h4 className="font-medium">Simulation ID</h4>
            <p>{wsData.simulation_id}</p>
          </div>
          <div className="bg-gray-50 p-3 rounded">
            <h4 className="font-medium">Connection Status</h4>
            <p className={connectionState === 'connected' ? "text-green-600" : "text-red-600"}>
              {connectionState === 'connected' ? "Live Updates" : "Disconnected"}
            </p>
          </div>
          <div className="bg-gray-50 p-3 rounded">
            <h4 className="font-medium">Current Tick</h4>
            <p>{wsData.current_tick.toLocaleString()}</p>
          </div>
          <div className="bg-gray-50 p-3 rounded">
            <h4 className="font-medium">Update Rate</h4>
            <p>Real-time (500ms)</p>
          </div>
        </div>
      )}
      
      {isLoading ? (
        <div className="flex items-center justify-center h-96 bg-gray-100 rounded-lg">
          <div className="text-center">
            <p className="text-gray-500 mb-2">Connecting to simulation...</p>
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto" />
          </div>
        </div>
      ) : (
        <SimulationField 
          simulation={hasSimulation ? { world_width: 800, world_height: 600 } as Simulation : null}
          ants={wsData?.ants || []}
          colonies={wsData?.colonies || []}
          foodSources={wsData?.foodSources || []}
          pheromoneTrails={wsData?.pheromoneTrails || []}
          onFieldSizeChange={(width, height) => setFieldSize({ width, height })}
          onMouseMove={handleMouseMove}
          onMouseLeave={handleMouseLeave}
        />
      )}

      {hasSimulation && wsData && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Ant Positions</h4>
            <div className="space-y-2 text-sm max-h-60 overflow-y-auto">
              {wsData?.ants.map((ant) => (
                <div key={ant.id} className="flex justify-between items-center border-b pb-1">
                  <div className="flex items-center gap-2">
                    <div 
                      className="w-2 h-2 rounded-full" 
                      style={{ backgroundColor: `hsl(${ant.ant_type.color_hue}, 70%, 50%)` }}
                    />
                    <span className="font-medium">ant id {ant.id} - {ant.ant_type.name}</span>
                    <span className="text-xs text-gray-500">({ant.state})</span>
                  </div>
                  <div className="text-xs text-gray-600">
                    ({ant.position_x.toFixed(1)}, {ant.position_y.toFixed(1)}) angle: {ant.angle.toFixed(1)}
                  </div>
                </div>
              ))}
              {wsData?.ants.length === 0 && (
                <p className="text-gray-500 text-xs">No ants in simulation</p>
              )}
            </div>
          </div>

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Ant Activity</h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>wandering:</span>
                <span>{wsData.ants.filter(ant => ant.state === 'wandering').length}</span>
              </div>
              <div className="flex justify-between">
                <span>seeking_food:</span>
                <span>{wsData.ants.filter(ant => ant.state === 'seeking_food').length}</span>
              </div>
              <div className="flex justify-between">
                <span>carrying_food:</span>
                <span>{wsData.ants.filter(ant => ant.state === 'carrying_food').length}</span>
              </div>
              <div className="flex justify-between">
                <span>following:</span>
                <span>{wsData.ants.filter(ant => ant.state === 'following').length}</span>
              </div>
              <div className="flex justify-between">
                <span>exploring:</span>
                <span>{wsData.ants.filter(ant => ant.state === 'exploring').length}</span>
              </div>
              <div className="flex justify-between">
                <span>patrolling:</span>
                <span>{wsData.ants.filter(ant => ant.state === 'patrolling').length}</span>
              </div>
              <div className="flex justify-between">
                <span>dead:</span>
                <span>{wsData.ants.filter(ant => ant.state === 'dead').length}</span>
              </div>
              <div className="flex justify-between">
                <span>other states:</span>
                <span>{wsData.ants.filter(ant => !['wandering', 'seeking_food', 'carrying_food', 'following', 'exploring', 'patrolling', 'dead'].includes(ant.state)).length}</span>
              </div>
            </div>
          </div>

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Pheromone Trails</h4>
            <div className="space-y-2 text-sm">
              {Object.entries(wsData.pheromoneTrails.reduce((acc, trail) => {
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
              {wsData.pheromoneTrails.length === 0 && (
                <p className="text-gray-500 text-xs">No active trails</p>
              )}
            </div>
          </div>

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Ants</h4>
            <div className="space-y-3 text-sm">
              {Object.entries(wsData.ants.reduce((acc, ant) => {
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
              {wsData.colonies.map((colony) => (
                <div key={colony.id} className="flex justify-between">
                  <span>{colony.name}:</span>
                  <span>{wsData.ants.filter(ant => ant.colony_id === colony.id).length} ants</span>
                </div>
              ))}
            </div>
          </div>

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Colony Resources</h4>
            <div className="space-y-3 text-sm">
              {wsData.colonies.map((colony) => {
                const resources = (colony.resources as Record<string, number>) || {};
                const totalResources = Object.values(resources).reduce((sum, value) => sum + (Number(value) || 0), 0);
                return (
                  <div key={colony.id} className="border-l-4 pl-3" style={{ borderColor: `hsl(${colony.color_hue}, 60%, 50%)` }}>
                    <div className="flex justify-between items-center mb-2">
                      <span className="font-medium">{colony.name}</span>
                      <span className="text-gray-600">Total: {totalResources.toFixed(1)}</span>
                    </div>
                    <div className="grid grid-cols-3 gap-2 text-xs">
                      <div className="flex justify-between">
                        <span className="text-amber-600">🌾 Seeds:</span>
                        <span>{Number(resources.seeds || 0).toFixed(1)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-blue-600">🍯 Sugar:</span>
                        <span>{Number(resources.sugar || 0).toFixed(1)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-red-600">🥩 Protein:</span>
                        <span>{Number(resources.protein || 0).toFixed(1)}</span>
                      </div>
                    </div>
                  </div>
                );
              })}
              {wsData.colonies.length === 0 && (
                <p className="text-gray-500 text-xs">No colonies found</p>
              )}
            </div>
          </div>

          <div className="bg-white border rounded-lg p-4">
            <h4 className="font-semibold mb-2">Food Sources</h4>
            <div className="space-y-2 text-sm">
              {wsData.foodSources.map((food) => (
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


