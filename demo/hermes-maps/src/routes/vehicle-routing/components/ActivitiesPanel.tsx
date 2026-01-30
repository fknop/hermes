import { Temporal } from 'temporal-polyfill'
import { useDistanceFormatter } from '../../../hooks/useDistanceFormatter'
import { Solution } from '../solution'
import { ActivityCard } from './ActivityCard'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { DescriptionItem } from '@/components/ui/description-item'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { Button } from '@/components/ui/button'
import { EyeIcon, EyeOffIcon, ScanEyeIcon, XIcon } from 'lucide-react'
import { WaitingDuration } from './WaitingDuration'
import { ButtonGroup } from '@/components/ui/button-group'
import { Separator } from '@/components/ui/separator'
import { useRoutingJobContext } from './RoutingJobContext'

type Route = Solution['routes'][number]

interface ActivitiesPanelProps {
  route: Route
  routeIndex: number
  color: string
  onClose: () => void
}

const percentFormatter = new Intl.NumberFormat('en-GB', {
  style: 'percent',
  maximumFractionDigits: 2,
})

export function ActivitiesPanel({
  route,
  routeIndex,
  color,
  onClose,
}: ActivitiesPanelProps) {
  const { toggleRoute, hideOtherRoutes, showAllRoutes, hiddenRoutes } =
    useRoutingJobContext()
  const formatDistance = useDistanceFormatter()
  const formatDuration = useDurationFormatter()
  const isHidden = hiddenRoutes.has(routeIndex)

  return (
    <div className="flex flex-col h-full bg-background min-w-80">
      <div className="px-3 mt-3 mb-1">
        <Card>
          <CardHeader>
            <CardTitle>
              <div className="flex items-center gap-2">
                <div
                  className="w-4 h-4 rounded-full shadow-sm"
                  style={{ backgroundColor: color }}
                />
                <h2>Route {routeIndex + 1}</h2>
              </div>
            </CardTitle>
            <CardDescription>Route summary</CardDescription>
            <CardAction>
              <Button
                variant="outline"
                size="icon"
                onClick={onClose}
                aria-label="Close panel"
              >
                <XIcon />
              </Button>
            </CardAction>
          </CardHeader>
          <CardContent className="grid grid-cols-2 gap-3 text-sm">
            <DescriptionItem
              label="Duration"
              value={formatDuration(route.duration, { style: 'narrow' })}
            />
            <DescriptionItem
              label="Distance"
              value={formatDistance(route.distance)}
            />
            <DescriptionItem
              label="Waiting"
              value={<WaitingDuration duration={route.waiting_duration} />}
            />
            <DescriptionItem
              label="Load"
              value={percentFormatter.format(route.vehicle_max_load)}
            />
          </CardContent>
          <Separator />
          <CardFooter>
            <CardAction>
              <ButtonGroup>
                <Button
                  variant="outline"
                  onClick={() => toggleRoute(routeIndex)}
                >
                  {isHidden ? (
                    <EyeIcon data-icon="inline-start" />
                  ) : (
                    <EyeOffIcon data-icon="inline-start" />
                  )}
                  {isHidden ? 'Show' : 'Hide'}
                </Button>
                <Button
                  variant="outline"
                  onClick={() => hideOtherRoutes(routeIndex)}
                >
                  <ScanEyeIcon data-icon="inline-start" />
                  Hide others
                </Button>
                <Button variant="outline" onClick={() => showAllRoutes()}>
                  <EyeIcon data-icon="inline-start" />
                  Show all
                </Button>
              </ButtonGroup>
            </CardAction>
          </CardFooter>
        </Card>
      </div>
      <div className="bg-card flex-1 overflow-auto mt-2">
        <h3 className="text-xs font-semibold text-muted-foreground mb-4 p-3">
          Activities ({route.activities.length})
        </h3>
        <div className="flex flex-col divide-y divide-border">
          {route.activities.map((activity, index) => (
            <ActivityCard
              key={index}
              activity={activity}
              index={index}
              isFirst={index === 0}
              isLast={index === route.activities.length - 1}
            />
          ))}
        </div>
      </div>
    </div>
  )
}
