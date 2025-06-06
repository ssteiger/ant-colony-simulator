import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema } from '@ant-colony-simulator/db-drizzle'

// Server function to add ants to the simulation
const addAntsToSimulation = createServerFn({ method: 'POST' })
  .validator((data: { count: number }) => data)
  .handler(async ({ data }) => {
    try {
      // Get the first active simulation
      const simulations = await postgres_db
        .select()
        .from(schema.simulations)
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

      // Get the first ant type
      const antTypes = await postgres_db
        .select()
        .from(schema.ant_types)
        .limit(1)

      if (antTypes.length === 0) {
        throw new Error('No ant type found')
      }

      const antType = antTypes[0]

      // Generate random ants
      const ants = []
      const worldWidth = simulation.world_width
      const worldHeight = simulation.world_height

      for (let i = 0; i < data.count; i++) {
        // Generate random position within simulation bounds
        const position_x = Math.random() * worldWidth
        const position_y = Math.random() * worldHeight
        
        // Random angle (0 to 2π radians)
        const angle = Math.random() * 2 * Math.PI

        ants.push({
          colony_id: colony.id,
          ant_type_id: antType.id,
          position_x: position_x.toString(),
          position_y: position_y.toString(),
          angle: angle.toString(),
          current_speed: '1.0',
          health: 100,
          age_ticks: 0,
          state: 'wandering' as const,
          energy: 100,
          mood: 'neutral' as const,
        })
      }

      // Insert ants into database
      const insertedAnts = await postgres_db
        .insert(schema.ants)
        .values(ants)
        .returning({ id: schema.ants.id })

      // Note: Colony population will be updated by the database trigger
      // or we can update it in a separate query later to avoid typing issues

      return {
        success: true,
        message: `Successfully added ${insertedAnts.length} ants to ${colony.name}`,
        antsAdded: insertedAnts.length
      }
    } catch (error) {
      console.error('Error adding ants:', error)
      throw new Error(`Failed to add ants: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  })

export function AddAntsButton() {
  const [isLoading, setIsLoading] = useState(false)
  const queryClient = useQueryClient()

  const addAntsMutation = useMutation({
    mutationFn: (count: number) => addAntsToSimulation({ data: { count } }),
    onSuccess: () => {
      // Invalidate and refetch simulation data
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsLoading(false)
    },
    onError: () => {
      setIsLoading(false)
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
    <button
      type="button"
      onClick={handleAddAnts}
      disabled={isLoading || addAntsMutation.isPending}
      className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
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
    </button>
  )
}