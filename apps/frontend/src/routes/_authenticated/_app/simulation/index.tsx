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
    <div className="group rounded-lg border bg-card p-5 shadow-sm transition-shadow hover:shadow-md">
      <div className="flex items-start justify-between mb-2">
        <h3 className="font-semibold">{simulation.name}</h3>
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

      <p className="text-sm text-muted-foreground mb-3">
        {simulation.world_width} x {simulation.world_height}
      </p>

      {simulation.created_at && (
        <p className="text-xs text-muted-foreground mb-4">
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
        className="inline-flex w-full items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
      >
        Open
      </Link>
    </div>
  )
}

function SimulationListPage() {
  const {
    data: simulations,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['all-simulations'],
    queryFn: () => getAllSimulations(),
  })

  return (
    <div className="flex-1 space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Simulations</h1>
          <p className="text-muted-foreground text-sm mt-1">
            Create and manage ant colony simulations
          </p>
        </div>
        <div className="flex gap-2">
          <Button type="button" variant="outline" size="sm" onClick={() => refetch()}>
            Refresh
          </Button>
          <CreateSimulationButton />
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 p-4">
          <p className="text-destructive text-sm">Error loading simulations.</p>
        </div>
      )}

      {isLoading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
          {['skel-a', 'skel-b', 'skel-c'].map((key) => (
            <div key={key} className="rounded-lg border bg-card p-5 animate-pulse">
              <div className="h-4 bg-muted rounded w-2/3 mb-3" />
              <div className="h-3 bg-muted rounded w-1/3 mb-3" />
              <div className="h-9 bg-muted rounded" />
            </div>
          ))}
        </div>
      ) : simulations && simulations.length > 0 ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
          {simulations.map((sim) => (
            <SimulationCard key={sim.id} simulation={sim} />
          ))}
        </div>
      ) : (
        <div className="text-center py-16">
          <h3 className="text-sm font-medium">No simulations yet</h3>
          <p className="mt-1 text-sm text-muted-foreground">
            Create your first ant colony simulation.
          </p>
          <div className="mt-6">
            <CreateSimulationButton />
          </div>
        </div>
      )}
    </div>
  )
}

export const Route = createFileRoute('/_authenticated/_app/simulation/')({
  component: SimulationListPage,
})
