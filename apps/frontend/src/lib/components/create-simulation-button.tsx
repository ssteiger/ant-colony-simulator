import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { Input } from '~/lib/components/ui/input'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/lib/components/ui/dialog'

interface CreateSimulationInput {
  name: string
  worldWidth: number
  worldHeight: number
  seed: number
  terrainDensity: number
  maxAnts: number
}

const createNewSimulation = createServerFn({ method: 'POST' })
  .validator((data: CreateSimulationInput) => data)
  .handler(async ({ data }) => {
    const [simulation] = await postgres_db
      .insert(schema.simulations)
      .values({
        name: data.name,
        world_width: data.worldWidth,
        world_height: data.worldHeight,
        config: {
          seed: data.seed,
          terrain_density: data.terrainDensity,
          max_ants: data.maxAnts,
        },
        is_active: true,
      })
      .returning()

    await postgres_db.insert(schema.colonies).values({
      simulation_id: simulation.id,
      name: 'Main Colony',
      center_x: Math.floor(data.worldWidth / 2),
      center_y: Math.floor(data.worldHeight / 2),
      color_hue: 30,
    })

    return { success: true, simulationId: simulation.id }
  })

const defaultForm = (): CreateSimulationInput => ({
  name: 'My Ant Colony',
  worldWidth: 4000,
  worldHeight: 3000,
  seed: Math.floor(Math.random() * 1_000_000),
  terrainDensity: 0.32,
  maxAnts: 50000,
})

export function CreateSimulationButton() {
  const [isOpen, setIsOpen] = useState(false)
  const [formData, setFormData] = useState<CreateSimulationInput>(defaultForm)
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const mutation = useMutation({
    mutationFn: (data: CreateSimulationInput) => createNewSimulation({ data }),
    onSuccess: (result) => {
      queryClient.invalidateQueries()
      setIsOpen(false)
      setFormData(defaultForm())
      navigate({
        to: '/simulation/$id',
        params: { id: String(result.simulationId) },
      })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.name.trim()) return
    mutation.mutate(formData)
  }

  if (!isOpen) {
    return (
      <Button onClick={() => setIsOpen(true)} variant="default" size="sm">
        New Simulation
      </Button>
    )
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create New Simulation</DialogTitle>
        </DialogHeader>
        <DialogDescription asChild>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="name" className="block text-sm font-medium mb-1">
                Name
              </label>
              <Input
                id="name"
                type="text"
                value={formData.name}
                onChange={(e) =>
                  setFormData((f) => ({ ...f, name: e.target.value }))
                }
                required
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label
                  htmlFor="worldWidth"
                  className="block text-sm font-medium mb-1"
                >
                  Width
                </label>
                <Input
                  id="worldWidth"
                  type="number"
                  value={formData.worldWidth}
                  onChange={(e) =>
                    setFormData((f) => ({
                      ...f,
                      worldWidth: Number.parseInt(e.target.value) || 4000,
                    }))
                  }
                  min={800}
                  max={8000}
                />
              </div>
              <div>
                <label
                  htmlFor="worldHeight"
                  className="block text-sm font-medium mb-1"
                >
                  Height
                </label>
                <Input
                  id="worldHeight"
                  type="number"
                  value={formData.worldHeight}
                  onChange={(e) =>
                    setFormData((f) => ({
                      ...f,
                      worldHeight: Number.parseInt(e.target.value) || 3000,
                    }))
                  }
                  min={600}
                  max={6000}
                />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label
                  htmlFor="seed"
                  className="block text-sm font-medium mb-1"
                >
                  Terrain seed
                </label>
                <Input
                  id="seed"
                  type="number"
                  value={formData.seed}
                  onChange={(e) =>
                    setFormData((f) => ({
                      ...f,
                      seed: Number.parseInt(e.target.value) || 0,
                    }))
                  }
                  min={0}
                />
              </div>
              <div>
                <label
                  htmlFor="terrainDensity"
                  className="block text-sm font-medium mb-1"
                >
                  Rock density
                </label>
                <Input
                  id="terrainDensity"
                  type="number"
                  step={0.05}
                  value={formData.terrainDensity}
                  onChange={(e) =>
                    setFormData((f) => ({
                      ...f,
                      terrainDensity: Number.parseFloat(e.target.value) || 0,
                    }))
                  }
                  min={0}
                  max={0.6}
                />
              </div>
            </div>
            <div>
              <label
                htmlFor="maxAnts"
                className="block text-sm font-medium mb-1"
              >
                Max ants
              </label>
              <Input
                id="maxAnts"
                type="number"
                value={formData.maxAnts}
                onChange={(e) =>
                  setFormData((f) => ({
                    ...f,
                    maxAnts: Number.parseInt(e.target.value) || 50000,
                  }))
                }
                min={100}
                max={200000}
              />
            </div>
          </form>
        </DialogDescription>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setIsOpen(false)}
            disabled={mutation.isPending}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            onClick={handleSubmit}
            disabled={mutation.isPending || !formData.name.trim()}
          >
            {mutation.isPending ? 'Creating...' : 'Create'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
