import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { eq, postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { toast } from 'sonner'
import { useParams, useRouter } from '@tanstack/react-router'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '~/lib/components/ui/alert-dialog'

// Server function to delete a simulation
const deleteSimulation = createServerFn({ method: 'POST' })
  .validator((data: { simulationId: string }) => data)
  .handler(async ({ data }) => {
    try {
      const { simulationId } = data

      // First, delete all related data
      // Delete ants associated with colonies in this simulation
      await postgres_db.execute(`
        DELETE FROM ants 
        WHERE colony_id IN (
          SELECT id FROM colonies WHERE simulation_id = '${simulationId}'
        )
      `)

      // Delete colonies associated with this simulation
      await postgres_db
        .delete(schema.colonies)
        .where(eq(schema.colonies.simulation_id, simulationId))

      // Delete food sources associated with this simulation  
      await postgres_db
        .delete(schema.food_sources)
        .where(eq(schema.food_sources.simulation_id, simulationId))

      // Finally, delete the simulation
      const result = await postgres_db
        .delete(schema.simulations)
        .where(eq(schema.simulations.id, simulationId))
        .returning({ id: schema.simulations.id })

      if (result.length === 0) {
        throw new Error('Simulation not found')
      }

      return {
        success: true,
        message: 'Simulation deleted successfully'
      }
    } catch (error) {
      console.error('Error deleting simulation:', error)
      throw new Error(`Failed to delete simulation: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  })

export function DeleteSimulationButton() {
  const [isLoading, setIsLoading] = useState(false)
  const [open, setOpen] = useState(false)
  const queryClient = useQueryClient()
  const router = useRouter()
  const params = useParams({ from: '/_authenticated/_app/simulation/$id' })
  const simulationId = params.id

  const deleteSimulationMutation = useMutation({
    mutationFn: () => deleteSimulation({ data: { simulationId } }),
    onSuccess: () => {
      // Invalidate queries and redirect to simulations list
      queryClient.invalidateQueries({ queryKey: ['simulations'] })
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsLoading(false)
      setOpen(false)
      toast.success('Simulation deleted successfully')
      // Navigate back to simulations list
      router.navigate({ to: '/dashboard' })
    },
    onError: (error) => {
      setIsLoading(false)
      setOpen(false)
      toast.error(`Failed to delete simulation: ${error.message}`)
    }
  })

  const handleDelete = async () => {
    setIsLoading(true)
    try {
      await deleteSimulationMutation.mutateAsync()
    } catch {
      // Error is already handled in onError
    }
  }

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogTrigger asChild>
        <Button
          variant="destructive"
          size="sm"
          disabled={isLoading || deleteSimulationMutation.isPending}
        >
          {isLoading || deleteSimulationMutation.isPending ? (
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
              Deleting...
            </span>
          ) : (
            'Delete Simulation'
          )}
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete Simulation</AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to delete this simulation? This action cannot be undone.
            This will permanently delete the simulation and all associated data including ants, colonies, and food sources.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            disabled={isLoading || deleteSimulationMutation.isPending}
          >
            {isLoading || deleteSimulationMutation.isPending ? 'Deleting...' : 'Delete'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}