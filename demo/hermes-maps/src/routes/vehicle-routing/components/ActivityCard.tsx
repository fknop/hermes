import { Temporal } from 'temporal-polyfill'
import { Activity } from '../solution'

interface ActivityCardProps {
  activity: Activity
  index: number
  isFirst: boolean
  isLast: boolean
}

function formatTime(isoTime: string): string {
  return Temporal.Instant.from(isoTime).toLocaleString('en-GB', {
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function ActivityCard({
  activity,
  index,
  isFirst,
  isLast,
}: ActivityCardProps) {
  const getActivityIcon = () => {
    switch (activity.type) {
      case 'Start':
        return (
          <div className="w-8 h-8 rounded-full bg-green-100 flex items-center justify-center">
            <svg
              className="w-4 h-4 text-green-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 3l14 9-14 9V3z"
              />
            </svg>
          </div>
        )
      case 'End':
        return (
          <div className="w-8 h-8 rounded-full bg-red-100 flex items-center justify-center">
            <svg
              className="w-4 h-4 text-red-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"
              />
            </svg>
          </div>
        )
      case 'Service':
        return (
          <div className="w-8 h-8 rounded-full bg-blue-100 flex items-center justify-center">
            <span className="text-sm font-semibold text-blue-600">{index}</span>
          </div>
        )
    }
  }

  const getActivityLabel = () => {
    switch (activity.type) {
      case 'Start':
        return 'Depot Start'
      case 'End':
        return 'Depot End'
      case 'Service':
        return `Service #${activity.service_id}`
    }
  }

  return (
    <div className="flex gap-3">
      <div className="flex flex-col items-center">
        {getActivityIcon()}
        {!isLast && <div className="w-0.5 flex-1 bg-zinc-200 my-1" />}
      </div>
      <div className="flex-1 pb-4">
        <div className="flex items-center justify-between">
          <span className="font-medium text-zinc-900">
            {getActivityLabel()}
          </span>
          <span className="text-xs text-zinc-400 uppercase">
            {activity.type}
          </span>
        </div>
        <div className="mt-2 grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
          <div className="flex items-center gap-2">
            <svg
              className="w-3.5 h-3.5 text-zinc-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span className="text-zinc-500">Arrival:</span>
            <span className="text-zinc-700 font-medium">
              {formatTime(activity.arrival_time)}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <svg
              className="w-3.5 h-3.5 text-zinc-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
              />
            </svg>
            <span className="text-zinc-500">Departure:</span>
            <span className="text-zinc-700 font-medium">
              {formatTime(activity.departure_time)}
            </span>
          </div>
          {activity.type === 'Service' && (
            <div className="flex items-center gap-2 col-span-2">
              <svg
                className="w-3.5 h-3.5 text-zinc-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <span className="text-zinc-500">Waiting:</span>
              <span className="text-zinc-700 font-medium">
                {Temporal.Duration.from(
                  activity.waiting_duration
                ).toLocaleString()}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
