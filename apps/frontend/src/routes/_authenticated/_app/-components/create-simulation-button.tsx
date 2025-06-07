import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { Input } from '~/lib/components/ui/input'
import { Textarea } from '~/lib/components/ui/textarea'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '~/lib/components/ui/dialog'

// Server function to create a new simulation
const createNewSimulation = createServerFn({ method: 'POST' })
  .validator((data: { 
    name: string
    description?: string
    worldWidth: number
    worldHeight: number
  }) => data)
  .handler(async ({ data }) => {
    try {
      // Create the simulation
      const [simulation] = await postgres_db
        .insert(schema.simulations)
        .values({
          name: data.name,
          description: data.description || '',
          world_width: data.worldWidth,
          world_height: data.worldHeight,
          is_active: true,
          current_tick: 0,
          season: 'spring',
          time_of_day: 720,
          weather_type: 'clear',
          weather_intensity: 0
        })
        .returning()

      // Create a default ant type if none exists
      const existingAntTypes = await postgres_db
        .select()
        .from(schema.ant_types)
        .limit(1)

      if (existingAntTypes.length === 0) {
        await postgres_db
          .insert(schema.ant_types)
          .values({
            name: 'Worker Ant',
            base_speed: 1,
            base_strength: 1,
            base_health: 100,
            base_size: 3,
            lifespan_ticks: 50000,
            carrying_capacity: 1,
            role: 'worker',
            color_hue: 30
          })
      }

      // Create a default colony in the center
      await postgres_db
        .insert(schema.colonies)
        .values({
          simulation_id: simulation.id,
          name: 'Main Colony',
          center_x: Math.floor(data.worldWidth / 2),
          center_y: Math.floor(data.worldHeight / 2),
          radius: 30,
          population: 0,
          color_hue: 30,
          nest_level: 1,
          territory_radius: 100,
          aggression_level: 1,
          is_active: true
        })

      // Create some initial food sources
      for (let i = 0; i < 5; i++) {
        const angle = (i / 5) * 2 * Math.PI
        const distance = 150 + Math.random() * 100
        const x = (data.worldWidth / 2) + Math.cos(angle) * distance
        const y = (data.worldHeight / 2) + Math.sin(angle) * distance
        
        await postgres_db
          .insert(schema.food_sources)
          .values({
            simulation_id: simulation.id,
            food_type: ['seeds', 'sugar', 'protein'][Math.floor(Math.random() * 3)],
            position_x: Math.floor(Math.max(20, Math.min(x, data.worldWidth - 20))),
            position_y: Math.floor(Math.max(20, Math.min(y, data.worldHeight - 20))),
            amount: Math.floor(50 + Math.random() * 100),
            max_amount: 150,
            regeneration_rate: 0,
            discovery_difficulty: 1,
            nutritional_value: 1,
            spoilage_rate: 0,
            is_renewable: true
          })
      }

      return {
        success: true,
        message: `Successfully created simulation "${data.name}" with initial colony and food sources`
      }
    } catch (error) {
      console.error('Error creating simulation:', error)
      throw new Error(`Failed to create simulation: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  })

export function CreateSimulationButton() {
  const [isOpen, setIsOpen] = useState(false)
  const [formData, setFormData] = useState({
    name: 'My Ant Colony',
    description: 'A new ant colony simulation with worker ants exploring for food sources.',
    worldWidth: 800,
    worldHeight: 600
  })
  const queryClient = useQueryClient()

  const createSimulationMutation = useMutation({
    mutationFn: (data: typeof formData) => createNewSimulation({ data }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsOpen(false)
      setFormData({ name: 'My Ant Colony', description: 'A new ant colony simulation with worker ants exploring for food sources.', worldWidth: 800, worldHeight: 600 })
      queryClient.invalidateQueries()
    }
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.name.trim()) return
    
    createSimulationMutation.mutate(formData)
  }

  const handleInputChange = (field: keyof typeof formData, value: string | number) => {
    setFormData(prev => ({ ...prev, [field]: value }))
  }


  if (!isOpen) {
    return (
      <Button
        onClick={() => setIsOpen(true)}
        variant="default"
        size="sm"
      >
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
        <DialogDescription>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-1">
                Simulation Name *
              </label>
              <Input
                id="name"
                type="text"
                value={formData.name}
                onChange={(e) => handleInputChange('name', e.target.value)}
                placeholder="My Ant Colony"
                required
              />
            </div>

            <div>
              <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-1">
                Description
              </label>
              <Textarea
                id="description"
                value={formData.description}
                onChange={(e) => handleInputChange('description', e.target.value)}
                placeholder="Optional description..."
                rows={3}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label htmlFor="worldWidth" className="block text-sm font-medium text-gray-700 mb-1">
                  World Width
                </label>
                <Input
                  id="worldWidth"
                  type="number"
                  value={formData.worldWidth}
                  onChange={(e) => handleInputChange('worldWidth', Number.parseInt(e.target.value) || 100)}
                  min={10}
                  max={200}
                />
              </div>
              <div>
                <label htmlFor="worldHeight" className="block text-sm font-medium text-gray-700 mb-1">
                  World Height
                </label>
                <Input
                  id="worldHeight"
                  type="number"
                  value={formData.worldHeight}
                  onChange={(e) => handleInputChange('worldHeight', Number.parseInt(e.target.value) || 100)}
                  min={10}
                  max={200}
                />
              </div>
            </div>
          </form>
        </DialogDescription>
        <DialogFooter>
          <Button variant="outline" onClick={() => setIsOpen(false)} disabled={createSimulationMutation.isPending}>
            Cancel
          </Button>
          <Button 
            type="submit" 
            onClick={handleSubmit}
            disabled={createSimulationMutation.isPending || !formData.name.trim()}
          >
            {createSimulationMutation.isPending ? 'Creating...' : 'Create Simulation'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
} 