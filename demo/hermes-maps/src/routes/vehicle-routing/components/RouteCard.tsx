import { Temporal } from 'temporal-polyfill'
import { useDistanceFormatter } from '../../../hooks/useDistanceFormatter'
import { Solution } from '../solution'
import clsx from 'clsx'
import { DescriptionItem } from '@/components/ui/description-item'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'

type Route = Solution['routes'][number]

interface RouteCardProps {
  route: Route
  index: number
  color: string
  isSelected: boolean
  onClick: () => void
}

const percentFormatter = new Intl.NumberFormat('en-GB', {
  style: 'percent',
  maximumFractionDigits: 2,
})

export function RouteCard({
  route,
  index,
  color,
  isSelected,
  onClick,
}: RouteCardProps) {
  const formatDuration = useDurationFormatter()
  const formatDistance = useDistanceFormatter()

  const startTime = route.activities[0]?.arrival_time
  const formattedStartTime = startTime
    ? Temporal.Instant.from(startTime).toLocaleString('en-GB', {
        hour: '2-digit',
        minute: '2-digit',
      })
    : 'N/A'

  const formattedDuration = formatDuration(route.duration, { style: 'narrow' })

  const formattedWaitingDuration = formatDuration(route.waiting_duration, {
    style: 'narrow',
  })

  return (
    <button
      onClick={onClick}
      className={clsx(
        'group flex flex-col gap-2 p-4 text-left transition-all duration-200 w-full cursor-pointer',
        isSelected ? 'bg-secondary' : 'bg-card hover:bg-muted'
        // isSelected
        //   ? 'border-slate-800 bg-slate-50 shadow-md'
        //   : 'border-zinc-200 hover:border-zinc-300 hover:bg-zinc-50/50 hover:shadow-sm'
      )}
    >
      <div className="flex items-center justify-between">
        <span className="inline-flex items-center gap-2">
          <div
            className="h-4 w-4 rounded-full"
            style={{ backgroundColor: color }}
          />
          <span className="font-medium text-muted-foreground">
            Route {index + 1}
          </span>
        </span>
        <Badge variant="outline">Vehicle {route.vehicle_id}</Badge>
      </div>

      <div className="grid grid-cols-2 gap-x-6 gap-y-2 text-sm mt-1">
        <DescriptionItem label="Start" value={formattedStartTime} />
        <DescriptionItem label="Duration" value={formattedDuration} />
        <DescriptionItem
          label="Distance"
          value={formatDistance(route.distance)}
        />
        <DescriptionItem
          label="Activities"
          value={
            route.activities.filter((activity) => activity.type === 'Service')
              .length
          }
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
      </div>
    </button>
  )
}
