import { useState, useEffect, useRef, useCallback } from 'react'

// Backend data types from Rust
interface BackendAnt {
  id: number
  position: [number, number]
  angle: number
  colony_id: number
  ant_type_id: number
  state: string
  speed: number
  health: number
  energy: number
}

interface BackendColony {
  id: number
  center: [number, number]
  radius: number
  population: number
  resources: Record<string, number>
}

interface BackendFoodSource {
  id: number
  position: [number, number]
  food_type: string
  amount: number
}

interface BackendPheromoneTrail {
  id: number
  colony_id: number
  trail_type: string
  position: [number, number]
  strength: number
}

// Types matching the Rust backend WebSocket messages
type WebSocketMessage = 
  | {
      type: 'FullState'
      simulation_id: number
      tick: number
      ants: BackendAnt[]
      colonies: BackendColony[]
      food_sources: BackendFoodSource[]
      pheromone_trails: BackendPheromoneTrail[]
    }
  | {
      type: 'DeltaUpdate'
      simulation_id: number
      tick: number
      updated_ants: BackendAnt[]
      updated_colonies: BackendColony[]
      updated_food_sources: BackendFoodSource[]
      new_pheromone_trails: BackendPheromoneTrail[]
      removed_ant_ids: number[]
      removed_food_source_ids: number[]
    }
  | {
      type: 'SimulationStatus'
      simulation_id: number
      is_running: boolean
      current_tick: number
    }
  | {
      type: 'Error'
      message: string
    }

// Frontend types (matching the existing types in the component)
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

type SimulationData = {
  simulation_id: number | null
  current_tick: number
  ants: RenderAnt[]
  colonies: RenderColony[]
  foodSources: RenderFoodSource[]
  pheromoneTrails: RenderPheromoneTrail[]
}

type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error'

interface UseSimulationWebSocketReturn {
  data: SimulationData | null
  connectionState: ConnectionState
  isLoading: boolean
  error: string | null
  connect: () => void
  disconnect: () => void
  lastUpdateTime: Date | null
}

export function useSimulationWebSocket(
  simulationId: string,
  wsUrl = 'ws://127.0.0.1:8080/ws'
): UseSimulationWebSocketReturn {
  const [data, setData] = useState<SimulationData | null>(null)
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected')
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [lastUpdateTime, setLastUpdateTime] = useState<Date | null>(null)

  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const reconnectAttemptsRef = useRef(0)
  const messageTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const maxReconnectAttempts = 10
  const reconnectDelayBase = 1000 // Start with 1 second

  // Convert backend data to frontend format
  const convertBackendAnt = useCallback((ant: BackendAnt): RenderAnt => ({
    id: ant.id.toString(),
    position_x: ant.position[0],
    position_y: ant.position[1],
    angle: ant.angle,
    colony_id: ant.colony_id.toString(),
    state: ant.state.toLowerCase(),
    ant_type: {
      id: ant.ant_type_id,
      name: 'Worker', // Default, could be enhanced with type lookup
      role: 'worker',
      color_hue: 200, // Default, could be enhanced with type lookup
      base_speed: ant.speed,
      base_strength: 1.0,
      base_health: ant.health,
      carrying_capacity: 1.0,
    }
  }), [])

  const convertBackendColony = useCallback((colony: BackendColony): RenderColony => ({
    id: colony.id.toString(),
    name: `Colony ${colony.id}`,
    center_x: colony.center[0],
    center_y: colony.center[1],
    radius: colony.radius,
    color_hue: (colony.id * 137) % 360, // Generate unique hue based on ID
    resources: colony.resources || null,
  }), [])

  const convertBackendFoodSource = useCallback((food: BackendFoodSource): RenderFoodSource => ({
    id: food.id.toString(),
    position_x: food.position[0],
    position_y: food.position[1],
    food_type: food.food_type,
    amount: food.amount,
  }), [])

  const convertBackendPheromoneTrail = useCallback((trail: BackendPheromoneTrail): RenderPheromoneTrail => ({
    id: trail.id.toString(),
    colony_id: trail.colony_id.toString(),
    trail_type: trail.trail_type,
    position_x: trail.position[0],
    position_y: trail.position[1],
    strength: trail.strength,
  }), [])

  // Handle incoming WebSocket messages
  const handleMessage = useCallback((event: MessageEvent) => {
    //console.log('📨 Raw WebSocket message received:', event.data)
    
    // Clear the message timeout since we received a response
    if (messageTimeoutRef.current) {
      clearTimeout(messageTimeoutRef.current)
      messageTimeoutRef.current = null
    }
    
    try {
      const message: WebSocketMessage = JSON.parse(event.data)
      console.log('📨 Parsed WebSocket message:', message)
      
      switch (message.type) {
        case 'FullState':
          {
            console.log('🎯 Processing FullState update', { 
              tick: message.tick, 
              antsCount: message.ants.length,
              coloniesCount: message.colonies.length,
              foodSourcesCount: message.food_sources.length,
              pheromoneTrailsCount: message.pheromone_trails.length
            })
            
            const newData = {
              simulation_id: message.simulation_id,
              current_tick: message.tick,
              ants: message.ants.map(convertBackendAnt),
              colonies: message.colonies.map(convertBackendColony),
              foodSources: message.food_sources.map(convertBackendFoodSource),
              pheromoneTrails: message.pheromone_trails.map(convertBackendPheromoneTrail),
            }
            
            setData(newData)
            setIsLoading(false)
            setError(null)
            setLastUpdateTime(new Date())
            console.log('✅ FullState processed successfully, isLoading set to false')
            break
          }

        case 'DeltaUpdate':
          {
            console.log('🔄 Processing DeltaUpdate', { 
              tick: message.tick, 
              updatedAnts: message.updated_ants.length,
              updatedColonies: message.updated_colonies.length,
              updatedFoodSources: message.updated_food_sources.length,
              newPheromoneTrails: message.new_pheromone_trails.length,
              removedAnts: message.removed_ant_ids.length,
              removedFoodSources: message.removed_food_source_ids.length
            })
            
            setData(prevData => {
              if (!prevData) {
                console.log('🔧 DEBUG: prevData is null, cannot apply delta update')
                console.log('🔧 DEBUG: Requesting FullState from server...')
                
                // Request FullState when we receive DeltaUpdate but have no baseline data
                if (wsRef.current?.readyState === WebSocket.OPEN) {
                  const requestFullState = {
                    type: 'RequestFullState',
                    simulation_id: message.simulation_id
                  }
                  try {
                    wsRef.current.send(JSON.stringify(requestFullState))
                    console.log('🔧 DEBUG: Sent RequestFullState message')
                  } catch (error) {
                    console.error('🔧 DEBUG: Failed to send RequestFullState:', error)
                  }
                }
                return prevData
              }
              
              // Apply delta updates
              let updatedAnts = [...prevData.ants]
              const updatedColonies = [...prevData.colonies]
              let updatedFoodSources = [...prevData.foodSources]
              let updatedPheromoneTrails = [...prevData.pheromoneTrails]

              // Update existing ants or add new ones
              for (const backendAnt of message.updated_ants) {
                const convertedAnt = convertBackendAnt(backendAnt)
                const existingIndex = updatedAnts.findIndex(ant => ant.id === convertedAnt.id)
                if (existingIndex >= 0) {
                  updatedAnts[existingIndex] = convertedAnt
                } else {
                  updatedAnts.push(convertedAnt)
                }
              }

              // Remove deleted ants
              for (const removedId of message.removed_ant_ids) {
                updatedAnts = updatedAnts.filter(ant => ant.id !== removedId.toString())
              }

              // Update existing colonies or add new ones
              for (const backendColony of message.updated_colonies) {
                const convertedColony = convertBackendColony(backendColony)
                const existingIndex = updatedColonies.findIndex(colony => colony.id === convertedColony.id)
                if (existingIndex >= 0) {
                  updatedColonies[existingIndex] = convertedColony
                } else {
                  updatedColonies.push(convertedColony)
                }
              }

              // Update existing food sources or add new ones
              for (const backendFood of message.updated_food_sources) {
                const convertedFood = convertBackendFoodSource(backendFood)
                const existingIndex = updatedFoodSources.findIndex(food => food.id === convertedFood.id)
                if (existingIndex >= 0) {
                  updatedFoodSources[existingIndex] = convertedFood
                } else {
                  updatedFoodSources.push(convertedFood)
                }
              }

              // Remove deleted food sources
              for (const removedId of message.removed_food_source_ids) {
                updatedFoodSources = updatedFoodSources.filter(food => food.id !== removedId.toString())
              }

              // Add new pheromone trails (replace all for simplicity)
              updatedPheromoneTrails = message.new_pheromone_trails.map(convertBackendPheromoneTrail)

              const deltaResult = {
                ...prevData,
                current_tick: message.tick,
                ants: updatedAnts,
                colonies: updatedColonies,
                foodSources: updatedFoodSources,
                pheromoneTrails: updatedPheromoneTrails,
              }
              
              return deltaResult
            })
            setLastUpdateTime(new Date())
            setIsLoading(false) // Connection is working, we're receiving data
            console.log('✅ DeltaUpdate processed successfully')
            break
          }

        case 'SimulationStatus':
          console.log('📊 Processing SimulationStatus', { 
            isRunning: message.is_running, 
            tick: message.current_tick 
          })
          setData(prevData => prevData ? {
            ...prevData,
            current_tick: message.current_tick
          } : null)
          console.log('✅ SimulationStatus processed')
          break

        case 'Error':
          console.error('💥 Server Error Message:', message.message)
          setError(message.message)
          break

        default:
          console.warn('❓ Unknown message type received:', message)
          console.warn('Full message:', message)
      }
    } catch (err) {
      console.error('💥 Failed to parse WebSocket message:', err)
      console.error('Raw message data:', event.data)
      setError('Failed to parse message from server')
    }
  }, [convertBackendAnt, convertBackendColony, convertBackendFoodSource, convertBackendPheromoneTrail])

  // Reconnection logic with exponential backoff
  const scheduleReconnect = useCallback(() => {
    if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
      console.error('Max reconnection attempts reached')
      setConnectionState('error')
      setError('Unable to connect to simulation server after multiple attempts')
      return
    }

    const delay = Math.min(
      reconnectDelayBase * (2 ** reconnectAttemptsRef.current),
      30000 // Max 30 seconds
    )
    
    console.log(`Scheduling reconnection attempt ${reconnectAttemptsRef.current + 1} in ${delay}ms`)
    
    reconnectTimeoutRef.current = setTimeout(() => {
      reconnectAttemptsRef.current++
      connect()
    }, delay)
  }, [])

  // Connect to WebSocket
  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      console.log('WebSocket already open, skipping connection attempt')
      return
    }

    // Clear any existing reconnection timeout
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }

    console.log('Connecting to WebSocket:', wsUrl)
    console.log('Browser WebSocket support:', typeof WebSocket !== 'undefined')
    setConnectionState('connecting')
    setError(null)

    try {
      const ws = new WebSocket(wsUrl)
      wsRef.current = ws

      // Add ready state logging
      console.log('WebSocket created, initial readyState:', ws.readyState)

      // Connection timeout - if not connected within 5 seconds, consider it failed
      const connectionTimeout = setTimeout(() => {
        if (ws.readyState === WebSocket.CONNECTING) {
          console.error('WebSocket connection timeout after 5 seconds')
          ws.close()
          setConnectionState('error')
          setError('Connection timeout - check if server is accessible')
        }
      }, 5000)

      ws.onopen = () => {
        console.log('✅ WebSocket onopen event fired successfully')
        console.log('WebSocket readyState after open:', ws.readyState)
        console.log('WebSocket URL:', ws.url)
        console.log('WebSocket protocol:', ws.protocol)
        
        clearTimeout(connectionTimeout)
        
        // Send simulation ID to the server after connection
        if (simulationId) {
          const subscribeMessage = {
            type: 'Subscribe',
            simulation_id: Number.parseInt(simulationId, 10)
          }
          console.log('🔔 Sending subscription message:', subscribeMessage)
          console.log('🔔 Parsed simulation ID:', Number.parseInt(simulationId, 10))
          console.log('🔔 Original simulation ID string:', simulationId)
          
          try {
            ws.send(JSON.stringify(subscribeMessage))
            console.log('✅ Subscription message sent successfully')
            
            // Set a timeout to check if we receive any response within 10 seconds
            messageTimeoutRef.current = setTimeout(() => {
              console.warn('⏰ No messages received within 10 seconds after subscription')
              console.warn('This might indicate:')
              console.warn('- Server is not responding to subscription')
              console.warn('- No active simulation with ID:', simulationId)  
              console.warn('- Server is not sending data for this simulation')
              setError('No response from server after subscription - check if simulation exists')
            }, 10000)
          } catch (error) {
            console.error('❌ Failed to send subscription message:', error)
            setError(`Failed to subscribe: ${error}`)
          }
        } else {
          console.warn('⚠️ No simulation ID provided for subscription')
          setError('No simulation ID provided')
        }
        
        setConnectionState('connected')
        reconnectAttemptsRef.current = 0 // Reset reconnection attempts on successful connection
        setError(null)
      }

      ws.onmessage = handleMessage

      ws.onclose = (event) => {
        console.log('WebSocket connection closed:', {
          code: event.code,
          reason: event.reason,
          wasClean: event.wasClean,
          readyState: ws.readyState
        })
        
        clearTimeout(connectionTimeout)
        
        // More detailed error messages based on close codes
        let errorMessage = 'Connection closed'
        switch (event.code) {
          case 1000:
            errorMessage = 'Normal closure'
            break
          case 1001:
            errorMessage = 'Going away'
            break
          case 1002:
            errorMessage = 'Protocol error'
            break
          case 1003:
            errorMessage = 'Unsupported data type'
            break
          case 1006:
            errorMessage = 'Connection lost - check if server is running'
            break
          case 1011:
            errorMessage = 'Server error'
            break
          case 1015:
            errorMessage = 'TLS handshake failure'
            break
          default:
            errorMessage = `Connection closed (code: ${event.code})`
        }
        
        setConnectionState('disconnected')
        
        // Only attempt to reconnect if it wasn't a manual disconnection
        if (event.code !== 1000) {
          setError(errorMessage)
          scheduleReconnect()
        }
      }

      ws.onerror = (event) => {
        console.error('❌ WebSocket error event:', event)
        console.log('WebSocket readyState during error:', ws.readyState)
        console.log('Event details:', {
          type: event.type,
          target: event.target,
          isTrusted: event.isTrusted
        })
        
        clearTimeout(connectionTimeout)
        setConnectionState('error')
        setError('WebSocket connection failed - check server and network')
      }

      // Add periodic state monitoring for debugging
      const stateMonitor = setInterval(() => {
        if (ws.readyState === WebSocket.CONNECTING) {
          console.log('🟡 Still connecting... readyState:', ws.readyState)
        } else {
          clearInterval(stateMonitor)
        }
      }, 1000)

      // Clear state monitor after connection timeout
      setTimeout(() => clearInterval(stateMonitor), 10000)

    } catch (err) {
      console.error('Failed to create WebSocket connection:', err)
      setConnectionState('error')
      setError(`Failed to create WebSocket: ${err instanceof Error ? err.message : 'Unknown error'}`)
      scheduleReconnect()
    }
  }, [wsUrl, simulationId, handleMessage, scheduleReconnect])

  // Disconnect from WebSocket
  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }

    if (messageTimeoutRef.current) {
      clearTimeout(messageTimeoutRef.current)
      messageTimeoutRef.current = null
    }

    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect')
      wsRef.current = null
    }
    
    setConnectionState('disconnected')
    reconnectAttemptsRef.current = 0
  }, [])

  // Effect to establish initial connection
  useEffect(() => {
    if (simulationId) {
      connect()
    }

    return () => {
      disconnect()
    }
  }, [simulationId, connect, disconnect])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      disconnect()
    }
  }, [disconnect])

  return {
    data,
    connectionState,
    isLoading,
    error,
    connect,
    disconnect,
    lastUpdateTime,
  }
} 