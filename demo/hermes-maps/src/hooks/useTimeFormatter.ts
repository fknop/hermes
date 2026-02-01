import { useCallback } from 'react'
import { Temporal } from 'temporal-polyfill'

type TimeLike = string | Temporal.Instant | Temporal.ZonedDateTime

function getZonedDateTime(
  time: TimeLike,
  tz: Temporal.TimeZoneLike
): Temporal.ZonedDateTime {
  if (typeof time === 'string') {
    return Temporal.Instant.from(time).toZonedDateTimeISO(tz)
  } else if (time instanceof Temporal.Instant) {
    return time.toZonedDateTimeISO(tz)
  } else {
    return time
  }
}

export function useTimeFormatter() {
  return {
    formatTime: useCallback(
      (
        time: string | Temporal.Instant,
        options?: { tz?: Temporal.TimeZoneLike }
      ) => {
        const formatter = Intl.DateTimeFormat('en-GB', {
          hour: '2-digit',
          minute: '2-digit',
        })

        const tz = options?.tz ?? Temporal.Now.zonedDateTimeISO().timeZoneId

        const zonedDateTime: Temporal.ZonedDateTime = getZonedDateTime(time, tz)

        return formatter.format(zonedDateTime.epochMilliseconds)
      },
      []
    ),
    formatTimeRange: useCallback(
      (
        start: TimeLike,
        end: TimeLike,
        options?: { tz?: Temporal.TimeZoneLike }
      ) => {
        const formatter = Intl.DateTimeFormat('en-GB', {
          hour: '2-digit',
          minute: '2-digit',
        })

        const tz = options?.tz ?? Temporal.Now.zonedDateTimeISO().timeZoneId

        const startInstant = getZonedDateTime(start, tz)
        const endInstant = getZonedDateTime(end, tz)

        return formatter.formatRange(
          startInstant.epochMilliseconds,
          endInstant.epochMilliseconds
        )
      },
      []
    ),
  }
}
