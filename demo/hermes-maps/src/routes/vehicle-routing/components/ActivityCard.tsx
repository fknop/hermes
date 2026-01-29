import { Temporal } from 'temporal-polyfill'
import { Activity } from '../solution'
import { DescriptionItem } from '@/components/ui/description-item'
import {
  ClockArrowDownIcon,
  ClockArrowUpIcon,
  CornerDownLeftIcon,
  PauseIcon,
  WarehouseIcon,
} from 'lucide-react'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { useRoutingJobContext } from './RoutingJobContext'

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
  const { input } = useRoutingJobContext()
  const formatDuration = useDurationFormatter()
  const getActivityIcon = () => {
    switch (activity.type) {
      case 'Start':
        return (
          <div className="p-2 rounded-full bg-green-100 dark:bg-neutral-600 flex items-center justify-center">
            <WarehouseIcon className="size-4" />
          </div>
        )
      case 'End':
        return (
          <div className="p-2 rounded-full bg-red-100 dark:bg-neutral-600 flex items-center justify-center">
            <CornerDownLeftIcon className="size-4" />
          </div>
        )
      case 'Service':
        return (
          <div className="size-8 rounded-full bg-blue-100 dark:bg-neutral-200 flex items-center justify-center">
            <span className="text-center text-sm font-semibold text-neutral-800">
              {index}
            </span>
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
      case 'Service': {
        const index = input!.services.findIndex(
          (service) => service.id === activity.id
        )

        return `Service #${index + 1}`
      }
    }
  }

  return (
    <div className="flex gap-3">
      <div className="flex flex-col items-center">
        {getActivityIcon()}
        {!isLast && <div className="w-0.5 flex-1 bg-zinc-200 my-1" />}
      </div>
      <div className="flex-1 pb-5 mt-1.5">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium text-secondary-foreground">
            {getActivityLabel()}
          </span>
        </div>
        <div className="mt-2 grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
          <DescriptionItem
            icon={ClockArrowDownIcon}
            label="Arrival"
            value={formatTime(activity.arrival_time)}
          />

          <DescriptionItem
            icon={ClockArrowUpIcon}
            label="Departure"
            value={formatTime(activity.departure_time)}
          />
          {activity.type === 'Service' && (
            <DescriptionItem
              icon={PauseIcon}
              label="Waiting"
              value={
                <span
                  className={
                    activity.waiting_duration === 'PT0S'
                      ? 'text-muted-foreground'
                      : 'text-amber-300'
                  }
                >
                  {formatDuration(activity.waiting_duration, {
                    style: 'narrow',
                  })}
                </span>
              }
            />
          )}
        </div>
      </div>
    </div>
  )
}
