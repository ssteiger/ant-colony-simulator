import type {
  SimBuffers,
  WorldInit,
} from '~/lib/hooks/useSimulationWebSocket'

export interface SimRenderer {
  resetWorld(init: WorldInit): void
  getFps(): number
  destroy(): void
}

const TAU = Math.PI * 2
const HEADING_TO_RAD = TAU / 256

// tint lookup indexed by flags (bits 0-1 role, bit 2 carrying)
const TINTS = new Uint32Array([
  0x9a6a3f, // worker
  0xd9b25f, // scout
  0xc25b40, // soldier
  0x9a6a3f, // (unused role 3)
  0x52d273, // worker carrying food
  0x86e09a, // scout carrying food
  0x52d273, // soldier carrying food
  0x52d273, // (unused role 3 carrying)
])

function hashJitter(i: number): number {
  // deterministic pseudo-random in [0, 1) for per-cell color variation
  let h = (i ^ 0x9e3779b9) >>> 0
  h = Math.imul(h ^ (h >>> 16), 0x45d9f3b)
  h = Math.imul(h ^ (h >>> 16), 0x45d9f3b)
  h ^= h >>> 16
  return (h >>> 8) / 16777216
}

function isSolid(bits: Uint8Array, i: number): boolean {
  return ((bits[i >> 3] >> (i & 7)) & 1) === 1
}

/** Paint terrain to an offscreen canvas: dark soil + jittered rock with shadowed edges. */
function paintTerrain(init: WorldInit): HTMLCanvasElement {
  const { terrainW: tw, terrainH: th, terrainBits: bits } = init
  const scale = 2
  const canvas = document.createElement('canvas')
  canvas.width = tw * scale
  canvas.height = th * scale
  const ctx = canvas.getContext('2d')
  if (!ctx) return canvas

  const img = ctx.createImageData(canvas.width, canvas.height)
  const data = img.data

  for (let gy = 0; gy < th; gy++) {
    for (let gx = 0; gx < tw; gx++) {
      const i = gy * tw + gx
      const solid = isSolid(bits, i)
      const j = hashJitter(i)

      let r: number
      let g: number
      let b: number
      if (solid) {
        // rock: warm grey with variation, darker near open edges
        const edge =
          (gx > 0 && !isSolid(bits, i - 1)) ||
          (gx < tw - 1 && !isSolid(bits, i + 1)) ||
          (gy > 0 && !isSolid(bits, i - tw)) ||
          (gy < th - 1 && !isSolid(bits, i + tw))
        const base = edge ? 38 : 72
        const v = base + j * 26
        r = v + 14
        g = v + 4
        b = v - 6
      } else {
        // soil: dark warm brown with subtle speckle
        const v = 26 + j * 14
        r = v + 10
        g = v + 1
        b = v - 8
        if (j > 0.97) {
          r += 26
          g += 18
          b += 6
        }
      }

      for (let sy = 0; sy < scale; sy++) {
        for (let sx = 0; sx < scale; sx++) {
          const o = ((gy * scale + sy) * canvas.width + gx * scale + sx) * 4
          data[o] = r
          data[o + 1] = g
          data[o + 2] = b
          data[o + 3] = 255
        }
      }
    }
  }

  ctx.putImageData(img, 0, 0)
  return canvas
}

export async function createSimRenderer(
  container: HTMLDivElement,
  buffersRef: { current: SimBuffers },
): Promise<SimRenderer> {
  const PIXI = await import('pixi.js')

  const app = new PIXI.Application()
  await app.init({
    resizeTo: container,
    backgroundColor: 0x120d08,
    antialias: true,
    resolution: Math.min(window.devicePixelRatio || 1, 2),
    autoDensity: true,
  })
  container.appendChild(app.canvas)

  // ── scene graph ──────────────────────────────────────────────────────
  const world = new PIXI.Container()
  app.stage.addChild(world)

  const terrainSprite = new PIXI.Sprite()
  const pheromoneSprite = new PIXI.Sprite()
  pheromoneSprite.blendMode = 'add'
  const foodLayer = new PIXI.Graphics()
  const colonyLayer = new PIXI.Graphics()
  const antContainer = new PIXI.ParticleContainer({
    dynamicProperties: {
      position: true,
      rotation: true,
      color: true,
      vertex: false,
      uvs: false,
    },
  })
  world.addChild(terrainSprite, pheromoneSprite, foodLayer, colonyLayer, antContainer)

  // ── ant texture (points along +x, white for tinting) ────────────────
  const antGfx = new PIXI.Graphics()
  antGfx.ellipse(-2.6, 0, 2.4, 1.5).fill(0xffffff) // abdomen
  antGfx.ellipse(0.6, 0, 1.5, 1.1).fill(0xffffff) // thorax
  antGfx.ellipse(3.1, 0, 1.2, 1.0).fill(0xffffff) // head
  const antTexture = app.renderer.generateTexture({
    target: antGfx,
    resolution: 4,
  })
  antGfx.destroy()

  const particles: InstanceType<typeof PIXI.Particle>[] = []

  function ensureParticles(count: number) {
    while (particles.length < count) {
      const p = new PIXI.Particle({
        texture: antTexture,
        anchorX: 0.5,
        anchorY: 0.5,
      })
      p.alpha = 0
      particles.push(p)
      antContainer.addParticle(p)
    }
  }

  // ── world state ──────────────────────────────────────────────────────
  let worldMeta: WorldInit | null = null
  let pheromoneCanvas: HTMLCanvasElement | null = null
  let pheromoneCtx: CanvasRenderingContext2D | null = null
  let pheromoneImage: ImageData | null = null
  let pheromoneTexture: InstanceType<typeof PIXI.Texture> | null = null
  let destroyed = false

  function redrawFood() {
    if (!worldMeta) return
    const buffers = buffersRef.current
    foodLayer.clear()
    for (const f of worldMeta.foods) {
      const amount = buffers.foodAmounts.get(f.id) ?? f.amount
      if (amount < 1) continue
      const radius = 5 + 16 * Math.sqrt(amount / f.maxAmount)
      foodLayer.circle(f.x, f.y, radius * 1.9).fill({ color: 0x2d7a44, alpha: 0.16 })
      foodLayer.circle(f.x, f.y, radius * 1.3).fill({ color: 0x37985a, alpha: 0.35 })
      foodLayer.circle(f.x, f.y, radius).fill(0x49c272)
    }
  }

  function redrawColonies() {
    if (!worldMeta) return
    colonyLayer.clear()
    for (const c of worldMeta.colonies) {
      colonyLayer.circle(c.x, c.y, c.radius * 1.5).fill({ color: 0x8a6a40, alpha: 0.25 })
      colonyLayer.circle(c.x, c.y, c.radius).fill(0x77552e)
      colonyLayer.circle(c.x, c.y, c.radius * 0.62).fill(0x4a3318)
      colonyLayer.circle(c.x, c.y, c.radius * 0.3).fill(0x17100a)
    }
  }

  function updatePheromoneTexture() {
    const buffers = buffersRef.current
    if (
      !worldMeta ||
      !pheromoneCtx ||
      !pheromoneImage ||
      !buffers.pheromoneFood ||
      !buffers.pheromoneHome
    ) {
      return
    }
    const food = buffers.pheromoneFood
    const home = buffers.pheromoneHome
    const data = pheromoneImage.data
    const n = worldMeta.pheromoneW * worldMeta.pheromoneH
    for (let i = 0; i < n; i++) {
      const fv = food[i]
      const hv = home[i]
      const o = i * 4
      // food trails glow green, home field a faint cool blue
      data[o] = hv >> 3
      data[o + 1] = fv
      data[o + 2] = (hv >> 1) + (fv >> 3)
      data[o + 3] = Math.min(255, fv * 1.5 + hv * 0.45)
    }
    pheromoneCtx.putImageData(pheromoneImage, 0, 0)
    pheromoneTexture?.source.update()
  }

  function fitCamera() {
    if (!worldMeta) return
    const scale = Math.min(
      app.screen.width / worldMeta.worldWidth,
      app.screen.height / worldMeta.worldHeight,
    )
    world.scale.set(scale)
    world.position.set(
      (app.screen.width - worldMeta.worldWidth * scale) / 2,
      (app.screen.height - worldMeta.worldHeight * scale) / 2,
    )
  }

  function resetWorld(init: WorldInit) {
    if (destroyed) return
    worldMeta = init

    // terrain
    const terrainCanvas = paintTerrain(init)
    terrainSprite.texture?.destroy(true)
    terrainSprite.texture = PIXI.Texture.from(terrainCanvas)
    terrainSprite.width = init.worldWidth
    terrainSprite.height = init.worldHeight

    // pheromone overlay
    pheromoneCanvas = document.createElement('canvas')
    pheromoneCanvas.width = init.pheromoneW
    pheromoneCanvas.height = init.pheromoneH
    pheromoneCtx = pheromoneCanvas.getContext('2d')
    pheromoneImage = pheromoneCtx
      ? pheromoneCtx.createImageData(init.pheromoneW, init.pheromoneH)
      : null
    pheromoneTexture?.destroy(true)
    pheromoneTexture = PIXI.Texture.from(pheromoneCanvas)
    pheromoneSprite.texture = pheromoneTexture
    pheromoneSprite.width = init.worldWidth
    pheromoneSprite.height = init.worldHeight

    redrawColonies()
    redrawFood()
    for (const p of particles) p.alpha = 0
    fitCamera()
  }

  // ── per-frame update: interpolate ant positions ──────────────────────
  app.ticker.add(() => {
    if (destroyed || !worldMeta) return
    const buffers = buffersRef.current

    if (buffers.pheromoneDirty) {
      buffers.pheromoneDirty = false
      updatePheromoneTexture()
    }
    if (buffers.foodDirty) {
      buffers.foodDirty = false
      redrawFood()
    }

    if (!buffers.hasAnts) return
    const prev = buffers.prev
    const curr = buffers.curr
    const interval = Math.min(Math.max(curr.time - prev.time, 30), 300) || 66
    const t = Math.min(Math.max((performance.now() - curr.time) / interval, 0), 1)

    const count = curr.count
    ensureParticles(count)

    const sx = worldMeta.worldWidth / 65535
    const sy = worldMeta.worldHeight / 65535
    const prevCount = prev.count

    for (let i = 0; i < count; i++) {
      const p = particles[i]
      if (i < prevCount) {
        p.x = (prev.x[i] + (curr.x[i] - prev.x[i]) * t) * sx
        p.y = (prev.y[i] + (curr.y[i] - prev.y[i]) * t) * sy
        // shortest-arc heading interpolation in quantized space
        const dh = ((curr.heading[i] - prev.heading[i] + 384) & 255) - 128
        p.rotation = (prev.heading[i] + dh * t) * HEADING_TO_RAD
      } else {
        p.x = curr.x[i] * sx
        p.y = curr.y[i] * sy
        p.rotation = curr.heading[i] * HEADING_TO_RAD
      }
      p.tint = TINTS[curr.flags[i] & 7]
      p.alpha = 1
    }
    for (let i = count; i < particles.length; i++) {
      particles[i].alpha = 0
    }
  })

  // ── camera: drag to pan, wheel to zoom at cursor ─────────────────────
  let dragging = false
  let lastX = 0
  let lastY = 0

  const canvas = app.canvas
  canvas.style.touchAction = 'none'
  canvas.style.cursor = 'grab'

  const onPointerDown = (e: PointerEvent) => {
    dragging = true
    lastX = e.clientX
    lastY = e.clientY
    canvas.setPointerCapture(e.pointerId)
    canvas.style.cursor = 'grabbing'
  }
  const onPointerMove = (e: PointerEvent) => {
    if (!dragging) return
    world.position.x += e.clientX - lastX
    world.position.y += e.clientY - lastY
    lastX = e.clientX
    lastY = e.clientY
  }
  const onPointerUp = (e: PointerEvent) => {
    dragging = false
    canvas.releasePointerCapture(e.pointerId)
    canvas.style.cursor = 'grab'
  }
  const onWheel = (e: WheelEvent) => {
    e.preventDefault()
    const rect = canvas.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top
    const oldScale = world.scale.x
    const factor = Math.exp(-e.deltaY * 0.0012)
    const newScale = Math.min(Math.max(oldScale * factor, 0.04), 12)
    // keep the world point under the cursor fixed
    world.position.x = mx - ((mx - world.position.x) / oldScale) * newScale
    world.position.y = my - ((my - world.position.y) / oldScale) * newScale
    world.scale.set(newScale)
  }

  canvas.addEventListener('pointerdown', onPointerDown)
  canvas.addEventListener('pointermove', onPointerMove)
  canvas.addEventListener('pointerup', onPointerUp)
  canvas.addEventListener('pointercancel', onPointerUp)
  canvas.addEventListener('wheel', onWheel, { passive: false })

  return {
    resetWorld,
    getFps: () => app.ticker.FPS,
    destroy() {
      destroyed = true
      canvas.removeEventListener('pointerdown', onPointerDown)
      canvas.removeEventListener('pointermove', onPointerMove)
      canvas.removeEventListener('pointerup', onPointerUp)
      canvas.removeEventListener('pointercancel', onPointerUp)
      canvas.removeEventListener('wheel', onWheel)
      app.destroy(true, { children: true, texture: true })
    },
  }
}
