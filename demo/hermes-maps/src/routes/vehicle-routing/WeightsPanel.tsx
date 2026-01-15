import { PropsWithChildren, useMemo } from 'react'
import {
  AlnsWeights,
  OperatorStatistics,
  OperatorWeights,
  SolutionStatistics,
} from './solution'
import { Temporal } from 'temporal-polyfill'

function RowValue({ children }: PropsWithChildren) {
  return <td className="px-1 truncate">{children}</td>
}

function Header({ children }: PropsWithChildren) {
  return (
    <th className="px-1 text-left font-medium text-neutral-600">{children}</th>
  )
}

function OperatorWeightsPanel({
  weights,
}: {
  weights: AlnsWeights['weights']
}) {
  const sortedWeights = useMemo(() => {
    return weights.toSorted((a, b) => a.weight - b.weight)
  }, [weights])

  return (
    <>
      {sortedWeights.map((weight) => {
        return (
          <tr key={weight.strategy}>
            <RowValue>{weight.strategy}</RowValue>
            <RowValue>{weight.weight}</RowValue>
          </tr>
        )
      })}
    </>
  )
}

export function WeightsPanel({ weights }: { weights: OperatorWeights }) {
  return (
    <div className="flex flex-col max-w-3xl overflow-auto">
      <table className="table w-full table-auto">
        <thead>
          <tr>
            <Header>Name</Header>
            <Header>Weight</Header>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td colSpan={2} className="px-1 font-semibold">
              Ruin weights
            </td>
          </tr>
          <OperatorWeightsPanel weights={weights.ruin.weights} />
          <tr>
            <td colSpan={2} className="px-1 pt-4 font-semibold">
              Recreate weights
            </td>
          </tr>
          <OperatorWeightsPanel weights={weights.recreate.weights} />
        </tbody>
      </table>
    </div>
  )
}
