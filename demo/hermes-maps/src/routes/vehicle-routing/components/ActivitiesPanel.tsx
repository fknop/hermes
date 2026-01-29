import { Temporal } from 'temporal-polyfill'
import { useDistanceFormatter } from '../../../hooks/useDistanceFormatter'
import { Solution } from '../solution'
import { ActivityCard } from './ActivityCard'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { DescriptionItem } from '@/components/ui/description-item'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { Button } from '@/components/ui/button'
import { XIcon } from 'lucide-react'

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
  const formatDistance = useDistanceFormatter()
  const formatDuration = useDurationFormatter()

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
              value={
                <span
                  className={
                    route.waiting_duration === 'PT0S'
                      ? 'text-muted-foreground'
                      : 'text-amber-300'
                  }
                >
                  {formatDuration(route.waiting_duration, {
                    style: 'narrow',
                  })}
                </span>
              }
            />
            <DescriptionItem
              label="Load"
              value={percentFormatter.format(route.vehicle_max_load)}
            />
          </CardContent>
        </Card>
      </div>
      <div className="bg-card flex-1 overflow-auto p-4 mt-2">
        <h3 className="text-xs font-semibold text-muted-foreground mb-4">
          Activities ({route.activities.length})
        </h3>
        <div className="flex flex-col">
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
