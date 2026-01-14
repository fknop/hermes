import { PropsWithChildren, useMemo } from 'react'
import { OperatorStatistics, SolutionStatistics } from './solution'
import { Temporal } from 'temporal-polyfill'

function RowValue({ children }: PropsWithChildren) {
  return <td className="px-1 truncate">{children}</td>
}

function Header({ children }: PropsWithChildren) {
  return (
    <th className="px-1 text-left font-medium text-neutral-600">{children}</th>
  )
}

function OperatorStatisticsPanel({
  statistics,
}: {
  statistics: { [name: string]: OperatorStatistics }
}) {
  const operatorStatistics = useMemo(() => {
    const entries = Object.entries(statistics)
    return entries.map(([name, stats]) => ({
      name,
      ...stats,
    }))
  }, [statistics])

  return (
    <>
      {operatorStatistics.map((stats) => {
        return (
          <tr key={stats.name}>
            <RowValue>{stats.name}</RowValue>
            <RowValue>{stats.total_invocations}</RowValue>
            <RowValue>{stats.total_best}</RowValue>
            <RowValue>{stats.total_improvements}</RowValue>
            <RowValue>
              {Temporal.Duration.from(stats.avg_duration).toLocaleString()}
            </RowValue>
          </tr>
        )
      })}
    </>
  )
}

export function StatisticsPanel({
  statistics,
}: {
  statistics: SolutionStatistics
}) {
  return (
    <div className="flex flex-col max-w-3xl overflow-auto">
      <table className="table w-full table-auto">
        <thead>
          <tr>
            <Header>Name</Header>
            <Header>Total Invocations</Header>
            <Header>Total Best</Header>
            <Header>Total improved</Header>
            <Header>Avg duration</Header>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td colSpan={5} className="px-1 font-semibold">
              Ruin statistics
            </td>
          </tr>
          <OperatorStatisticsPanel
            statistics={statistics.aggregated_ruin_statistics}
          />
          <tr>
            <td colSpan={5} className="px-1 pt-4 font-semibold">
              Recreate statistics
            </td>
          </tr>
          <OperatorStatisticsPanel
            statistics={statistics.aggregated_recreate_statistics}
          />
        </tbody>
      </table>
    </div>
  )
}
