import {
  Schedule,
  ScheduleProps,
  ScheduleSegmentData,
} from '@/components/ui/schedule/schedule'
import { useRoutingJobContext } from './RoutingJobContext'
import { PropsWithChildren, ReactElement, useCallback, useMemo } from 'react'
import { Activity, SolutionRoute } from '../solution'
import { Temporal } from 'temporal-polyfill'
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from '@/components/ui/hover-card'
import { getRouteColor } from '../colors'
import { HTMLProps, mergeProps } from '@base-ui/react'
import { useTimeFormatter } from '@/hooks/useTimeFormatter'
import { Separator } from '@/components/ui/separator'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import { Service } from '../input'
import { useTimeWindowFormatter } from '@/hooks/useTimeWindowFormatter'
import {
  CornerDownLeftIcon,
  PackageIcon,
  PackageMinusIcon,
  PauseIcon,
  TruckIcon,
} from 'lucide-react'
import clsx from 'clsx'

type SegmentData =
  | { type: 'activity'; activity: Activity }
  | { type: 'waiting'; activity: Extract<Activity, { type: 'Service' }> }
  | { type: 'driving'; from: Activity; to: Activity }
type Segment = ScheduleSegmentData<SegmentData>
type Schedule = { route: SolutionRoute; index: number }

const startBgClassName = 'bg-emerald-700'
const returnBgClassName = 'bg-red-800'
const serviceBgClassName = 'bg-sky-700'
const waitingBgClassName = 'bg-yellow-600'

const TZ = 'Europe/Brussels'

function SegmentTimeRange({ segment }: { segment: Segment }) {
  const { formatTimeRange } = useTimeFormatter()

  return <span>{formatTimeRange(segment.start, segment.end)}</span>
}

function SegmentDuration({ segment }: { segment: Segment }) {
  const formatDuration = useDurationFormatter()

  return (
    <Badge variant="outline">
      {formatDuration(segment.end.since(segment.start), { style: 'short' })}
    </Badge>
  )
}

function SegmentHoverCardHeader({ children }: PropsWithChildren) {
  return <div className="flex flex-row justify-between gap-8">{children}</div>
}

function SegmentHoverCardInfo({
  label,
  children,
}: PropsWithChildren<{ label: string }>) {
  return (
    <div className="flex flex-row justify-between gap-8">
      <Label className="text-muted-foreground">{label}</Label>
      <span>{children}</span>
    </div>
  )
}

function StartHoverCardContent({
  activity,
  segment,
}: {
  activity: Extract<Activity, { type: 'Start' }>
  segment: Segment
}) {
  return (
    <div className="flex flex-col min-w-36">
      <SegmentHoverCardHeader>
        <span>Pickup</span>
        <SegmentDuration segment={segment} />
      </SegmentHoverCardHeader>
      <Separator className="my-2" />
      <SegmentHoverCardInfo label="Time">
        <SegmentTimeRange segment={segment} />
      </SegmentHoverCardInfo>
    </div>
  )
}

function ServiceHoverCardContent({
  activity,
  service,
  segment,
}: {
  activity: Activity
  service: Service
  segment: Segment
}) {
  const formatTimeWindows = useTimeWindowFormatter()
  return (
    <div className="flex flex-col min-w-36">
      <SegmentHoverCardHeader>
        <span>Service</span>
        <SegmentDuration segment={segment} />
      </SegmentHoverCardHeader>
      <Separator className="my-2" />
      <div>
        <SegmentHoverCardInfo label="Time">
          <SegmentTimeRange segment={segment} />
        </SegmentHoverCardInfo>
        <SegmentHoverCardInfo label="Time windows">
          {service.time_windows?.map((tw) => (
            <span>{formatTimeWindows(tw.start, tw.end)}</span>
          )) ?? 'N/A'}
        </SegmentHoverCardInfo>
        <SegmentHoverCardInfo label="Parcels">
          {JSON.stringify(service.demand)}
        </SegmentHoverCardInfo>
        <SegmentHoverCardInfo label="Location">
          {service.location_id}
        </SegmentHoverCardInfo>
        <SegmentHoverCardInfo label="Type">
          {service.type ?? 'Delivery'}
        </SegmentHoverCardInfo>
      </div>
    </div>
  )
}

function EndHoverCardContent({
  activity,
  segment,
}: {
  activity: Activity
  segment: Segment
}) {
  return (
    <div className="flex flex-col min-w-36">
      <SegmentHoverCardHeader>
        <span>Return</span>
        <SegmentDuration segment={segment} />
      </SegmentHoverCardHeader>
      <Separator className="my-2" />
      <SegmentHoverCardInfo label="Time">
        <SegmentTimeRange segment={segment} />
      </SegmentHoverCardInfo>
    </div>
  )
}

function DrivingHoverCardContent({
  from,
  to,
  segment,
}: {
  from: Activity
  to: Activity
  segment: Segment
}) {
  return (
    <div className="flex flex-col min-w-36">
      <SegmentHoverCardHeader>
        <span>Driving</span>
        <SegmentDuration segment={segment} />
      </SegmentHoverCardHeader>
      <Separator className="my-2" />
      <SegmentHoverCardInfo label="Time">
        <SegmentTimeRange segment={segment} />
      </SegmentHoverCardInfo>
    </div>
  )
}

function WaitingHoverCardContent({
  activity,
  service,
  segment,
}: {
  service: Service
  activity: Extract<Activity, { type: 'Service' }>
  segment: Segment
}) {
  const formatTimeWindows = useTimeWindowFormatter()
  return (
    <div className="flex flex-col min-w-36">
      <SegmentHoverCardHeader>
        <span>Idle</span>
        <SegmentDuration segment={segment} />
      </SegmentHoverCardHeader>
      <Separator className="my-2" />
      <div>
        <SegmentHoverCardInfo label="Time">
          <SegmentTimeRange segment={segment} />
        </SegmentHoverCardInfo>
        <SegmentHoverCardInfo label="Time windows">
          {service.time_windows?.map((tw) => (
            <span>{formatTimeWindows(tw.start, tw.end)}</span>
          )) ?? 'N/A'}
        </SegmentHoverCardInfo>
      </div>
    </div>
  )
}

function SegmentHoverCardContent({ segment }: { segment: Segment }) {
  const { input } = useRoutingJobContext()
  if (segment.data.type === 'activity') {
    if (segment.data.activity.type === 'Start') {
      return (
        <StartHoverCardContent
          activity={segment.data.activity}
          segment={segment}
        />
      )
    }
    if (segment.data.activity.type === 'End') {
      return (
        <EndHoverCardContent
          activity={segment.data.activity}
          segment={segment}
        />
      )
    }

    const activityId = segment.data.activity.id
    const service = input?.services.find(
      (service) => service.id === activityId
    )!
    return (
      <ServiceHoverCardContent
        activity={segment.data.activity}
        segment={segment}
        service={service}
      />
    )
  }
  if (segment.data.type === 'waiting') {
    const activityId = segment.data.activity.id
    const service = input?.services.find(
      (service) => service.id === activityId
    )!
    return (
      <WaitingHoverCardContent
        service={service}
        activity={segment.data.activity}
        segment={segment}
      />
    )
  }
  if (segment.data.type === 'driving') {
    return (
      <DrivingHoverCardContent
        from={segment.data.from}
        to={segment.data.to}
        segment={segment}
      />
    )
  }
  return null
}

function SegmentHoverCard({
  segment,
  children,
}: {
  segment: Segment
  children: ReactElement
}) {
  return (
    <HoverCard>
      <HoverCardTrigger delay={150} render={children} />
      <HoverCardContent side="top" sideOffset={8}>
        <SegmentHoverCardContent segment={segment} />
      </HoverCardContent>
    </HoverCard>
  )
}

const Legend = () => (
  <div className="flex gap-4">
    <div className="flex items-center gap-1.5 text-xs text-zinc-400 font-medium">
      <div className={clsx('w-3 h-3 rounded-sm', startBgClassName)} />
      <span>Pickup</span>
    </div>
    <div className="flex items-center gap-1.5 text-xs text-zinc-400 font-medium">
      <div
        className="w-3 h-3 rounded-sm"
        style={{ border: '2px dashed #6b7280' }}
      />
      <span>Driving</span>
    </div>
    <div className="flex items-center gap-1.5 text-xs text-zinc-400 font-medium">
      <div className={clsx('w-3 h-3 rounded-sm', waitingBgClassName)} />
      <span>Waiting</span>
    </div>
    <div className="flex items-center gap-1.5 text-xs text-zinc-400 font-medium">
      <div className={clsx('w-3 h-3 rounded-sm', serviceBgClassName)} />
      <span>Service</span>
    </div>
    <div className="flex items-center gap-1.5 text-xs text-zinc-400 font-medium">
      <div className={clsx('w-3 h-3 rounded-sm', returnBgClassName)} />
      <span>Return</span>
    </div>
  </div>
)

export function RoutingSchedule() {
  const { response } = useRoutingJobContext()

  const schedules: ScheduleProps<Schedule, SegmentData>['schedules'] =
    useMemo(() => {
      if (!response?.solution) {
        return []
      }

      return response.solution.routes.map((route, index) => {
        const segments: Segment[] = route.activities.flatMap(
          (activity, index) => {
            const segments: Segment[] = []

            let waitingDuration = Temporal.Duration.from({ seconds: 0 })
            if (activity.type === 'Service') {
              waitingDuration = Temporal.Duration.from(
                activity.waiting_duration
              )
            }

            if (
              activity.type === 'Service' &&
              waitingDuration.total('seconds') > 0
            ) {
              segments.push({
                start: Temporal.Instant.from(
                  activity.arrival_time
                ).toZonedDateTimeISO(TZ),
                end: Temporal.Instant.from(
                  Temporal.Instant.from(activity.arrival_time).add(
                    Temporal.Duration.from(activity.waiting_duration)
                  )
                ).toZonedDateTimeISO(TZ),
                data: { type: 'waiting', activity },
              })
            }

            segments.push({
              start: Temporal.Instant.from(activity.arrival_time)
                .add(waitingDuration)
                .toZonedDateTimeISO(TZ),
              end: Temporal.Instant.from(
                activity.departure_time
              ).toZonedDateTimeISO(TZ),
              data: { type: 'activity', activity },
            })

            if (index + 1 < route.activities.length) {
              const nextActivity = route.activities[index + 1]

              segments.push({
                start: Temporal.Instant.from(
                  activity.departure_time
                ).toZonedDateTimeISO(TZ),
                end: Temporal.Instant.from(
                  nextActivity.arrival_time
                ).toZonedDateTimeISO(TZ),
                data: { type: 'driving', from: activity, to: nextActivity },
              })
            }

            return segments
          }
        )

        return {
          route,
          index,
          segments,
        }
      })
    }, [response])

  const renderSegment = useCallback(
    (segment: ScheduleSegmentData<SegmentData>, props: HTMLProps) => {
      if (segment.data.type === 'activity') {
        if (segment.data.activity.type === 'Start') {
          return (
            <SegmentHoverCard segment={segment}>
              <div
                {...mergeProps(props, {
                  className: startBgClassName,
                })}
              >
                <TruckIcon className="size-3" />
              </div>
            </SegmentHoverCard>
          )
        }

        if (segment.data.activity.type === 'End') {
          return (
            <SegmentHoverCard segment={segment}>
              <div
                {...mergeProps(props, {
                  className: returnBgClassName,
                })}
              >
                <CornerDownLeftIcon className="size-3" />
              </div>
            </SegmentHoverCard>
          )
        }
        return (
          <SegmentHoverCard segment={segment}>
            <div
              {...mergeProps(props, {
                className: clsx(
                  serviceBgClassName,
                  'border border-neutral-1000/10'
                ),
              })}
            >
              <PackageMinusIcon className="size-3" />
            </div>
          </SegmentHoverCard>
        )
      }

      if (segment.data.type === 'waiting') {
        return (
          <SegmentHoverCard segment={segment}>
            <div
              {...mergeProps(props, {
                className: waitingBgClassName,
              })}
            >
              <PauseIcon className="size-3" />
            </div>
          </SegmentHoverCard>
        )
      }

      if (segment.data.type === 'driving') {
        return (
          <SegmentHoverCard segment={segment}>
            <div
              {...mergeProps(props, {
                className: 'flex items-center w-full h-full',
                style: { padding: '0 2px' },
              })}
            >
              <div
                className="flex-1 h-full text-gray-500"
                style={{ minWidth: 4, padding: '0 2px' }}
              >
                <svg className="w-full h-full" preserveAspectRatio="none">
                  <line
                    x1="0"
                    y1="50%"
                    x2="100%"
                    y2="50%"
                    stroke="currentColor"
                    strokeWidth="1"
                    strokeDasharray="6 4"
                    opacity="0.5"
                  />
                </svg>
              </div>
            </div>
          </SegmentHoverCard>
        )
      }

      return <div {...props} />
    },
    []
  )

  if (!response?.solution) {
    return null
  }

  return (
    <Schedule
      schedules={schedules}
      renderSegment={renderSegment}
      renderScheduleName={(schedule) => (
        <div className="inline-flex items-center gap-2">
          <div
            className="size-4 rounded-full"
            style={{ backgroundColor: getRouteColor(schedule.index) }}
          />
          <span className="text-sm text-muted-foreground">
            Route {schedule.index + 1}
          </span>
        </div>
      )}
      getSegmentId={(segment) =>
        `${segment.start.epochMilliseconds}-${segment.end.epochMilliseconds}`
      }
      getScheduleId={(schedule) => schedule.index.toString()}
      renderLegend={() => <Legend />}
    />
  )
}
