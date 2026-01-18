import { Temporal } from 'temporal-polyfill'
import { useDistanceFormatter } from '../../../hooks/useDistanceFormatter'
import { Solution } from '../solution'
import { ActivityCard } from './ActivityCard'

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

  return (
    <div className="flex flex-col h-full bg-white border-l-2 border-zinc-200 min-w-80 max-w-96">
      <div className="flex items-center justify-between p-4 border-b border-zinc-100 flex-shrink-0">
        <div className="flex items-center gap-2">
          <div
            className="w-4 h-4 rounded-full ring-2 ring-white shadow-sm"
            style={{ backgroundColor: color }}
          />
          <h2 className="font-semibold text-zinc-900">
            Route {routeIndex + 1}
          </h2>
        </div>
        <button
          onClick={onClose}
          className="p-1.5 rounded-lg hover:bg-zinc-100 transition-colors"
          aria-label="Close panel"
        >
          <svg
            className="w-5 h-5 text-zinc-500"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <div className="p-4 border-b border-zinc-100 bg-zinc-50 flex-shrink-0">
        <div className="grid grid-cols-2 gap-3 text-sm">
          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Duration
            </span>
            <span className="text-zinc-800 font-medium">
              {Temporal.Duration.from(route.duration).toLocaleString()}
            </span>
          </div>
          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Distance
            </span>
            <span className="text-zinc-800 font-medium">
              {formatDistance(route.distance)}
            </span>
          </div>
          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Waiting
            </span>
            <span className="text-zinc-800 font-medium">
              {Temporal.Duration.from(route.waiting_duration).toLocaleString()}
            </span>
          </div>
          <div className="flex flex-col">
            <span className="text-zinc-400 text-xs uppercase tracking-wide">
              Load
            </span>
            <span className="text-zinc-800 font-medium">
              {percentFormatter.format(route.vehicle_max_load)}
            </span>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        <h3 className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-4">
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
