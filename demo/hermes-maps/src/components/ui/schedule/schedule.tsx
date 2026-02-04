import { useTimeFormatter } from '@/hooks/useTimeFormatter'
import { isNil } from '@/utils/isNil'
import { HTMLProps, mergeProps, useRender } from '@base-ui/react'
import { MinusIcon, PlusIcon } from 'lucide-react'
import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Temporal } from 'temporal-polyfill'
import { Button } from '../button'
import { ButtonGroup, ButtonGroupText } from '../button-group'

const SIDEBAR_WIDTH = 120
const ROW_HEIGHT = 44
const AXIS_HEIGHT = 36

export type ScheduleSegmentData<D> = {
  start: Temporal.ZonedDateTime
  end: Temporal.ZonedDateTime
  data: D
}

type Schedule<S, D> = S & {
  segments: ScheduleSegmentData<D>[]
}

export type ScheduleProps<S, D> = {
  schedules: Schedule<S, D>[]
  getScheduleId: (schedule: Schedule<S, D>) => string
  renderScheduleName: (schedule: Schedule<S, D>) => React.ReactNode
  getSegmentId: (segment: ScheduleSegmentData<D>) => string
  renderSegment: (
    segment: ScheduleSegmentData<D>,
    props: HTMLProps
  ) => React.ReactElement
  renderLegend: () => React.ReactElement
}

function minDateTime(a: Temporal.ZonedDateTime, b: Temporal.ZonedDateTime) {
  return Temporal.ZonedDateTime.compare(a, b) <= 0 ? a : b
}

function maxDateTime(a: Temporal.ZonedDateTime, b: Temporal.ZonedDateTime) {
  return Temporal.ZonedDateTime.compare(a, b) >= 0 ? a : b
}

function epochMinutes(instant: Temporal.ZonedDateTime): number {
  const ms = instant.epochMilliseconds
  return ms / 1000 / 60
}

const minutesToInstant = (m: number): Temporal.Instant =>
  Temporal.Instant.fromEpochMilliseconds(m * 60 * 1000)

// Time Axis
const TimeAxis = ({
  startTime,
  endTime,
  pixelsPerMinute,
}: {
  startTime: number
  endTime: number
  pixelsPerMinute: number
}) => {
  const { formatTime } = useTimeFormatter()

  const ticks = useMemo(() => {
    // Choose tick interval based on how many pixels per minute we have
    const pixelsPer15Min = pixelsPerMinute * 15
    const tickInterval = pixelsPer15Min >= 40 ? 15 : 30

    const result: {
      time: number
      label: string
      isMajor: boolean
      hideLabel: boolean
    }[] = []
    const start = Math.floor(startTime / tickInterval) * tickInterval
    for (let t = start; t <= endTime; t += tickInterval) {
      result.push({
        time: t,
        label: formatTime(minutesToInstant(t)),
        isMajor: t % 60 === 0,
        hideLabel: false,
      })
    }
    // Hide first and last tick labels
    if (result.length > 0) result[0].hideLabel = true
    if (result.length > 1) result[result.length - 1].hideLabel = true
    return result
  }, [startTime, endTime, pixelsPerMinute, formatTime])

  return (
    <div className="relative h-full">
      {ticks.map((tick) => {
        if (tick.hideLabel) return null
        return (
          <div
            key={tick.time}
            className="absolute top-0 h-full"
            style={{ left: (tick.time - startTime) * pixelsPerMinute }}
          >
            <div
              className={`absolute left-0 top-0 ${tick.isMajor ? 'bg-zinc-500' : 'bg-zinc-700'}`}
              style={{ width: 1, bottom: 18 }}
            />
            <div
              className={`absolute bottom-0 font-mono whitespace-nowrap ${tick.isMajor ? 'text-zinc-400 font-medium' : 'text-zinc-600'}`}
              style={{
                fontSize: 9,
                transform: 'translateX(-50%)',
                height: 18,
                display: 'flex',
                alignItems: 'center',
              }}
            >
              {tick.label}
            </div>
          </div>
        )
      })}
    </div>
  )
}

// Segment
function Segment<D>({
  segment,
  startTime,
  pixelsPerMinute,
  render,
  ...otherProps
}: {
  segment: ScheduleSegmentData<D>
  startTime: number
  pixelsPerMinute: number
} & useRender.ComponentProps<'div'>) {
  const segStart = epochMinutes(segment.start) - startTime
  const duration = epochMinutes(segment.end) - epochMinutes(segment.start)
  const left = segStart * pixelsPerMinute
  const width = duration * pixelsPerMinute

  const defaultProps: useRender.ElementProps<'div'> = {
    className:
      'absolute top-1/2 rounded cursor-pointer overflow-hidden flex items-center justify-center hover:z-10',
    style: {
      left,
      width: Math.max(width, 2),
      height: 24,
      transform: 'translateY(-50%)',
    },
  }

  console.log(otherProps)
  const element = useRender({
    defaultTagName: 'div',
    render,
    props: mergeProps<'div'>(defaultProps, otherProps),
  })

  return element
}

// Main
export function Schedule<S, D>({
  schedules,
  getScheduleId,
  getSegmentId,
  renderScheduleName,
  renderSegment,
  renderLegend,
}: ScheduleProps<S, D>) {
  const [zoomLevel, setZoomLevel] = useState<number | null>(null)
  const [baseZoom, setBaseZoom] = useState(3)

  const scrollRef = useRef<HTMLDivElement>(null)
  const [isDragging, setIsDragging] = useState(false)
  const dragStart = useRef({ x: 0, y: 0, scrollLeft: 0, scrollTop: 0 })

  const { startTime, endTime } = useMemo(() => {
    let min: Temporal.ZonedDateTime | null = null
    let max: Temporal.ZonedDateTime | null = null

    schedules.forEach((r) =>
      r.segments.forEach((s) => {
        min = isNil(min) ? s.start : minDateTime(min, s.start)
        max = isNil(max) ? s.end : maxDateTime(max, s.end)
      })
    )

    if (isNil(min) || isNil(max)) {
      return {
        startTime: Temporal.Now.zonedDateTimeISO(),
        endTime: Temporal.Now.zonedDateTimeISO(),
      }
    }

    return { startTime: min, endTime: max }
  }, [schedules])

  const { startMinutes, endMinutes } = useMemo(() => {
    return {
      startMinutes: Math.floor((epochMinutes(startTime) - 15) / 30) * 30,
      endMinutes: Math.ceil((epochMinutes(endTime) + 15) / 30) * 30,
    }
  }, [startTime, endTime])

  useEffect(() => {
    const calc = () => {
      if (scrollRef.current) {
        const available = scrollRef.current.clientWidth - SIDEBAR_WIDTH
        const minutes = endMinutes - startMinutes
        if (minutes === 0) {
          return
        }
        const z = available / minutes
        setBaseZoom(z)
        setZoomLevel((prev) => (prev === null ? z : Math.max(z, prev)))
      }
    }
    calc()
    window.addEventListener('resize', calc)
    return () => window.removeEventListener('resize', calc)
  }, [startMinutes, endMinutes])

  const effectiveZoom = zoomLevel ?? baseZoom
  const timelineWidth = (endMinutes - startMinutes) * effectiveZoom

  // Min zoom = baseZoom (100% fills the view exactly), max = 4x
  const clampZoom = useCallback(
    (z: number) => Math.max(baseZoom, Math.min(baseZoom * 4, z)),
    [baseZoom]
  )

  const handleZoom = (d: number) =>
    setZoomLevel((p) => clampZoom((p || baseZoom) + d * baseZoom * 0.25))

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return
    setIsDragging(true)
    dragStart.current = {
      x: e.clientX,
      y: e.clientY,
      scrollLeft: scrollRef.current?.scrollLeft || 0,
      scrollTop: scrollRef.current?.scrollTop || 0,
    }
  }, [])

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!isDragging) return
      if (scrollRef.current) {
        scrollRef.current.scrollLeft =
          dragStart.current.scrollLeft - (e.clientX - dragStart.current.x)
        scrollRef.current.scrollTop =
          dragStart.current.scrollTop - (e.clientY - dragStart.current.y)
      }
    },
    [isDragging]
  )

  const stopDrag = useCallback(() => setIsDragging(false), [])

  return (
    <div className="bg-background text-foreground rounded-lg overflow-hidden border border-border relative">
      {/* Header */}
      <div className="flex justify-between items-center px-5 py-3 border-b border-border bg-background">
        <div className="flex items-baseline gap-3">
          <h2 className="text-sm font-bold tracking-tight">Planning</h2>
          <span className="text-xs text-muted-foreground font-medium">
            {schedules.length} routes
          </span>
        </div>
        <div className="flex items-center gap-6">
          {renderLegend()}
          <ButtonGroup>
            <Button
              size="icon"
              variant="outline"
              onClick={() => handleZoom(-1)}
            >
              <MinusIcon />
            </Button>
            <ButtonGroupText className="bg-input">
              {Math.round((effectiveZoom / baseZoom) * 100)}%
            </ButtonGroupText>
            <Button size="icon" variant="outline" onClick={() => handleZoom(1)}>
              <PlusIcon />
            </Button>
          </ButtonGroup>
        </div>
      </div>

      {/* Single scroll container */}
      <div
        ref={scrollRef}
        className={`overflow-auto ${isDragging ? 'cursor-grabbing select-none' : 'cursor-grab'}`}
        style={{ maxHeight: 400 }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={stopDrag}
        onMouseLeave={stopDrag}
      >
        <div style={{ width: SIDEBAR_WIDTH + timelineWidth }}>
          {/* Sticky time axis */}
          <div className="flex sticky top-0 z-20 bg-neutral-900 border-b border-zinc-800">
            <div
              className="shrink-0 border-r border-zinc-800 bg-neutral-900 sticky left-0 z-20"
              style={{ width: SIDEBAR_WIDTH, height: AXIS_HEIGHT }}
            />
            <div
              className="relative"
              style={{ width: timelineWidth, height: AXIS_HEIGHT }}
            >
              <TimeAxis
                startTime={startMinutes}
                endTime={endMinutes}
                pixelsPerMinute={effectiveZoom}
              />
            </div>
          </div>

          {/* Route rows */}
          {schedules.map((schedule) => (
            <div
              key={getScheduleId(schedule)}
              className="flex border-b border-zinc-800"
            >
              {/* Sticky sidebar */}
              <div
                className="shrink-0 sticky left-0 z-10 bg-neutral-900 border-r border-zinc-800 flex items-center"
                style={{
                  width: SIDEBAR_WIDTH,
                  height: ROW_HEIGHT,
                  padding: '0 16px',
                }}
              >
                <div className="font-semibold text-sm text-zinc-100 truncate">
                  {renderScheduleName(schedule)}
                </div>
              </div>
              {/* Timeline track */}
              <div
                className="relative"
                style={{ width: timelineWidth, height: ROW_HEIGHT }}
              >
                <div className="absolute inset-0 pointer-events-none">
                  {Array.from({
                    length: Math.ceil((endMinutes - startMinutes) / 60) + 1,
                  }).map((_, i) => (
                    <div
                      key={i}
                      className="absolute top-0 bottom-0 bg-zinc-800 opacity-30"
                      style={{ left: i * 60 * effectiveZoom, width: 1 }}
                    />
                  ))}
                </div>
                <div
                  className="relative w-full flex items-center"
                  style={{ height: ROW_HEIGHT }}
                >
                  <div className="relative w-full" style={{ height: 24 }}>
                    {schedule.segments.map((segment) => (
                      <Segment
                        key={getSegmentId(segment)}
                        segment={segment}
                        startTime={startMinutes}
                        pixelsPerMinute={effectiveZoom}
                        render={(props) => renderSegment(segment, props)}
                      />
                    ))}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
