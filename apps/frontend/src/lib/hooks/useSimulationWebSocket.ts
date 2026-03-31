import { useState, useEffect, useRef, useCallback } from 'react'

// ── types matching the Rust backend messages ──────────────────────────

export interface AntData {
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

export interface ColonyData {
  id: number
  center: [number, number]
  radius: number
  population: number
  food_stored: number
  color_hue: number
}

export interface FoodData {
  id: number
  position: [number, number]
  food_type: string
  amount: number
}

export interface PheromoneGrid {
  grid_w: number
  grid_h: number
  cell_size: number
  food: number[]
  home: number[]
}

type ServerMessage =
  | {
      type: 'FullState'
      simulation_id: number
      tick: number
      world_width: number
      world_height: number
      ants: AntData[]
      colonies: ColonyData[]
      food_sources: FoodData[]
      pheromone_grid: PheromoneGrid
    }
  | {
      type: 'DeltaUpdate'
      simulation_id: number
      tick: number
      updated_ants: AntData[]
      updated_colonies: ColonyData[]
      updated_food_sources: FoodData[]
      removed_ant_ids: number[]
      removed_food_source_ids: number[]
    }
  | { type: 'Error'; message: string }

export interface SimulationSnapshot {
  simulationId: number
  tick: number
  worldWidth: number
  worldHeight: number
  ants: AntData[]
  colonies: ColonyData[]
  foodSources: FoodData[]
  pheromoneGrid: PheromoneGrid | null
}

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error'

export function useSimulationWebSocket(
  simulationId: string,
  wsUrl = 'ws://127.0.0.1:8080/ws',
) {
  const [current, setCurrent] = useState<SimulationSnapshot | null>(null)
  const [prev, setPrev] = useState<SimulationSnapshot | null>(null)
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected')
  const [error, setError] = useState<string | null>(null)
  const updateTimeRef = useRef(0)

  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const attemptsRef = useRef(0)

  const handleMessage = useCallback((ev: MessageEvent) => {
    try {
      const msg: ServerMessage = JSON.parse(ev.data)

      if (msg.type === 'FullState') {
        setPrev((c) => c)
        setCurrent((old) => {
          if (old) setPrev(old)
          return {
            simulationId: msg.simulation_id,
            tick: msg.tick,
            worldWidth: msg.world_width,
            worldHeight: msg.world_height,
            ants: msg.ants,
            colonies: msg.colonies,
            foodSources: msg.food_sources,
            pheromoneGrid: msg.pheromone_grid,
          }
        })
        updateTimeRef.current = performance.now()
      } else if (msg.type === 'DeltaUpdate') {
        setCurrent((old) => {
          if (!old) return old
          setPrev(old)

          const antMap = new Map(old.ants.map((a) => [a.id, a]))
          for (const a of msg.updated_ants) antMap.set(a.id, a)
          for (const id of msg.removed_ant_ids) antMap.delete(id)

          const foodMap = new Map(old.foodSources.map((f) => [f.id, f]))
          for (const f of msg.updated_food_sources) foodMap.set(f.id, f)
          for (const id of msg.removed_food_source_ids) foodMap.delete(id)

          return {
            ...old,
            tick: msg.tick,
            ants: Array.from(antMap.values()),
            colonies: msg.updated_colonies.length > 0 ? msg.updated_colonies : old.colonies,
            foodSources: Array.from(foodMap.values()),
          }
        })
        updateTimeRef.current = performance.now()
      } else if (msg.type === 'Error') {
        setError(msg.message)
      }
    } catch {
      // ignore parse errors
    }
  }, [])

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return
    if (reconnectTimer.current) {
      clearTimeout(reconnectTimer.current)
      reconnectTimer.current = null
    }

    setConnectionState('connecting')
    setError(null)

    const ws = new WebSocket(wsUrl)
    wsRef.current = ws

    ws.onopen = () => {
      setConnectionState('connected')
      attemptsRef.current = 0
      ws.send(JSON.stringify({ type: 'Subscribe', simulation_id: Number(simulationId) }))
    }

    ws.onmessage = handleMessage

    ws.onclose = (ev) => {
      setConnectionState('disconnected')
      if (ev.code !== 1000 && attemptsRef.current < 10) {
        const delay = Math.min(1000 * 2 ** attemptsRef.current, 15000)
        attemptsRef.current++
        reconnectTimer.current = setTimeout(connect, delay)
      }
    }

    ws.onerror = () => {
      setConnectionState('error')
      setError('WebSocket connection failed')
    }
  }, [wsUrl, simulationId, handleMessage])

  const disconnect = useCallback(() => {
    if (reconnectTimer.current) clearTimeout(reconnectTimer.current)
    wsRef.current?.close(1000)
    wsRef.current = null
    setConnectionState('disconnected')
  }, [])

  useEffect(() => {
    if (simulationId) connect()
    return disconnect
  }, [simulationId, connect, disconnect])

  return {
    data: current,
    prevData: prev,
    connectionState,
    error,
    connect,
    disconnect,
    updateTimeRef,
  }
}
