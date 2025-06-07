import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'
import { eq, postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '~/lib/components/ui/dialog'
import { toast } from 'sonner'
import { useParams } from '@tanstack/react-router'

/**
 * Reset Ant Positions Button Component
 * 
 * Allows users to reset the positions of all ants in the colony back to the colony center.
 * This is useful for:
 * - Reorganizing scattered ants
 * - Resetting after simulation issues
 * - Starting fresh formations
 * - Gathering dispersed colony members
 */

// Server function to reset ant positions in the simulation
const resetAntPositions = createServerFn({ method: 'POST' })
  .validator((data: { simulationId: string }) => data)
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

      // Get all colonies in the simulation
      const colonies = await postgres_db
        .select()
        .from(schema.colonies)
        .limit(1)

      if (colonies.length === 0) {
        throw new Error('No colony found')
      }

      const colony = colonies[0]

      // Get all ants in the colony
      const existingAnts = await postgres_db
        .select()
        .from(schema.ants)
        .where(eq(schema.ants.colony_id, colony.id))

      if (existingAnts.length === 0) {
        throw new Error('No ants found in the colony to reset')
      }

      // Reset positions of all ants to near colony center
      const worldWidth = simulation.world_width
      const worldHeight = simulation.world_height
      const colonyX = colony.center_x
      const colonyY = colony.center_y
      const colonyRadius = colony.radius

      const resetPromises = existingAnts.map(async (ant) => {
        // Generate new position near the colony center (within colony radius)
        const angle = Math.random() * 2 * Math.PI
        const distance = Math.random() * colonyRadius
        const position_x = Math.floor(colonyX + Math.cos(angle) * distance)
        const position_y = Math.floor(colonyY + Math.sin(angle) * distance)
        
        // Random facing angle (0 to 359 degrees as integer)
        const facingAngle = Math.floor(Math.random() * 360)

        return postgres_db
          .update(schema.ants)
          .set({
            position_x: Math.max(0, Math.min(position_x, worldWidth - 1)),
            position_y: Math.max(0, Math.min(position_y, worldHeight - 1)),
            angle: facingAngle,
            state: 'wandering' as const,
          })
          .where(eq(schema.ants.id, ant.id))
      })

      await Promise.all(resetPromises)

      return {
        success: true,
        message: `Successfully reset positions of ${existingAnts.length} ants in ${colony.name}`,
        antsReset: existingAnts.length,
        colonyName: colony.name
      }
    } catch (error) {
      console.error('Error resetting ant positions:', error)
      throw new Error(`Failed to reset ant positions: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  })

export function ResetAntPositionsButton() {
  const [isOpen, setIsOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const queryClient = useQueryClient()
  const params = useParams({ from: '/_authenticated/_app/simulation/$id' })
  const simulationId = params.id

  const resetPositionsMutation = useMutation({
    mutationFn: () => resetAntPositions({ data: { simulationId } }),
    onSuccess: (data) => {
      // Invalidate and refetch simulation data
      queryClient.invalidateQueries({ queryKey: ['simulation-data'] })
      setIsLoading(false)
      setIsOpen(false)
      
      toast.success(`Reset positions of ${data.antsReset} ants in ${data.colonyName}`)
    },
    onError: (error) => {
      setIsLoading(false)
      toast.error(error instanceof Error ? error.message : 'Failed to reset ant positions')
    }
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    setIsLoading(true)
    try {
      await resetPositionsMutation.mutateAsync()
    } catch {
      // Error is already handled in onError
    }
  }

  return (
    <>
      <Button
        variant="outline"
        size="sm"
        onClick={() => setIsOpen(true)}
      >
        Reset Ant Positions
      </Button>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Reset Ant Positions</DialogTitle>
            <DialogDescription>
              This will move all ants back to the colony center area. Their positions will be randomized within the colony radius.
            </DialogDescription>
          </DialogHeader>
          
          <div className="py-4">
            <p className="text-sm text-gray-600">
              All ants will be repositioned near the colony center and their state will be reset to "wandering". 
              This action cannot be undone.
            </p>
          </div>

          <DialogFooter>
            <Button 
              variant="outline" 
              onClick={() => setIsOpen(false)} 
              disabled={isLoading || resetPositionsMutation.isPending}
            >
              Cancel
            </Button>
            <Button 
              type="submit" 
              onClick={handleSubmit}
              disabled={isLoading || resetPositionsMutation.isPending}
              variant="destructive"
            >
              {isLoading || resetPositionsMutation.isPending ? (
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
                  Resetting...
                </span>
              ) : (
                'Reset Positions'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}