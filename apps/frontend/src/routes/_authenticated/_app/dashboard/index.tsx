import { useQuery } from '@tanstack/react-query'
import { createFileRoute, Link } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema } from '@ant-colony-simulator/db-drizzle'
import { Button } from '~/lib/components/ui/button'
import { CreateSimulationButton } from '../-components/create-simulation-button'
import type { Simulation } from '~/types/drizzle'

const getAllSimulations = createServerFn({ method: 'GET' })
  .handler(async () => {
    try {
      const simulations = await postgres_db
        .select({
          id: schema.simulations.id,
          name: schema.simulations.name,
          description: schema.simulations.description,
          world_width: schema.simulations.world_width,
          world_height: schema.simulations.world_height,
          current_tick: schema.simulations.current_tick,
          is_active: schema.simulations.is_active,
          season: schema.simulations.season,
          weather_type: schema.simulations.weather_type,
          created_at: schema.simulations.created_at,
          updated_at: schema.simulations.updated_at
        })
        .from(schema.simulations)
        .orderBy(schema.simulations.updated_at)

      // Ensure dates are never null by providing fallback values
      return simulations.map(sim => ({
        ...sim,
        created_at: sim.created_at || new Date().toISOString(),
        updated_at: sim.updated_at || new Date().toISOString()
      }))
    } catch (error) {
      console.error('Database error:', error)
      return []
    }
  })

const SimulationCard = ({ simulation }: { simulation: Simulation }) => {
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  return (
    <div className="bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow duration-200">
      <div className="p-6">
        <div className="flex items-start justify-between mb-4">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-1">
              {simulation.name}
            </h3>
            <div className="flex items-center gap-2">
              <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                simulation.is_active 
                  ? 'bg-green-100 text-green-800' 
                  : 'bg-gray-100 text-gray-800'
              }`}>
                {simulation.is_active ? 'Active' : 'Inactive'}
              </span>
              <span className="text-xs text-gray-500">
                Tick: {(simulation.current_tick || 0).toLocaleString()}
              </span>
            </div>
          </div>
        </div>

        {simulation.description && (
          <p className="text-sm text-gray-600 mb-4 line-clamp-2">
            {simulation.description}
          </p>
        )}

        <div className="grid grid-cols-2 gap-4 mb-4 text-sm">
          <div>
            <span className="text-gray-500">World Size:</span>
            <p className="font-medium">{simulation.world_width} × {simulation.world_height}</p>
          </div>
          <div>
            <span className="text-gray-500">Environment:</span>
            <p className="font-medium capitalize">
              {simulation.season} • {simulation.weather_type}
            </p>
          </div>
        </div>

        <div className="text-xs text-gray-500 mb-4">
          <p>Created: {formatDate(simulation.created_at)}</p>
          <p>Updated: {formatDate(simulation.updated_at)}</p>
        </div>

        <div className="flex gap-2">
          <Link
            to="/simulation/$id"
            params={{ id: simulation.id }}
            className="flex-1 bg-black text-white text-sm px-4 py-2 rounded-md hover:bg-gray-800 transition-colors text-center"
          >
            View Simulation
          </Link>
        </div>
      </div>
    </div>
  )
}

function RouteComponent() {
  const {
    data: simulations,
    isLoading,
    error,
    refetch
  } = useQuery({
    queryKey: ['all-simulations'],
    queryFn: () => getAllSimulations(),
    refetchInterval: 30000, // Refresh every 30 seconds
  })

  return (
    <div className="flex-1 space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
          <p className="text-gray-600 mt-1">
            Manage and monitor your ant colony simulations
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            type="button"
            onClick={() => refetch()}
          >
            Refresh
          </Button>
          <CreateSimulationButton />
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-md p-4">
          <p className="text-red-800">
            Error loading simulations. Please try again.
          </p>
        </div>
      )}

      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {Array.from({ length: 6 }, (_, i) => `skeleton-${i}`).map((key) => (
            <div key={key} className="bg-white border border-gray-200 rounded-lg p-6 animate-pulse">
              <div className="h-4 bg-gray-200 rounded w-3/4 mb-4" />
              <div className="h-3 bg-gray-200 rounded w-1/2 mb-2" />
              <div className="h-3 bg-gray-200 rounded w-2/3 mb-4" />
              <div className="grid grid-cols-2 gap-4 mb-4">
                <div className="h-3 bg-gray-200 rounded" />
                <div className="h-3 bg-gray-200 rounded" />
              </div>
              <div className="h-8 bg-gray-200 rounded" />
            </div>
          ))}
        </div>
      ) : simulations && simulations.length > 0 ? (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {simulations.map((simulation) => (
              <SimulationCard key={simulation.id} simulation={simulation} />
            ))}
          </div>
          
          <div className="text-center text-gray-500 mt-8">
            <p>{simulations.length} simulation{simulations.length === 1 ? '' : 's'} found</p>
          </div>
        </>
      ) : (
        <div className="text-center py-12">
          <div className="max-w-md mx-auto">
            <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-label="Empty state illustration">
              <title>No simulations found</title>
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">No simulations</h3>
            <p className="mt-1 text-sm text-gray-500">
              Get started by creating your first ant colony simulation.
            </p>
            <div className="mt-6">
            <CreateSimulationButton />
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export const Route = createFileRoute('/_authenticated/_app/dashboard/')({
  component: RouteComponent,
})
