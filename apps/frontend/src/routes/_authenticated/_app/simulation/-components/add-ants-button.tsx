import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { eq, postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { Input } from '~/lib/components/ui/input'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '~/lib/components/ui/dialog'
import { toast } from 'sonner'
import { useParams } from '@tanstack/react-router'

/**
 * Add Ants Button Component
 * 
 * Allows users to add custom numbers of each ant type to the simulation:
 * - Workers: foraging, building, maintenance - the backbone of the colony
 * - Soldiers: defense, specialized combat tasks
 * - Scouts: exploration, pathfinding, reconnaissance
 * - Nurses: brood care, colony health, larval care
 * - Queens: reproduction - typically 1 per colony, but allows for polygynous colonies
 * 
 * Users can specify exact counts for each ant type through a modal form.
 */

// Server function to get available ant types
const getAntTypes = createServerFn({ method: 'GET' })
  .handler(async () => {
    try {
      const antTypes = await postgres_db
        .select()
        .from(schema.ant_types)
        .orderBy(schema.ant_types.role)

      return antTypes as Array<{
        id: number
        name: string
        base_speed: number
        base_strength: number
        base_health: number
        base_size: number
        lifespan_ticks: number
        carrying_capacity: number
        role: string
        color_hue: number
        special_abilities: {}
        food_preferences: {}
      }>
    } catch (error) {
      console.error('Error fetching ant types:', error)
      throw new Error(`Failed to fetch ant types: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  })

// Server function to add ants to the simulation
const addAntsToSimulation = createServerFn({ method: 'POST' })
  .validator((data: { antCounts: Record<string, number>, simulationId: string }) => data)
  .handler(async ({ data }) => {
    try {
      const { simulationId, antCounts } = data
      
      // Get the first active simulation
      const simulations = await postgres_db
        .select()
        .from(schema.simulations)
        .where(eq(schema.simulations.id, simulationId))
        .limit(1)

      if (simulations.length === 0) {
        throw new Error('No simulation found')
      }

      const simulation = simulations[0]

      // Get the first colony to add ants to
      const colonies = await postgres_db
        .select()
        .from(schema.colonies)
        .limit(1)

      if (colonies.length === 0) {
        throw new Error('No colony found to add ants to')
      }

      const colony = colonies[0]

      // Get all available ant types
      const antTypes = await postgres_db
        .select()
        .from(schema.ant_types)

      if (antTypes.length === 0) {
        throw new Error('No ant types found')
      }

      // Generate ants based on user-specified counts
      const ants = []
      const worldWidth = simulation.world_width
      const worldHeight = simulation.world_height
      const colonyX = colony.center_x
      const colonyY = colony.center_y
      const colonyRadius = colony.radius

      for (const [antTypeId, count] of Object.entries(antCounts)) {
        if (count > 0) {
          const antType = antTypes.find(type => type.id === Number.parseInt(antTypeId))
          if (antType) {
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
                position_x: Math.max(0, Math.min(position_x, worldWidth - 1)),
                position_y: Math.max(0, Math.min(position_y, worldHeight - 1)),
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
        }
      }

      if (ants.length === 0) {
        throw new Error('No ants to add - please specify at least one ant')
      }

      // Insert ants into database
      const insertedAnts = await postgres_db
        .insert(schema.ants)
        .values(ants)
        .returning({ id: schema.ants.id })

      // Create summary of ant types added
      const summary = Object.entries(antCounts)
        .filter(([, count]) => count > 0)
        .map(([antTypeId, count]) => {
          const antType = antTypes.find(type => type.id === Number.parseInt(antTypeId))
          return `${count} ${antType?.name || 'Unknown'}${count > 1 ? 's' : ''}`
        })
        .join(', ')

      return {
        success: true,
        message: `Successfully added ${insertedAnts.length} ants to ${colony.name}: ${summary}`,
        antsAdded: insertedAnts.length,
        distribution: Object.fromEntries(
          Object.entries(antCounts)
            .filter(([, count]) => count > 0)
            .map(([antTypeId, count]) => {
              const antType = antTypes.find(type => type.id === Number.parseInt(antTypeId))
              return [antType?.name || 'Unknown', count]
            })
        )
      }
    } catch (error) {
      console.error('Error adding ants:', error)
      throw new Error(`Failed to add ants: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  })

export function AddAntsButton() {
  const [isOpen, setIsOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [antCounts, setAntCounts] = useState<Record<string, number>>({})
  const [antTypes, setAntTypes] = useState<Array<{
    id: number
    name: string
    role: string
    description?: string
    base_health?: number
  }>>([])
  const queryClient = useQueryClient()
  const params = useParams({ from: '/_authenticated/_app/simulation/$id' })
  const simulationId = params.id

  // Fetch ant types when modal opens
  const fetchAntTypes = async () => {
    try {
      const types = await getAntTypes()
      setAntTypes(types)
      // Initialize ant counts with realistic defaults
      const defaultCounts: Record<string, number> = {}
      for (const type of types) {
        switch (type.role) {
          case 'worker':
            defaultCounts[type.id.toString()] = 0
            break
          case 'soldier':
            defaultCounts[type.id.toString()] = 0
            break
          case 'scout':
            defaultCounts[type.id.toString()] = 0
            break
          case 'nurse':
            defaultCounts[type.id.toString()] = 0
            break
          case 'queen':
            defaultCounts[type.id.toString()] = 0
            break
          default:
            defaultCounts[type.id.toString()] = 0
        }
      }
      setAntCounts(defaultCounts)
    } catch {
      toast.error('Failed to load ant types')
    }
  }

  const addAntsMutation = useMutation({
    mutationFn: (antCounts: Record<string, number>) => addAntsToSimulation({ data: { antCounts, simulationId } }),
    onSuccess: (data) => {
      // Invalidate and refetch simulation data
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsLoading(false)
      setIsOpen(false)
      
      // Show detailed success message with ant type breakdown
      const distributionText = Object.entries(data.distribution || {})
        .map(([type, count]) => `${count} ${type}${count > 1 ? 's' : ''}`)
        .join(', ')
      
      toast.success(`Added ${data.antsAdded} ants: ${distributionText}`)
    },
    onError: () => {
      setIsLoading(false)
      toast.error('Failed to add ants')
    }
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    // Check if at least one ant is specified
    const totalAnts = Object.values(antCounts).reduce((sum, count) => sum + count, 0)
    if (totalAnts === 0) {
      toast.error('Please specify at least one ant to add')
      return
    }

    setIsLoading(true)
    try {
      await addAntsMutation.mutateAsync(antCounts)
    } catch {
      // Error is already handled in onError
    }
  }

  const handleInputChange = (antTypeId: string, value: number) => {
    setAntCounts(prev => ({ ...prev, [antTypeId]: Math.max(0, value) }))
  }

  const handleOpenModal = async () => {
    setIsOpen(true)
    await fetchAntTypes()
  }

  const getTotalAnts = () => {
    return Object.values(antCounts).reduce((sum, count) => sum + count, 0)
  }

  return (
    <>
      <Button
        variant="outline"
        size="sm"
        onClick={handleOpenModal}
      >
        Add Ants
      </Button>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Add Ants to Colony</DialogTitle>
            <DialogDescription>
              Specify how many ants of each type to add to your colony.
            </DialogDescription>
          </DialogHeader>
          
          <form onSubmit={handleSubmit} className="space-y-4">
            {antTypes.map((antType) => (
              <div key={antType.id} className="flex justify-between items-center">
                <div className="flex-1">
                  <label htmlFor={`ant-${antType.id}`} className="block text-sm font-medium text-gray-700 capitalize">
                    {antType.name}
                  </label>
                  <p className="text-xs text-gray-500">{antType.description}</p>
                </div>
                <div className="w-20 ml-4">
                  <Input
                    id={`ant-${antType.id}`}
                    type="number"
                    min="0"
                    max="1000"
                    value={antCounts[antType.id.toString()] || 0}
                    onChange={(e) => handleInputChange(antType.id.toString(), Number.parseInt(e.target.value) || 0)}
                    className="text-center"
                  />
                </div>
              </div>
            ))}
            
            <div className="pt-4 border-t">
              <div className="flex justify-between items-center text-sm font-medium">
                <span>Total Ants:</span>
                <span>{getTotalAnts()}</span>
              </div>
            </div>
          </form>

          <DialogFooter>
            <Button 
              variant="outline" 
              onClick={() => setIsOpen(false)} 
              disabled={isLoading || addAntsMutation.isPending}
            >
              Cancel
            </Button>
            <Button 
              type="submit" 
              onClick={handleSubmit}
              disabled={isLoading || addAntsMutation.isPending || getTotalAnts() === 0}
            >
              {isLoading || addAntsMutation.isPending ? (
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
                  Adding...
                </span>
              ) : (
                `Add ${getTotalAnts()} Ants`
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}