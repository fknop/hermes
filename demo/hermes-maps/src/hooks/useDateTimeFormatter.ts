import { useCallback } from 'react'
import { Temporal } from 'temporal-polyfill'

type DateTimeLike = string | Temporal.Instant | Temporal.ZonedDateTime

function getZonedDateTime(
  time: DateTimeLike,
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

export function useDateTimeFormatter() {
  return {
    formatDateTime: useCallback(
      (
        time: DateTimeLike,
        options: {
          format: Intl.DateTimeFormatOptions
          tz?: Temporal.TimeZoneLike
        }
      ) => {
        const formatter = Intl.DateTimeFormat('en-GB', {
          ...options.format,
        })

        const tz = options?.tz ?? Temporal.Now.zonedDateTimeISO().timeZoneId
        const zonedDateTime: Temporal.ZonedDateTime = getZonedDateTime(time, tz)

        return formatter.format(zonedDateTime.epochMilliseconds)
      },
      []
    ),
  }
}
