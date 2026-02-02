import { AlnsWeights, OperatorWeights } from '@/api/generated/schemas'
import { DataTable } from '@/components/ui/data-table'
import { ColumnDef } from '@tanstack/react-table'
import { useMemo } from 'react'

function WeightsDataTable({ weights }: { weights: AlnsWeights['weights'] }) {
  const columns: ColumnDef<AlnsWeights['weights'][number]>[] = useMemo(() => {
    const columns: ColumnDef<AlnsWeights['weights'][number]>[] = [
      {
        accessorKey: 'strategy',
        header: 'Strategy',
      },
      {
        accessorKey: 'weight',
        header: 'Weight',
      },
    ]

    return columns
  }, [])

  return <DataTable data={weights} columns={columns} />
}

export function WeightsPanel({ weights }: { weights: OperatorWeights }) {
  return (
    <div className="flex flex-row gap-4">
      <WeightsDataTable weights={weights.ruin.weights} />
      <WeightsDataTable weights={weights.recreate.weights} />
    </div>
  )
}
