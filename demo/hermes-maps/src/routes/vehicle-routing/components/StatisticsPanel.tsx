import { DataTable } from '@/components/ui/data-table'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { usePercentageFormatter } from '@/hooks/usePercentageFormatter'
import { ColumnDef } from '@tanstack/react-table'
import { useMemo } from 'react'
import { OperatorStatistics, SolutionStatistics } from '../solution'

function StatisticsDataTable({
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
  const formatPercentage = usePercentageFormatter()
  const formatDuration = useDurationFormatter()

  const columns: ColumnDef<OperatorStatistics & { name: string }>[] =
    useMemo(() => {
      const columns: ColumnDef<OperatorStatistics & { name: string }>[] = [
        {
          accessorKey: 'name',
          header: 'Name',
        },
        {
          accessorKey: 'total_invocations',
          header: 'Invocations',
        },
        {
          accessorKey: 'total_best',
          header: 'Total Best',
        },
        {
          accessorKey: 'total_improvements',
          header: 'Total Improved',
        },
        {
          accessorKey: 'avg_duration',
          header: 'Avg Duration',
          cell: (info) => formatDuration(info.row.original.avg_duration),
        },
        {
          accessorKey: 'avg_score_percentage_improvement',
          header: 'Avg %',
          cell: (info) =>
            formatPercentage(
              info.row.original.avg_score_percentage_improvement
            ),
        },
      ]

      return columns
    }, [formatPercentage])

  return <DataTable data={operatorStatistics} columns={columns} />
}

export function StatisticsPanel({
  statistics,
}: {
  statistics: SolutionStatistics
}) {
  return (
    <div className="flex flex-col gap-4">
      <StatisticsDataTable statistics={statistics.aggregated_ruin_statistics} />
      <StatisticsDataTable
        statistics={statistics.aggregated_recreate_statistics}
      />
    </div>
  )
}
