import { Temporal } from 'temporal-polyfill'
import { useDistanceFormatter } from '../../../hooks/useDistanceFormatter'
import { Solution } from '../solution'
import { VRP_COLORS } from '../colors'
import { RouteCard } from './RouteCard'
import { VehicleRoutingProblem } from '../input'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { PropsWithChildren } from 'react'
import { Separator } from '@/components/ui/separator'
import { DescriptionItem } from '@/components/ui/description-item'

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
  const formatDuration = useDurationFormatter()
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
      <div className="p-3">
        <Card size="sm">
          <CardHeader>
            <CardTitle>Summary</CardTitle>
          </CardHeader>{' '}
          <CardContent>
            <div className="grid grid-cols-3 gap-2">
              <DescriptionItem
                label="Duration"
                value={formatDuration(solution.duration, { style: 'narrow' })}
              />

              <DescriptionItem
                label="Distance"
                value={formatDistance(totalDistance)}
              />

              <DescriptionItem
                label="Transport"
                value={formatDuration(totalTransportDuration, {
                  style: 'narrow',
                })}
              />

              <DescriptionItem label="Jobs" value={problem.services.length} />

              <DescriptionItem label="Routes" value={solution.routes.length} />

              <DescriptionItem
                label="Unassigned"
                value={solution.unassigned_jobs.length}
              />
            </div>
          </CardContent>
        </Card>
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
