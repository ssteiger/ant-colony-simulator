import { createFileRoute } from '@tanstack/react-router'
import { type MutableRefObject, type RefObject, useRef, useEffect, useCallback } from 'react'
import {
  useSimulationWebSocket,
  type SimulationSnapshot,
  type PheromoneGrid,
  type AntData,
} from '~/lib/hooks/useSimulationWebSocket'

// ── role constants (must match Rust ROLE_WORKER/SCOUT/SOLDIER) ────────
const ROLE_WORKER = 0
const ROLE_SCOUT = 1
const ROLE_SOLDIER = 2

// ── colors ────────────────────────────────────────────────────────────
const ANT_COLORS: Record<number, { foraging: string; returning: string }> = {
  [ROLE_WORKER]: { foraging: '#60a5fa', returning: '#f59e0b' },
  [ROLE_SCOUT]: { foraging: '#34d399', returning: '#fbbf24' },
  [ROLE_SOLDIER]: { foraging: '#f87171', returning: '#f87171' },
}

const BG_COLOR = '#060610'
const BORDER_COLOR = '#1a1a2e'

// ── Canvas Renderer ───────────────────────────────────────────────────

interface Camera {
  x: number
  y: number
  zoom: number
}

function SimCanvas({
  data,
  prevData,
  updateTimeRef,
}: {
  data: SimulationSnapshot | null
  prevData: SimulationSnapshot | null
  updateTimeRef: RefObject<number>
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const wrapperRef = useRef<HTMLDivElement>(null)
  const cameraRef = useRef<Camera>({ x: 0, y: 0, zoom: 1 })
  const dragRef = useRef({ active: false, lastX: 0, lastY: 0 })
  const pheroCanvas = useRef<HTMLCanvasElement | null>(null)
  const minimapCanvas = useRef<HTMLCanvasElement | null>(null)
  const antSpriteRef = useRef<HTMLImageElement | null>(null)
  const tintCacheRef = useRef<Map<string, HTMLCanvasElement>>(new Map())
  const rafRef = useRef(0)

  const dataRef = useRef(data)
  const prevRef = useRef(prevData)
  dataRef.current = data
  prevRef.current = prevData

  const initialized = useRef(false)
  useEffect(() => {
    if (data && !initialized.current) {
      cameraRef.current.x = data.worldWidth / 2
      cameraRef.current.y = data.worldHeight / 2
      initialized.current = true
    }
  }, [data])

  useEffect(() => {
    if (!data?.pheromoneGrid) return
    updatePheromoneTexture(data.pheromoneGrid, pheroCanvas)
  }, [data?.pheromoneGrid])

  useEffect(() => {
    const resize = () => {
      const canvas = canvasRef.current
      const wrapper = wrapperRef.current
      if (!canvas || !wrapper) return
      const rect = wrapper.getBoundingClientRect()
      canvas.width = rect.width
      canvas.height = rect.height
    }
    resize()
    window.addEventListener('resize', resize)
    return () => window.removeEventListener('resize', resize)
  }, [])

  // load ant sprite once
  useEffect(() => {
    const img = new Image()
    img.src = '/ant_sprite.png'
    img.onload = () => {
      antSpriteRef.current = img
    }
  }, [])

  // ── render loop ─────────────────────────────────────────────────────
  useEffect(() => {
    const render = () => {
      const canvas = canvasRef.current
      if (!canvas) {
        rafRef.current = requestAnimationFrame(render)
        return
      }
      const ctx = canvas.getContext('2d')
      if (!ctx) return

      const cam = cameraRef.current
      const cur = dataRef.current
      const prev = prevRef.current
      const W = canvas.width
      const H = canvas.height

      ctx.clearRect(0, 0, W, H)

      if (!cur) {
        rafRef.current = requestAnimationFrame(render)
        return
      }

      const elapsed = performance.now() - updateTimeRef.current
      const t = Math.min(elapsed / 50, 1.0)

      ctx.save()
      ctx.translate(W / 2, H / 2)
      ctx.scale(cam.zoom, cam.zoom)
      ctx.translate(-cam.x, -cam.y)

      // world border
      ctx.strokeStyle = BORDER_COLOR
      ctx.lineWidth = 2
      ctx.strokeRect(0, 0, cur.worldWidth, cur.worldHeight)

      // pheromone heatmap with additive blending for glow effect
      if (pheroCanvas.current) {
        ctx.globalCompositeOperation = 'lighter'
        ctx.globalAlpha = 0.8
        ctx.imageSmoothingEnabled = true
        ctx.drawImage(pheroCanvas.current, 0, 0, cur.worldWidth, cur.worldHeight)
        ctx.globalCompositeOperation = 'source-over'
        ctx.globalAlpha = 1.0
      }

      // food sources with glow
      for (const f of cur.foodSources) {
        const r = Math.max(3, Math.min(f.amount / 40, 12))
        // outer glow
        const grad = ctx.createRadialGradient(
          f.position[0], f.position[1], r * 0.3,
          f.position[0], f.position[1], r * 2,
        )
        grad.addColorStop(0, 'rgba(34, 197, 94, 0.4)')
        grad.addColorStop(1, 'rgba(34, 197, 94, 0)')
        ctx.beginPath()
        ctx.arc(f.position[0], f.position[1], r * 2, 0, Math.PI * 2)
        ctx.fillStyle = grad
        ctx.fill()
        // core
        ctx.beginPath()
        ctx.arc(f.position[0], f.position[1], r, 0, Math.PI * 2)
        ctx.fillStyle = '#22c55e'
        ctx.fill()
      }

      // colonies with glow
      for (const c of cur.colonies) {
        const grad = ctx.createRadialGradient(
          c.center[0], c.center[1], c.radius * 0.2,
          c.center[0], c.center[1], c.radius,
        )
        grad.addColorStop(0, `hsla(${c.color_hue}, 60%, 55%, 0.25)`)
        grad.addColorStop(1, `hsla(${c.color_hue}, 60%, 40%, 0.02)`)
        ctx.beginPath()
        ctx.arc(c.center[0], c.center[1], c.radius, 0, Math.PI * 2)
        ctx.fillStyle = grad
        ctx.fill()
        ctx.strokeStyle = `hsla(${c.color_hue}, 60%, 50%, 0.35)`
        ctx.lineWidth = 1
        ctx.stroke()

        ctx.beginPath()
        ctx.arc(c.center[0], c.center[1], 4, 0, Math.PI * 2)
        ctx.fillStyle = `hsl(${c.color_hue}, 70%, 60%)`
        ctx.fill()
      }

      // ants (interpolated, role-shaped, with cargo indicator)
      const prevMap = buildAntMap(prev)
      for (const ant of cur.ants) {
        const pa = prevMap?.get(ant.id)
        let ax = ant.position[0]
        let ay = ant.position[1]
        let angle = ant.angle

        if (pa) {
          ax = pa.position[0] + (ax - pa.position[0]) * t
          ay = pa.position[1] + (ay - pa.position[1]) * t
          let ad = ant.angle - pa.angle
          if (ad > Math.PI) ad -= Math.PI * 2
          if (ad < -Math.PI) ad += Math.PI * 2
          angle = pa.angle + ad * t
        }

        drawAnt(ctx, ax, ay, angle, ant.ant_type_id, ant.state, antSpriteRef.current, tintCacheRef.current)
      }

      ctx.restore()

      // minimap
      drawMinimap(ctx, cur, cam, W, H)

      rafRef.current = requestAnimationFrame(render)
    }

    rafRef.current = requestAnimationFrame(render)
    return () => cancelAnimationFrame(rafRef.current)
  }, [updateTimeRef])

  // ── mouse handlers ──────────────────────────────────────────────────
  const onWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault()
    const factor = e.deltaY > 0 ? 0.9 : 1.1
    cameraRef.current.zoom = Math.max(0.2, Math.min(8, cameraRef.current.zoom * factor))
  }, [])

  const onPointerDown = useCallback((e: React.PointerEvent) => {
    dragRef.current = { active: true, lastX: e.clientX, lastY: e.clientY }
    ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
  }, [])

  const onPointerMove = useCallback((e: React.PointerEvent) => {
    if (!dragRef.current.active) return
    const dx = (e.clientX - dragRef.current.lastX) / cameraRef.current.zoom
    const dy = (e.clientY - dragRef.current.lastY) / cameraRef.current.zoom
    cameraRef.current.x -= dx
    cameraRef.current.y -= dy
    dragRef.current.lastX = e.clientX
    dragRef.current.lastY = e.clientY
  }, [])

  const onPointerUp = useCallback(() => {
    dragRef.current.active = false
  }, [])

  const onDoubleClick = useCallback(() => {
    if (dataRef.current) {
      cameraRef.current.x = dataRef.current.worldWidth / 2
      cameraRef.current.y = dataRef.current.worldHeight / 2
      cameraRef.current.zoom = 1
    }
  }, [])

  return (
    <div ref={wrapperRef} className="flex-1 min-h-0 rounded-lg overflow-hidden border border-border">
      <canvas
        ref={canvasRef}
        className="w-full h-full cursor-grab active:cursor-grabbing"
        onWheel={onWheel}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onDoubleClick={onDoubleClick}
      />
    </div>
  )
}

// ── drawing helpers ───────────────────────────────────────────────────

const ANT_SIZES: Record<number, number> = {
  [ROLE_WORKER]: 14,
  [ROLE_SCOUT]: 12,
  [ROLE_SOLDIER]: 18,
}

function getTintedSprite(
  sprite: HTMLImageElement,
  color: string,
  cache: Map<string, HTMLCanvasElement>,
): HTMLCanvasElement {
  const cached = cache.get(color)
  if (cached) return cached

  const c = document.createElement('canvas')
  c.width = sprite.width
  c.height = sprite.height
  const ctx = c.getContext('2d')
  if (!ctx) return c

  ctx.drawImage(sprite, 0, 0)
  ctx.globalCompositeOperation = 'source-atop'
  ctx.fillStyle = color
  ctx.fillRect(0, 0, c.width, c.height)

  cache.set(color, c)
  return c
}

function drawAnt(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  angle: number,
  role: number,
  state: string,
  sprite: HTMLImageElement | null,
  tintCache: Map<string, HTMLCanvasElement>,
) {
  const colors = ANT_COLORS[role] ?? ANT_COLORS[ROLE_WORKER]
  const color = state === 'returning' ? colors.returning : colors.foraging
  const size = ANT_SIZES[role] ?? ANT_SIZES[ROLE_WORKER]
  const half = size / 2

  ctx.save()
  ctx.translate(x, y)
  ctx.rotate(angle)

  if (sprite) {
    const tinted = getTintedSprite(sprite, color, tintCache)
    ctx.drawImage(tinted, -half, -half, size, size)
  } else {
    ctx.fillStyle = color
    ctx.beginPath()
    ctx.moveTo(4, 0)
    ctx.lineTo(-3, -2.5)
    ctx.lineTo(-2, 0)
    ctx.lineTo(-3, 2.5)
    ctx.closePath()
    ctx.fill()
  }

  if (state === 'returning') {
    ctx.globalCompositeOperation = 'lighter'
    ctx.fillStyle = 'rgba(251, 191, 36, 0.25)'
    ctx.beginPath()
    ctx.arc(0, 0, half + 2, 0, Math.PI * 2)
    ctx.fill()
    ctx.globalCompositeOperation = 'source-over'
  }

  ctx.restore()
}

function drawMinimap(
  ctx: CanvasRenderingContext2D,
  data: SimulationSnapshot,
  cam: Camera,
  canvasW: number,
  canvasH: number,
) {
  const mmW = 160
  const mmH = Math.round(mmW * (data.worldHeight / data.worldWidth))
  const mmX = canvasW - mmW - 12
  const mmY = canvasH - mmH - 12
  const sx = mmW / data.worldWidth
  const sy = mmH / data.worldHeight

  ctx.save()

  // background
  ctx.globalAlpha = 0.7
  ctx.fillStyle = '#0c0c1a'
  ctx.fillRect(mmX - 1, mmY - 1, mmW + 2, mmH + 2)
  ctx.strokeStyle = '#334155'
  ctx.lineWidth = 1
  ctx.strokeRect(mmX - 1, mmY - 1, mmW + 2, mmH + 2)
  ctx.globalAlpha = 1.0

  // food
  ctx.fillStyle = '#22c55e'
  for (const f of data.foodSources) {
    ctx.fillRect(mmX + f.position[0] * sx - 1, mmY + f.position[1] * sy - 1, 2, 2)
  }

  // colonies
  for (const c of data.colonies) {
    ctx.fillStyle = `hsl(${c.color_hue}, 60%, 55%)`
    ctx.fillRect(mmX + c.center[0] * sx - 2, mmY + c.center[1] * sy - 2, 4, 4)
  }

  // ants as single pixels
  for (const a of data.ants) {
    ctx.fillStyle = a.state === 'returning' ? '#f59e0b' : '#60a5fa'
    ctx.fillRect(mmX + a.position[0] * sx, mmY + a.position[1] * sy, 1, 1)
  }

  // viewport rect
  const vw = canvasW / cam.zoom
  const vh = canvasH / cam.zoom
  const vx = cam.x - vw / 2
  const vy = cam.y - vh / 2

  ctx.strokeStyle = '#f8fafc'
  ctx.lineWidth = 1
  ctx.globalAlpha = 0.6
  ctx.strokeRect(mmX + vx * sx, mmY + vy * sy, vw * sx, vh * sy)
  ctx.globalAlpha = 1.0

  ctx.restore()
}

function buildAntMap(snap: SimulationSnapshot | null): Map<number, AntData> | null {
  if (!snap) return null
  return new Map(snap.ants.map((a) => [a.id, a]))
}

function updatePheromoneTexture(
  grid: PheromoneGrid,
  ref: MutableRefObject<HTMLCanvasElement | null>,
) {
  if (!ref.current) {
    ref.current = document.createElement('canvas')
  }
  const c = ref.current
  c.width = grid.grid_w
  c.height = grid.grid_h
  const ctx = c.getContext('2d')
  if (!ctx) return
  const img = ctx.createImageData(grid.grid_w, grid.grid_h)

  for (let i = 0; i < grid.grid_w * grid.grid_h; i++) {
    const f = grid.food[i]
    const h = grid.home[i]
    const px = i * 4
    // food = green glow, home = cyan-blue glow, overlap = bright teal
    img.data[px] = 0
    img.data[px + 1] = Math.min(255, f * 4)
    img.data[px + 2] = Math.min(255, h * 3)
    img.data[px + 3] = Math.min(220, f * 5 + h * 4)
  }

  ctx.putImageData(img, 0, 0)
}

// ── page component ───────────────────────────────────────────────────

function SimulationPage() {
  const { id } = Route.useParams()
  const { data, prevData, connectionState, error, connect, updateTimeRef } =
    useSimulationWebSocket(id)

  const foraging = data?.ants.filter((a) => a.state === 'foraging').length ?? 0
  const returning = data?.ants.filter((a) => a.state === 'returning').length ?? 0
  const workers = data?.ants.filter((a) => a.ant_type_id === ROLE_WORKER).length ?? 0
  const scouts = data?.ants.filter((a) => a.ant_type_id === ROLE_SCOUT).length ?? 0
  const soldiers = data?.ants.filter((a) => a.ant_type_id === ROLE_SOLDIER).length ?? 0

  const statusColor =
    connectionState === 'connected'
      ? 'text-green-500'
      : connectionState === 'connecting'
        ? 'text-yellow-500'
        : 'text-red-500'

  return (
    <div className="flex-1 flex flex-col p-4 gap-3 h-full">
      {/* header */}
      <div className="flex items-center justify-between flex-shrink-0 flex-wrap gap-2">
        <div className="flex items-center gap-4">
          <h2 className="text-lg font-semibold tracking-tight">Ant Colony</h2>
          <span className={`text-sm font-medium ${statusColor}`}>
            {connectionState === 'connected' ? 'Live' : connectionState}
          </span>
          {error && (
            <button type="button" onClick={connect} className="text-xs text-blue-500 underline">
              retry
            </button>
          )}
        </div>

        {data && (
          <div className="flex items-center gap-4 text-xs text-muted-foreground font-mono">
            <span>tick {data.tick.toLocaleString()}</span>
            <span className="text-foreground font-semibold">{data.ants.length} ants</span>
            <span className="text-blue-400">{workers}w</span>
            <span className="text-emerald-400">{scouts}s</span>
            <span className="text-red-400">{soldiers}g</span>
            <span className="text-muted-foreground">|</span>
            <span className="text-blue-300">{foraging} out</span>
            <span className="text-amber-300">{returning} back</span>
            <span className="text-green-400">
              {data.colonies[0] ? `${Math.round(data.colonies[0].food_stored)} stored` : ''}
            </span>
          </div>
        )}
      </div>

      {/* canvas */}
      {data ? (
        <SimCanvas data={data} prevData={prevData} updateTimeRef={updateTimeRef} />
      ) : (
        <div className="flex-1 flex items-center justify-center rounded-lg border border-border bg-muted/30">
          <p className="text-muted-foreground text-sm">
            {connectionState === 'connecting'
              ? 'Connecting to simulation server...'
              : 'Start the Rust backend to see the simulation.'}
          </p>
        </div>
      )}

      {/* controls hint */}
      <div className="flex items-center justify-between flex-shrink-0">
        <p className="text-xs text-muted-foreground">
          Scroll to zoom &middot; Drag to pan &middot; Double-click to reset
        </p>
        <div className="flex items-center gap-3 text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-blue-400 inline-block" /> Worker
          </span>
          <span className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-emerald-400 inline-block" /> Scout
          </span>
          <span className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-red-400 inline-block" /> Soldier
          </span>
          <span className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-amber-400 inline-block" /> Carrying food
          </span>
        </div>
      </div>
    </div>
  )
}

export const Route = createFileRoute('/_authenticated/_app/simulation/$id')({
  component: SimulationPage,
})
