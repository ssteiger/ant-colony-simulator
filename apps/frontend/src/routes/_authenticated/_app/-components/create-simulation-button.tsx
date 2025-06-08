import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { Input } from '~/lib/components/ui/input'
import { Textarea } from '~/lib/components/ui/textarea'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '~/lib/components/ui/dialog'

/**
 * Create Simulation Button Component
 * 
 * Creates a new ant colony simulation with:
 * - Customizable world dimensions
 * - Default colony placed at the center
 * - Initial food sources scattered around the colony
 * - Realistic ant distribution based on real-world research:
 *   - Workers: ~87% (foraging, building, maintenance)
 *   - Soldiers: ~8% (defense, specialized combat tasks)
 *   - Scouts: ~3% (exploration, pathfinding, reconnaissance)
 *   - Nurses: ~2% (brood care, colony health, larval care)
 *   - Queens: ~0.1% (reproduction - typically 1 per colony)
 * 
 * Ants are positioned near the colony center within the colony radius for a realistic start.
 */

// Server function to create a new simulation
const createNewSimulation = createServerFn({ method: 'POST' })
  .validator((data: { 
    name: string
    description?: string
    worldWidth: number
    worldHeight: number
    initialAntCount?: number
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

      // Create a default colony in the center
      const [colony] = await postgres_db
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
        .returning()

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

      // Create initial ants with realistic distribution
      const initialAntCount = data.initialAntCount || 50 // Default to 50 ants
      
      if (false) {
      //if (initialAntCount > 0) {
        // Get all available ant types
        const antTypes = await postgres_db
          .select()
          .from(schema.ant_types)

        if (antTypes.length > 0) {
          // Realistic ant colony distribution based on research:
          // Workers: ~87% (foraging, building, maintenance)
          // Soldiers: ~8% (defense, specialized tasks)
          // Scouts: ~3% (exploration, pathfinding)
          // Nurses: ~2% (brood care, colony health)
          // Queens: ~0.1% (reproduction - usually just 1 per colony)
          
          const distributionMap = new Map<string, number>([
            ['worker', 0.87],    // 87% workers - the backbone of the colony
            ['soldier', 0.08],   // 8% soldiers - defense and heavy lifting
            ['scout', 0.03],     // 3% scouts - exploration and pathfinding
            ['nurse', 0.02],     // 2% nurses - brood care and health
            ['queen', 0.001]     // 0.1% queens - usually just 1, but allowing for multiple queen colonies
          ])

          // Calculate ant counts per type
          const antCounts: { antType: typeof antTypes[0], count: number }[] = []
          let remainingAnts = initialAntCount

          for (const [role, percentage] of distributionMap) {
            const antType = antTypes.find(type => type.role === role)
            if (antType) {
              const count = Math.floor(initialAntCount * percentage)
              if (count > 0) {
                antCounts.push({ antType, count })
                remainingAnts -= count
              }
            }
          }

          // Add any remaining ants as workers (most common type)
          if (remainingAnts > 0) {
            const workerType = antTypes.find(type => type.role === 'worker')
            if (workerType) {
              const existingWorkerEntry = antCounts.find(entry => entry.antType.role === 'worker')
              if (existingWorkerEntry) {
                existingWorkerEntry.count += remainingAnts
              } else {
                antCounts.push({ antType: workerType, count: remainingAnts })
              }
            }
          }

          // Generate ants based on distribution
          const ants = []
          const colonyX = colony.center_x
          const colonyY = colony.center_y
          const colonyRadius = colony.radius

          for (const { antType, count } of antCounts) {
            for (let i = 0; i < count; i++) {
              // Generate position near the colony center (within colony radius)
              const angle = Math.random() * 2 * Math.PI
              const distance = Math.random() * colonyRadius
              const position_x = Math.floor(colonyX + Math.cos(angle) * distance)
              const position_y = Math.floor(colonyY + Math.sin(angle) * distance)
              
              // Random facing angle (0 to 359 degrees as integer)
              const facingAngle = Math.floor(Math.random() * 360)

              ants.push({
                colony_id: colony.id,
                ant_type_id: antType.id,
                position_x: Math.max(0, Math.min(position_x, data.worldWidth - 1)),
                position_y: Math.max(0, Math.min(position_y, data.worldHeight - 1)),
                angle: facingAngle,
                current_speed: 1,
                health: antType.base_health || 100,
                age_ticks: 0,
                state: 'wandering' as const,
                energy: 100,
                mood: 'neutral' as const,
              })
            }
          }

          // Insert ants into database
          if (ants.length > 0) {
            await postgres_db
              .insert(schema.ants)
              .values(ants)
          }

          // Create summary of ant types created
          const antSummary = antCounts
            .filter(entry => entry.count > 0)
            .map(entry => `${entry.count} ${entry.antType.name}${entry.count > 1 ? 's' : ''}`)
            .join(', ')

          return {
            success: true,
            message: `Successfully created simulation "${data.name}" with colony, food sources, and ${ants.length} ants (${antSummary})`,
            antsCreated: ants.length
          }
        }
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
    worldHeight: 600,
    initialAntCount: 50
  })
  const queryClient = useQueryClient()

  const createSimulationMutation = useMutation({
    mutationFn: (data: typeof formData) => createNewSimulation({ data }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsOpen(false)
      setFormData({ name: 'My Ant Colony', description: 'A new ant colony simulation with worker ants exploring for food sources.', worldWidth: 800, worldHeight: 600, initialAntCount: 50 })
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

            <div>
              <label htmlFor="initialAntCount" className="block text-sm font-medium text-gray-700 mb-1">
                Initial Ant Count
              </label>
              <Input
                id="initialAntCount"
                type="number"
                value={formData.initialAntCount}
                onChange={(e) => handleInputChange('initialAntCount', Number.parseInt(e.target.value) || 0)}
                min={0}
                max={500}
                placeholder="50"
              />
              <p className="text-xs text-gray-500 mt-1">
                Number of ants to start with (realistic distribution: ~87% workers, 8% soldiers, 3% scouts, 2% nurses, 0.1% queens)
              </p>
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