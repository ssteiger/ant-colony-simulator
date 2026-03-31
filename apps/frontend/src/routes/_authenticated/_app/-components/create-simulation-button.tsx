import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
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

const createNewSimulation = createServerFn({ method: 'POST' })
  .validator(
    (data: { name: string; worldWidth: number; worldHeight: number }) => data,
  )
  .handler(async ({ data }) => {
    const [simulation] = await postgres_db
      .insert(schema.simulations)
      .values({
        name: data.name,
        world_width: data.worldWidth,
        world_height: data.worldHeight,
        config: {},
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

export function CreateSimulationButton() {
  const [isOpen, setIsOpen] = useState(false)
  const [formData, setFormData] = useState({
    name: 'My Ant Colony',
    worldWidth: 1200,
    worldHeight: 800,
  })
  const queryClient = useQueryClient()

  const mutation = useMutation({
    mutationFn: (data: typeof formData) => createNewSimulation({ data }),
    onSuccess: () => {
      queryClient.invalidateQueries()
      setIsOpen(false)
      setFormData({ name: 'My Ant Colony', worldWidth: 1200, worldHeight: 800 })
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
              <label
                htmlFor="name"
                className="block text-sm font-medium mb-1"
              >
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
                      worldWidth: Number.parseInt(e.target.value) || 800,
                    }))
                  }
                  min={400}
                  max={4000}
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
                      worldHeight: Number.parseInt(e.target.value) || 600,
                    }))
                  }
                  min={300}
                  max={3000}
                />
              </div>
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
