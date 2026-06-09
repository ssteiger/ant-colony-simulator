import { useEffect, useRef, useState } from 'react'
import { createFileRoute, Link } from '@tanstack/react-router'
import { useSimulationWebSocket } from '~/lib/hooks/useSimulationWebSocket'
import type { SimRenderer } from '~/lib/simulation/pixi-renderer'

function formatNumber(n: number): string {
  return Math.round(n).toLocaleString('en-US')
}

function ConnectionBadge({ state }: { state: string }) {
  const styles: Record<string, string> = {
    connected: 'bg-green-500/20 text-green-400',
    connecting: 'bg-yellow-500/20 text-yellow-400',
    disconnected: 'bg-zinc-500/20 text-zinc-400',
    error: 'bg-red-500/20 text-red-400',
  }
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium ${styles[state] ?? styles.disconnected}`}
    >
      <span
        className={`h-1.5 w-1.5 rounded-full ${
          state === 'connected'
            ? 'bg-green-400'
            : state === 'connecting'
              ? 'bg-yellow-400 animate-pulse'
              : 'bg-zinc-400'
        }`}
      />
      {state}
    </span>
  )
}

function SimulationViewer() {
  const { id } = Route.useParams()
  const { init, stats, connectionState, buffersRef } = useSimulationWebSocket(id)

  const containerRef = useRef<HTMLDivElement>(null)
  const rendererRef = useRef<SimRenderer | null>(null)
  const [rendererReady, setRendererReady] = useState(false)
  const [fps, setFps] = useState(0)

  // create the Pixi renderer once on mount
  useEffect(() => {
    const container = containerRef.current
    if (!container) return
    let cancelled = false

    import('~/lib/simulation/pixi-renderer').then(async ({ createSimRenderer }) => {
      if (cancelled) return
      const renderer = await createSimRenderer(container, buffersRef)
      if (cancelled) {
        renderer.destroy()
        return
      }
      rendererRef.current = renderer
      setRendererReady(true)
    })

    return () => {
      cancelled = true
      rendererRef.current?.destroy()
      rendererRef.current = null
    }
  }, [buffersRef])

  // (re)build the world whenever an INIT frame arrives
  useEffect(() => {
    if (init && rendererReady) {
      rendererRef.current?.resetWorld(init)
    }
  }, [init, rendererReady])

  useEffect(() => {
    const timer = setInterval(() => {
      setFps(Math.round(rendererRef.current?.getFps() ?? 0))
    }, 1000)
    return () => clearInterval(timer)
  }, [])

  return (
    <div className="relative h-screen w-screen overflow-hidden bg-[#120d08]">
      <div ref={containerRef} className="absolute inset-0" />

      {/* top bar */}
      <div className="pointer-events-none absolute inset-x-0 top-0 flex items-start justify-between p-4">
        <div className="pointer-events-auto flex items-center gap-3 rounded-lg bg-black/60 px-4 py-2 backdrop-blur">
          <Link
            to="/"
            className="text-sm font-medium text-zinc-300 transition-colors hover:text-white"
          >
            &larr; Simulations
          </Link>
          <span className="text-zinc-600">|</span>
          <span className="text-sm font-semibold text-white">
            Simulation #{id}
          </span>
          <ConnectionBadge state={connectionState} />
        </div>

        <div className="pointer-events-auto rounded-lg bg-black/60 px-4 py-2 backdrop-blur">
          <div className="grid grid-cols-3 gap-x-6 gap-y-1 text-xs text-zinc-400">
            <span>
              Ants{' '}
              <span className="font-mono text-white">
                {stats ? formatNumber(stats.antCount) : '-'}
              </span>
            </span>
            <span>
              Collected{' '}
              <span className="font-mono text-white">
                {stats ? formatNumber(stats.foodCollected) : '-'}
              </span>
            </span>
            <span>
              Stored{' '}
              <span className="font-mono text-white">
                {stats ? formatNumber(stats.colonyFood) : '-'}
              </span>
            </span>
            <span>
              Tick{' '}
              <span className="font-mono text-white">
                {stats ? formatNumber(stats.tick) : '-'}
              </span>
            </span>
            <span>
              Sim <span className="font-mono text-white">{stats ? `${Math.round(stats.tps)} tps` : '-'}</span>
            </span>
            <span>
              Render <span className="font-mono text-white">{fps} fps</span>
            </span>
          </div>
        </div>
      </div>

      {/* legend */}
      <div className="pointer-events-none absolute bottom-4 left-4 rounded-lg bg-black/60 px-4 py-3 backdrop-blur">
        <div className="space-y-1.5 text-xs text-zinc-300">
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-[#9a6a3f]" /> Worker
          </div>
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-[#d9b25f]" /> Scout
          </div>
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-[#c25b40]" /> Soldier
          </div>
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-[#52d273]" /> Carrying food
          </div>
          <div className="mt-2 border-t border-zinc-700 pt-2 text-zinc-500">
            Drag to pan, scroll to zoom
          </div>
        </div>
      </div>

      {/* connection overlay */}
      {connectionState !== 'connected' && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/50">
          <div className="rounded-lg bg-zinc-900 px-8 py-6 text-center">
            <p className="text-lg font-medium text-white">
              {connectionState === 'connecting'
                ? 'Connecting to simulation...'
                : 'Simulation backend unreachable'}
            </p>
            <p className="mt-2 text-sm text-zinc-400">
              {connectionState === 'connecting'
                ? 'Waiting for the Rust backend'
                : 'Start it with: cd apps/rust-backend && sh ./run.sh'}
            </p>
          </div>
        </div>
      )}
    </div>
  )
}

export const Route = createFileRoute('/simulation/$id')({
  component: SimulationViewer,
})
