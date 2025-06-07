import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { eq, postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { toast } from 'sonner'
import { useParams } from '@tanstack/react-router'

/**
 * Add Ants Button Component
 * 
 * Implements realistic ant colony distribution based on real-world research:
 * - Workers: ~87% (foraging, building, maintenance - the backbone of the colony)
 * - Soldiers: ~8% (defense, specialized combat tasks)
 * - Scouts: ~3% (exploration, pathfinding, reconnaissance)
 * - Nurses: ~2% (brood care, colony health, larval care)
 * - Queens: ~0.1% (reproduction - typically 1 per colony, but allows for polygynous colonies)
 * 
 * This distribution is based on studies of real ant colonies where workers comprise
 * the vast majority (85-90%), with specialized castes making up smaller percentages.
 */

// Server function to add ants to the simulation
const addAntsToSimulation = createServerFn({ method: 'POST' })
  .validator((data: { count: number, simulationId: string }) => data)
  .handler(async ({ data }) => {
    try {
      const { simulationId } = data
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

      // Realistic ant colony distribution based on research:
      // Workers: ~85-90% (foraging, building, maintenance)
      // Soldiers: ~5-10% (defense, specialized tasks)
      // Scouts: ~3-5% (exploration, pathfinding)
      // Nurses: ~2-5% (brood care, colony health)
      // Queens: ~0.1% (reproduction - usually just 1 per colony)
      
      // Create distribution map
      const distributionMap = new Map<string, number>([
        ['worker', 0.87],    // 87% workers - the backbone of the colony
        ['soldier', 0.08],   // 8% soldiers - defense and heavy lifting
        ['scout', 0.03],     // 3% scouts - exploration and pathfinding
        ['nurse', 0.02],     // 2% nurses - brood care and health
        ['queen', 0.001]     // 0.1% queens - usually just 1, but allowing for multiple queen colonies
      ])

      // Calculate ant counts per type
      const antCounts: { antType: typeof antTypes[0], count: number }[] = []
      let remainingAnts = data.count

      for (const [role, percentage] of distributionMap) {
        const antType = antTypes.find(type => type.role === role)
        if (antType) {
          const count = Math.floor(data.count * percentage)
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
      const worldWidth = simulation.world_width
      const worldHeight = simulation.world_height

      for (const { antType, count } of antCounts) {
        for (let i = 0; i < count; i++) {
          // Generate random position within simulation bounds (convert to integers)
          const position_x = Math.floor(Math.random() * worldWidth)
          const position_y = Math.floor(Math.random() * worldHeight)
          
          // Random angle (0 to 359 degrees as integer)
          const angle = Math.floor(Math.random() * 360)

          ants.push({
            colony_id: colony.id,
            ant_type_id: antType.id,
            position_x: position_x,
            position_y: position_y,
            angle: angle,
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
      const insertedAnts = await postgres_db
        .insert(schema.ants)
        .values(ants)
        .returning({ id: schema.ants.id })

      // Create summary of ant types added
      const summary = antCounts
        .filter(entry => entry.count > 0)
        .map(entry => `${entry.count} ${entry.antType.name}${entry.count > 1 ? 's' : ''}`)
        .join(', ')

      return {
        success: true,
        message: `Successfully added ${insertedAnts.length} ants to ${colony.name}: ${summary}`,
        antsAdded: insertedAnts.length,
        distribution: antCounts.reduce((acc, entry) => {
          acc[entry.antType.name] = entry.count
          return acc
        }, {} as Record<string, number>)
      }
    } catch (error) {
      console.error('Error adding ants:', error)
      throw new Error(`Failed to add ants: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  })

export function AddAntsButton() {
  const [isLoading, setIsLoading] = useState(false)
  const queryClient = useQueryClient()
  const params = useParams({ from: '/_authenticated/_app/simulation/$id' })
  const simulationId = params.id

  const addAntsMutation = useMutation({
    mutationFn: (count: number) => addAntsToSimulation({ data: { count, simulationId } }),
    onSuccess: (data) => {
      // Invalidate and refetch simulation data
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsLoading(false)
      
      // Show detailed success message with ant type breakdown
      const distributionText = Object.entries(data.distribution || {})
        .map(([type, count]) => `${count} ${type}${count > 1 ? 's' : ''}`)
        .join(', ')
      
      toast.success(`Added ${data.antsAdded} ants with realistic distribution: ${distributionText}`)
    },
    onError: () => {
      setIsLoading(false)
      toast.error('Failed to add ants')
    }
  })

  const handleAddAnts = async () => {
    setIsLoading(true)
    try {
      await addAntsMutation.mutateAsync(100)
    } catch {
      // Error is already handled in onError
    }
  }

  return (
    <Button
      variant="outline"
      size="sm"
      onClick={handleAddAnts}
      disabled={isLoading || addAntsMutation.isPending}
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
          Adding Ants...
        </span>
      ) : (
        'Add 100 Ants'
      )}
    </Button>
  )
}