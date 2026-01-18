import { Temporal } from 'temporal-polyfill'
import { useDistanceFormatter } from '../../../hooks/useDistanceFormatter'
import { Solution } from '../solution'
import clsx from 'clsx'

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
  const formatDistance = useDistanceFormatter()

  const startTime = route.activities[0]?.arrival_time
  const formattedStartTime = startTime
    ? Temporal.Instant.from(startTime).toLocaleString('en-GB', {
        hour: '2-digit',
        minute: '2-digit',
      })
    : 'N/A'

  const formattedDuration = Temporal.Duration.from(
    route.duration
  ).toLocaleString()

  const formattedWaitingDuration = Temporal.Duration.from(
    route.waiting_duration
  ).toLocaleString()

  return (
    <button
      onClick={onClick}
      className={clsx(
        'group flex flex-col gap-2 p-4 text-left transition-all duration-200 w-full cursor-pointer',
        isSelected ? '' : 'hover:bg-neutral-50'
        // isSelected
        //   ? 'border-slate-800 bg-slate-50 shadow-md'
        //   : 'border-zinc-200 hover:border-zinc-300 hover:bg-zinc-50/50 hover:shadow-sm'
      )}
    >
      <div className="flex items-center justify-between">
        <span className="inline-flex items-center gap-2.5">
          <div
            className="h-4 w-4 rounded-full ring-2 ring-white shadow-sm"
            style={{ backgroundColor: color }}
          />
          <span className="font-semibold text-zinc-900">Route {index + 1}</span>
        </span>
        <span className="text-xs font-medium text-zinc-500 bg-zinc-100 px-2 py-0.5 rounded-full">
          Vehicle {route.vehicle_id + 1}
        </span>
      </div>

      <div className="grid grid-cols-2 gap-x-6 gap-y-2 text-sm mt-1">
        <div className="flex flex-col">
          <span className="text-zinc-400 text-xs uppercase tracking-wide">
            Start
          </span>
          <span className="text-zinc-700 font-medium">
            {formattedStartTime}
          </span>
        </div>
        <div className="flex flex-col">
          <span className="text-zinc-400 text-xs uppercase tracking-wide">
            Duration
          </span>
          <span className="text-zinc-700 font-medium">{formattedDuration}</span>
        </div>
        <div className="flex flex-col">
          <span className="text-zinc-400 text-xs uppercase tracking-wide">
            Distance
          </span>
          <span className="text-zinc-700 font-medium">
            {formatDistance(route.distance)}
          </span>
        </div>
        <div className="flex flex-col">
          <span className="text-zinc-400 text-xs uppercase tracking-wide">
            Activities
          </span>
          <span className="text-zinc-700 font-medium">
            {
              route.activities.filter((activity) => activity.type === 'Service')
                .length
            }
          </span>
        </div>
        <div className="flex flex-col">
          <span className="text-zinc-400 text-xs uppercase tracking-wide">
            Waiting
          </span>
          <span className="text-zinc-700 font-medium">
            {formattedWaitingDuration}
          </span>
        </div>
        <div className="flex flex-col">
          <span className="text-zinc-400 text-xs uppercase tracking-wide">
            Load
          </span>
          <span className="text-zinc-700 font-medium">
            {percentFormatter.format(route.vehicle_max_load)}
          </span>
        </div>
      </div>
    </button>
  )
}
