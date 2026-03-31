import { useQuery } from '@tanstack/react-query'
import { createFileRoute, Link } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { CreateSimulationButton } from '../-components/create-simulation-button'

const getAllSimulations = createServerFn({ method: 'GET' }).handler(
  async () => {
    try {
      return await postgres_db
        .select()
        .from(schema.simulations)
        .orderBy(schema.simulations.created_at)
    } catch (error) {
      console.error('Database error:', error)
      return []
    }
  },
)

function SimulationCard({
  simulation,
}: {
  simulation: {
    id: number
    name: string
    world_width: number
    world_height: number
    is_active: boolean | null
    created_at: string | null
  }
}) {
  return (
    <div className="rounded-lg border bg-card p-6 shadow-sm hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between mb-3">
        <h3 className="text-lg font-semibold">{simulation.name}</h3>
        <span
          className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${
            simulation.is_active
              ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
              : 'bg-muted text-muted-foreground'
          }`}
        >
          {simulation.is_active ? 'Active' : 'Inactive'}
        </span>
      </div>

      <p className="text-sm text-muted-foreground mb-4">
        {simulation.world_width} x {simulation.world_height} world
      </p>

      {simulation.created_at && (
        <p className="text-xs text-muted-foreground mb-4">
          Created{' '}
          {new Date(simulation.created_at).toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric',
          })}
        </p>
      )}

      <Link
        to="/simulation/$id"
        params={{ id: String(simulation.id) }}
        className="inline-flex w-full items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
      >
        Open Simulation
      </Link>
    </div>
  )
}

function RouteComponent() {
  const {
    data: simulations,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['all-simulations'],
    queryFn: () => getAllSimulations(),
    refetchInterval: 30000,
  })

  return (
    <div className="flex-1 space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Dashboard</h1>
          <p className="text-muted-foreground mt-1">
            Manage your ant colony simulations
          </p>
        </div>
        <div className="flex gap-2">
          <Button type="button" variant="outline" onClick={() => refetch()}>
            Refresh
          </Button>
          <CreateSimulationButton />
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 p-4 dark:bg-red-900/20 dark:border-red-800">
          <p className="text-red-800 dark:text-red-400">
            Error loading simulations.
          </p>
        </div>
      )}

      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {Array.from({ length: 3 }, (_, i) => (
            <div
              key={`skel-${i}`}
              className="rounded-lg border bg-card p-6 animate-pulse"
            >
              <div className="h-4 bg-muted rounded w-3/4 mb-4" />
              <div className="h-3 bg-muted rounded w-1/2 mb-4" />
              <div className="h-9 bg-muted rounded" />
            </div>
          ))}
        </div>
      ) : simulations && simulations.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {simulations.map((sim) => (
            <SimulationCard key={sim.id} simulation={sim} />
          ))}
        </div>
      ) : (
        <div className="text-center py-16">
          <h3 className="text-sm font-medium">No simulations yet</h3>
          <p className="mt-1 text-sm text-muted-foreground">
            Create your first ant colony simulation to get started.
          </p>
          <div className="mt-6">
            <CreateSimulationButton />
          </div>
        </div>
      )}
    </div>
  )
}

export const Route = createFileRoute('/_authenticated/_app/dashboard/')({
  component: RouteComponent,
})
