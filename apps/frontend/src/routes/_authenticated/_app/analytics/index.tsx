import { useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { postgres_db, schema, desc } from '@ant-colony-simulator/db-drizzle'

const getAnalyticsData = createServerFn({ method: 'GET' }).handler(async () => {
  try {
    const simulations = await postgres_db
      .select()
      .from(schema.simulations)
      .orderBy(desc(schema.simulations.created_at))

    const stats = await postgres_db
      .select()
      .from(schema.simulation_stats)
      .orderBy(desc(schema.simulation_stats.tick))
      .limit(100)

    return { simulations, stats }
  } catch (error) {
    console.error('Analytics query error:', error)
    return { simulations: [], stats: [] }
  }
})

function AnalyticsPage() {
  const { data, isLoading } = useQuery({
    queryKey: ['analytics'],
    queryFn: () => getAnalyticsData(),
    refetchInterval: 10000,
  })

  const simulations = data?.simulations ?? []
  const stats = data?.stats ?? []

  return (
    <div className="flex-1 space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold">Analytics</h1>
        <p className="text-muted-foreground text-sm mt-1">
          Simulation statistics and performance data
        </p>
      </div>

      {/* summary cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="rounded-lg border bg-card p-4">
          <p className="text-sm text-muted-foreground">Total Simulations</p>
          <p className="text-3xl font-bold mt-1">
            {isLoading ? '--' : simulations.length}
          </p>
        </div>
        <div className="rounded-lg border bg-card p-4">
          <p className="text-sm text-muted-foreground">Active</p>
          <p className="text-3xl font-bold mt-1">
            {isLoading
              ? '--'
              : simulations.filter((s) => s.is_active).length}
          </p>
        </div>
        <div className="rounded-lg border bg-card p-4">
          <p className="text-sm text-muted-foreground">Stat Snapshots</p>
          <p className="text-3xl font-bold mt-1">
            {isLoading ? '--' : stats.length}
          </p>
        </div>
      </div>

      {/* stats table */}
      {stats.length > 0 ? (
        <div className="rounded-lg border">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/50">
                  <th className="px-4 py-2 text-left font-medium">Simulation</th>
                  <th className="px-4 py-2 text-right font-medium">Tick</th>
                  <th className="px-4 py-2 text-right font-medium">Ants</th>
                  <th className="px-4 py-2 text-right font-medium">Food Collected</th>
                  <th className="px-4 py-2 text-right font-medium">Recorded</th>
                </tr>
              </thead>
              <tbody>
                {stats.map((row) => (
                  <tr key={row.id} className="border-b last:border-0">
                    <td className="px-4 py-2 text-muted-foreground">
                      #{row.simulation_id}
                    </td>
                    <td className="px-4 py-2 text-right tabular-nums">
                      {row.tick.toLocaleString()}
                    </td>
                    <td className="px-4 py-2 text-right tabular-nums">
                      {row.total_ants}
                    </td>
                    <td className="px-4 py-2 text-right tabular-nums">
                      {Math.round(row.food_collected)}
                    </td>
                    <td className="px-4 py-2 text-right text-muted-foreground">
                      {row.recorded_at
                        ? new Date(row.recorded_at).toLocaleTimeString()
                        : '--'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : (
        <div className="text-center py-16 rounded-lg border bg-card">
          <p className="text-sm font-medium">No stats yet</p>
          <p className="mt-1 text-sm text-muted-foreground">
            Run a simulation with the Rust backend to start collecting analytics data.
          </p>
        </div>
      )}
    </div>
  )
}

export const Route = createFileRoute('/_authenticated/_app/analytics/')({
  component: AnalyticsPage,
})
