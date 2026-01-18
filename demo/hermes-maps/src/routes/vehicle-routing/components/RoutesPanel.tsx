import { Temporal } from 'temporal-polyfill'
import { useDistanceFormatter } from '../../../hooks/useDistanceFormatter'
import { Solution } from '../solution'
import { VRP_COLORS } from '../colors'
import { RouteCard } from './RouteCard'
import { VehicleRoutingProblem } from '../input'

interface RoutesPanelProps {
  problem: VehicleRoutingProblem
  solution: Solution
  selectedRouteIndex: number | null
  onRouteSelect: (index: number | null) => void
}

export function RoutesPanel({
  problem,
  solution,
  selectedRouteIndex,
  onRouteSelect,
}: RoutesPanelProps) {
  const formatDistance = useDistanceFormatter()

  const totalDistance = solution.routes.reduce(
    (acc, route) => acc + route.distance,
    0
  )

  const totalTransportDuration = solution.routes.reduce(
    (acc, route) => acc.add(Temporal.Duration.from(route.transport_duration)),
    Temporal.Duration.from({ seconds: 0 })
  )

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1 px-3">
        <h2 className="text-lg font-semibold text-zinc-900">
          Solution summary
        </h2>
        <div className="grid grid-cols-3 gap-3 p-3 bg-neutral-100 rounded-lg">
          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Duration
            </span>
            <span className="text-zinc-800 font-semibold">
              {Temporal.Duration.from(solution.duration).toLocaleString()}
            </span>
          </div>
          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Distance
            </span>
            <span className="text-zinc-800 font-semibold">
              {formatDistance(totalDistance)}
            </span>
          </div>
          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Transport
            </span>
            <span className="text-zinc-800 font-semibold">
              {totalTransportDuration.toLocaleString()}
            </span>
          </div>

          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Jobs
            </span>
            <span className="text-zinc-800 font-semibold">
              {problem.services.length}
            </span>
          </div>

          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Routes
            </span>
            <span className="text-zinc-800 font-semibold">
              {solution.routes.length}
            </span>
          </div>

          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Unassigned
            </span>
            <span className="text-zinc-800 font-semibold">
              {solution.unassigned_jobs.length}
            </span>
          </div>
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-semibold text-zinc-700 uppercase tracking-wide px-3">
          Routes ({solution.routes.length})
        </h3>
        <div className="flex flex-col divide-y divide-gray-900/10">
          {solution.routes.map((route, index) => (
            <RouteCard
              key={index}
              route={route}
              index={index}
              color={VRP_COLORS[index % VRP_COLORS.length]}
              isSelected={selectedRouteIndex === index}
              onClick={() =>
                onRouteSelect(selectedRouteIndex === index ? null : index)
              }
            />
          ))}
        </div>
      </div>
    </div>
  )
}
