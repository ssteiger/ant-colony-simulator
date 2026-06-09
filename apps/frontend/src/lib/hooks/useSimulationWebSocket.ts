import { useCallback, useEffect, useRef, useState } from 'react'

// ── binary protocol (must match rust-backend/src/server/binary.rs) ────
const FRAME_INIT = 1
const FRAME_ANTS = 2
const FRAME_PHEROMONE = 3
const FRAME_FOOD = 4

export interface ColonyInit {
  id: number
  x: number
  y: number
  radius: number
  hue: number
}

export interface FoodInit {
  id: number
  x: number
  y: number
  amount: number
  maxAmount: number
}

export interface WorldInit {
  simulationId: number
  tick: number
  worldWidth: number
  worldHeight: number
  terrainW: number
  terrainH: number
  terrainCellSize: number
  /** bit-packed solid mask, LSB-first (1 = rock) */
  terrainBits: Uint8Array
  pheromoneW: number
  pheromoneH: number
  pheromoneCellSize: number
  colonies: ColonyInit[]
  foods: FoodInit[]
}

export interface AntFrame {
  /** quantized 0..65535 over world width/height */
  x: Uint16Array
  y: Uint16Array
  /** quantized heading, 0..255 over 2*PI */
  heading: Uint8Array
  /** bits 0-1 role, bit 2 carrying */
  flags: Uint8Array
  count: number
  tick: number
  /** performance.now() when the frame arrived */
  time: number
}

export interface SimStats {
  tick: number
  antCount: number
  foodCollected: number
  colonyFood: number
  worldFood: number
  tps: number
}

/**
 * Mutable buffers shared with the renderer. Updated in place on each
 * WebSocket frame -- never triggers React re-renders.
 */
export interface SimBuffers {
  prev: AntFrame
  curr: AntFrame
  hasAnts: boolean
  pheromoneFood: Uint8Array | null
  pheromoneHome: Uint8Array | null
  pheromoneDirty: boolean
  /** food source id -> current amount */
  foodAmounts: Map<number, number>
  foodDirty: boolean
}

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error'

function makeAntFrame(capacity: number): AntFrame {
  return {
    x: new Uint16Array(capacity),
    y: new Uint16Array(capacity),
    heading: new Uint8Array(capacity),
    flags: new Uint8Array(capacity),
    count: 0,
    tick: 0,
    time: 0,
  }
}

function growFrame(frame: AntFrame, needed: number) {
  if (frame.x.length >= needed) return
  const capacity = Math.max(needed, frame.x.length * 2, 1024)
  frame.x = new Uint16Array(capacity)
  frame.y = new Uint16Array(capacity)
  frame.heading = new Uint8Array(capacity)
  frame.flags = new Uint8Array(capacity)
}

function decodeInit(view: DataView, bytes: Uint8Array): WorldInit {
  let o = 1 // skip frame type
  o += 1 // protocol version
  const simulationId = view.getInt32(o, true)
  o += 4
  const tick = Number(view.getBigUint64(o, true))
  o += 8
  const worldWidth = view.getFloat32(o, true)
  o += 4
  const worldHeight = view.getFloat32(o, true)
  o += 4
  const terrainW = view.getUint32(o, true)
  o += 4
  const terrainH = view.getUint32(o, true)
  o += 4
  const terrainCellSize = view.getFloat32(o, true)
  o += 4
  const pheromoneW = view.getUint32(o, true)
  o += 4
  const pheromoneH = view.getUint32(o, true)
  o += 4
  const pheromoneCellSize = view.getFloat32(o, true)
  o += 4

  const colonyCount = view.getUint16(o, true)
  o += 2
  const colonies: ColonyInit[] = []
  for (let i = 0; i < colonyCount; i++) {
    colonies.push({
      id: view.getUint32(o, true),
      x: view.getFloat32(o + 4, true),
      y: view.getFloat32(o + 8, true),
      radius: view.getFloat32(o + 12, true),
      hue: view.getUint16(o + 16, true),
    })
    o += 18
  }

  const foodCount = view.getUint16(o, true)
  o += 2
  const foods: FoodInit[] = []
  for (let i = 0; i < foodCount; i++) {
    foods.push({
      id: view.getUint32(o, true),
      x: view.getFloat32(o + 4, true),
      y: view.getFloat32(o + 8, true),
      amount: view.getFloat32(o + 12, true),
      maxAmount: view.getFloat32(o + 16, true),
    })
    o += 20
  }

  const bitLen = Math.ceil((terrainW * terrainH) / 8)
  const terrainBits = bytes.slice(o, o + bitLen)

  return {
    simulationId,
    tick,
    worldWidth,
    worldHeight,
    terrainW,
    terrainH,
    terrainCellSize,
    terrainBits,
    pheromoneW,
    pheromoneH,
    pheromoneCellSize,
    colonies,
    foods,
  }
}

export function useSimulationWebSocket(
  simulationId: string,
  wsUrl = import.meta.env.VITE_SIMULATION_WS_URL ?? 'ws://127.0.0.1:8080/ws',
) {
  const [init, setInit] = useState<WorldInit | null>(null)
  const [stats, setStats] = useState<SimStats | null>(null)
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected')

  const buffersRef = useRef<SimBuffers>({
    prev: makeAntFrame(0),
    curr: makeAntFrame(0),
    hasAnts: false,
    pheromoneFood: null,
    pheromoneHome: null,
    pheromoneDirty: false,
    foodAmounts: new Map(),
    foodDirty: false,
  })

  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const attemptsRef = useRef(0)

  const handleBinary = useCallback((data: ArrayBuffer) => {
    const view = new DataView(data)
    const bytes = new Uint8Array(data)
    const frameType = view.getUint8(0)
    const buffers = buffersRef.current

    if (frameType === FRAME_INIT) {
      const world = decodeInit(view, bytes)
      // reset ant/pheromone buffers for the (possibly new) world
      buffers.hasAnts = false
      buffers.prev.count = 0
      buffers.curr.count = 0
      buffers.pheromoneFood = null
      buffers.pheromoneHome = null
      buffers.pheromoneDirty = false
      buffers.foodAmounts = new Map(world.foods.map((f) => [f.id, f.amount]))
      buffers.foodDirty = true
      setInit(world)
      return
    }

    if (frameType === FRAME_ANTS) {
      const tick = Number(view.getBigUint64(1, true))
      const count = view.getUint32(9, true)

      // recycle: previous "curr" becomes "prev", decode into old "prev"
      const target = buffers.prev
      buffers.prev = buffers.curr
      buffers.curr = target

      growFrame(target, count)
      const { x, y, heading, flags } = target
      let o = 13
      for (let i = 0; i < count; i++) {
        x[i] = view.getUint16(o, true)
        y[i] = view.getUint16(o + 2, true)
        heading[i] = view.getUint8(o + 4)
        flags[i] = view.getUint8(o + 5)
        o += 6
      }
      target.count = count
      target.tick = tick
      target.time = performance.now()
      if (!buffers.hasAnts) {
        // first frame: make prev identical so interpolation has a source
        buffers.prev = target
        buffers.hasAnts = true
      }
      return
    }

    if (frameType === FRAME_PHEROMONE) {
      const w = view.getUint32(9, true)
      const h = view.getUint32(13, true)
      const size = w * h
      buffers.pheromoneFood = bytes.subarray(17, 17 + size)
      buffers.pheromoneHome = bytes.subarray(17 + size, 17 + size * 2)
      buffers.pheromoneDirty = true
      return
    }

    if (frameType === FRAME_FOOD) {
      const count = view.getUint16(9, true)
      let o = 11
      for (let i = 0; i < count; i++) {
        buffers.foodAmounts.set(view.getUint32(o, true), view.getFloat32(o + 4, true))
        o += 8
      }
      buffers.foodDirty = true
    }
  }, [])

  const handleMessage = useCallback(
    (ev: MessageEvent) => {
      if (ev.data instanceof ArrayBuffer) {
        handleBinary(ev.data)
        return
      }
      try {
        const msg = JSON.parse(ev.data)
        if (msg.type === 'stats') {
          setStats({
            tick: msg.tick,
            antCount: msg.antCount,
            foodCollected: msg.foodCollected,
            colonyFood: msg.colonyFood,
            worldFood: msg.worldFood,
            tps: msg.tps,
          })
        }
      } catch {
        // ignore malformed text messages
      }
    },
    [handleBinary],
  )

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return
    if (reconnectTimer.current) {
      clearTimeout(reconnectTimer.current)
      reconnectTimer.current = null
    }

    setConnectionState('connecting')

    const ws = new WebSocket(wsUrl)
    ws.binaryType = 'arraybuffer'
    wsRef.current = ws

    ws.onopen = () => {
      setConnectionState('connected')
      attemptsRef.current = 0
      ws.send(JSON.stringify({ type: 'Subscribe', simulation_id: Number(simulationId) }))
    }

    ws.onmessage = handleMessage

    ws.onclose = (ev) => {
      setConnectionState('disconnected')
      if (ev.code !== 1000 && attemptsRef.current < 20) {
        const delay = Math.min(1000 * 2 ** attemptsRef.current, 15000)
        attemptsRef.current++
        reconnectTimer.current = setTimeout(connect, delay)
      }
    }

    ws.onerror = () => {
      setConnectionState('error')
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

  return { init, stats, connectionState, buffersRef }
}
