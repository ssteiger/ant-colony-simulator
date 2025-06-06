import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema } from '@ant-colony-simulator/db-drizzle'

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
          weather_intensity: '0.0'
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
            base_speed: '1.0',
            base_strength: '1.0',
            base_health: 100,
            base_size: '3.0',
            lifespan_ticks: 50000,
            carrying_capacity: '1.0',
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
          center_x: (data.worldWidth / 2).toString(),
          center_y: (data.worldHeight / 2).toString(),
          radius: '30.0',
          population: 0,
          color_hue: 30,
          nest_level: 1,
          territory_radius: '100.0',
          aggression_level: '0.5',
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
            position_x: Math.max(20, Math.min(x, data.worldWidth - 20)).toString(),
            position_y: Math.max(20, Math.min(y, data.worldHeight - 20)).toString(),
            amount: (50 + Math.random() * 100).toString(),
            max_amount: '150',
            regeneration_rate: '0.1',
            discovery_difficulty: '0.5',
            nutritional_value: '1.0',
            spoilage_rate: '0.001',
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
    name: '',
    description: '',
    worldWidth: 800,
    worldHeight: 600
  })
  const queryClient = useQueryClient()

  const createSimulationMutation = useMutation({
    mutationFn: (data: typeof formData) => createNewSimulation({ data }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsOpen(false)
      setFormData({ name: '', description: '', worldWidth: 800, worldHeight: 600 })
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
      <button
        type="button"
        onClick={() => setIsOpen(true)}
        className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
      >
        New Simulation
      </button>
    )
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md">
        <h3 className="text-lg font-semibold mb-4">Create New Simulation</h3>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-1">
              Simulation Name *
            </label>
            <input
              id="name"
              type="text"
              value={formData.name}
              onChange={(e) => handleInputChange('name', e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="My Ant Colony"
              required
            />
          </div>

          <div>
            <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-1">
              Description
            </label>
            <textarea
              id="description"
              value={formData.description}
              onChange={(e) => handleInputChange('description', e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Optional description..."
              rows={3}
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label htmlFor="worldWidth" className="block text-sm font-medium text-gray-700 mb-1">
                World Width
              </label>
              <input
                id="worldWidth"
                type="number"
                value={formData.worldWidth}
                onChange={(e) => handleInputChange('worldWidth', Number.parseInt(e.target.value) || 800)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                min={400}
                max={2000}
              />
            </div>
            <div>
              <label htmlFor="worldHeight" className="block text-sm font-medium text-gray-700 mb-1">
                World Height
              </label>
              <input
                id="worldHeight"
                type="number"
                value={formData.worldHeight}
                onChange={(e) => handleInputChange('worldHeight', Number.parseInt(e.target.value) || 600)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                min={300}
                max={1500}
              />
            </div>
          </div>

          <div className="flex justify-end gap-3 pt-4">
            <button
              type="button"
              onClick={() => setIsOpen(false)}
              className="px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
              disabled={createSimulationMutation.isPending}
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={createSimulationMutation.isPending || !formData.name.trim()}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {createSimulationMutation.isPending ? (
                <span className="flex items-center gap-2">
                  <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24" aria-label="Loading">
                    <title>Loading spinner</title>
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                      fill="none"
                    />
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  Creating...
                </span>
              ) : (
                'Create Simulation'
              )}
            </button>
          </div>
        </form>

        {createSimulationMutation.error && (
          <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-md">
            <p className="text-sm text-red-600">
              {createSimulationMutation.error.message}
            </p>
          </div>
        )}
      </div>
    </div>
  )
} 