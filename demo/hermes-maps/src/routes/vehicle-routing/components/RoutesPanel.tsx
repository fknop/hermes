import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { DescriptionItem } from '@/components/ui/description-item'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { SquareArrowUpRightIcon } from 'lucide-react'
import { Temporal } from 'temporal-polyfill'
import { useDistanceFormatter } from '../../../hooks/useDistanceFormatter'
import { VRP_COLORS } from '../colors'
import { VehicleRoutingProblem } from '../input'
import { Solution } from '../solution'
import { RouteCard } from './RouteCard'
import { WaitingDuration } from './WaitingDuration'
import { useRoutingJobContext } from './RoutingJobContext'

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
  const { showUnassigned, setShowUnassigned } = useRoutingJobContext()
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

  const totalWaitingDuration = solution.routes.reduce(
    (acc, route) => acc.add(Temporal.Duration.from(route.waiting_duration)),
    Temporal.Duration.from({ seconds: 0 })
  )

  return (
    <div className="flex flex-col gap-4 mt-3">
      <div className="px-3">
        <Card size="sm">
          <CardHeader>
            <CardTitle>Summary</CardTitle>
          </CardHeader>
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

              <DescriptionItem
                label="Idle"
                value={<WaitingDuration duration={totalWaitingDuration} />}
              />

              <DescriptionItem label="Jobs" value={problem.services.length} />

              <DescriptionItem label="Routes" value={solution.routes.length} />

              <DescriptionItem
                label="Unassigned"
                value={
                  solution.unassigned_jobs.length > 0 ? (
                    <Button
                      size="sm"
                      variant="destructive"
                      className="mt-0.5"
                      onClick={() => {
                        setShowUnassigned(!showUnassigned)
                      }}
                    >
                      <span>{solution.unassigned_jobs.length}</span>
                      <SquareArrowUpRightIcon data-icon="inline-end" />
                    </Button>
                  ) : (
                    <span>{solution.unassigned_jobs.length}</span>
                  )
                }
              />
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="flex flex-col gap-2 px-3">
        <div className="flex flex-col divide-y divide-border  rounded-lg border border-border">
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
