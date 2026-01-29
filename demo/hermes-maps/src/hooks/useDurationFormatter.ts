import { useCallback } from 'react'
import { Temporal } from 'temporal-polyfill'

export function useDurationFormatter() {
  return useCallback(
    (
      duration: number | string | Temporal.Duration,
      options?: { style: 'long' | 'short' | 'narrow' }
    ) => {
      let temporal =
        duration instanceof Temporal.Duration
          ? duration
          : typeof duration === 'number'
            ? Temporal.Duration.from({ seconds: duration })
            : Temporal.Duration.from(duration)

      const style = options?.style ?? 'long'
      // @ts-ignore
      const formatter = new Intl.DurationFormat('en-GB', {
        style,
        numeric: 'auto',
        minutesDisplay: 'always',
      })

      return formatter.format(temporal)
    },
    []
  )
}
