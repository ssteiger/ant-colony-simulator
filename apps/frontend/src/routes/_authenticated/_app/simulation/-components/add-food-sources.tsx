import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { eq, postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { toast } from 'sonner'
import { useParams } from '@tanstack/react-router'

// Server function to add food sources to the simulation
const addFoodSourcesToSimulation = createServerFn({ method: 'POST' })
  .validator((data: { count: number, simulationId: string }) => data)
  .handler(async ({ data }) => {
    try {
      const { simulationId } = data
      // Get the simulation
      const simulations = await postgres_db
        .select()
        .from(schema.simulations)
        .where(eq(schema.simulations.id, simulationId))
        .limit(1)

      if (simulations.length === 0) {
        throw new Error('No simulation found')
      }

      const simulation = simulations[0]

      // Generate random food sources
      const foodSources = []
      const worldWidth = simulation.world_width
      const worldHeight = simulation.world_height

      for (let i = 0; i < data.count; i++) {
        // Generate random position within simulation bounds (convert to integers)
        const position_x = Math.floor(Math.random() * worldWidth)
        const position_y = Math.floor(Math.random() * worldHeight)
        
        // Random food amount (between 50 and 200)
        const amount = Math.floor(Math.random() * 151) + 50

        foodSources.push({
          simulation_id: simulationId,
          position_x: position_x,
          position_y: position_y,
          amount: amount,
          max_amount: amount,
          food_type: 'sugar' as const, // Default food type
          regeneration_rate: 1, // Food regenerates at 1 unit per tick
        })
      }

      // Insert food sources into database
      const insertedFoodSources = await postgres_db
        .insert(schema.food_sources)
        .values(foodSources)
        .returning({ id: schema.food_sources.id })

      return {
        success: true,
        message: `Successfully added ${insertedFoodSources.length} food sources to the simulation`,
        foodSourcesAdded: insertedFoodSources.length
      }
    } catch (error) {
      console.error('Error adding food sources:', error)
      throw new Error(`Failed to add food sources: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  })

export function AddFoodSourcesButton() {
  const [isLoading, setIsLoading] = useState(false)
  const queryClient = useQueryClient()
  const params = useParams({ from: '/_authenticated/_app/simulation/$id' })
  const simulationId = params.id

  const addFoodSourcesMutation = useMutation({
    mutationFn: (count: number) => addFoodSourcesToSimulation({ data: { count, simulationId } }),
    onSuccess: () => {
      // Invalidate and refetch simulation data
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsLoading(false)
      toast.success('10 food sources added')
    },
    onError: () => {
      setIsLoading(false)
      toast.error('Failed to add food sources')
    }
  })

  const handleAddFoodSources = async () => {
    setIsLoading(true)
    try {
      await addFoodSourcesMutation.mutateAsync(10)
    } catch {
      // Error is already handled in onError
    }
  }

  return (
    <Button
      variant="outline"
      size="sm"
      onClick={handleAddFoodSources}
      disabled={isLoading || addFoodSourcesMutation.isPending}
    >
      {isLoading || addFoodSourcesMutation.isPending ? (
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
          Adding Food Sources...
        </span>
      ) : (
        'Add 10 Food Sources'
      )}
    </Button>
  )
}