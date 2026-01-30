import { useCallback } from 'react'
import { Temporal } from 'temporal-polyfill'

export function useTimeFormatter() {
  return {
    formatTime: useCallback((iso: string) => {
      const formatter = Intl.DateTimeFormat('en-GB', {
        hour: '2-digit',
        minute: '2-digit',
      })

      // const timezone = Temporal.Now.timeZoneId()
      const instant = Temporal.Instant.from(iso)
      // const zonedDateTime = instant.toZonedDateTimeISO(timezone)

      return formatter.format(instant.epochMilliseconds)
    }, []),
    formatTimeRange: useCallback((start: string, end: string) => {
      const formatter = Intl.DateTimeFormat('en-GB', {
        hour: '2-digit',
        minute: '2-digit',
      })

      const startInstant = Temporal.Instant.from(start)
      const endInstant = Temporal.Instant.from(end)

      return formatter.formatRange(
        startInstant.epochMilliseconds,
        endInstant.epochMilliseconds
      )
    }, []),
  }
}
