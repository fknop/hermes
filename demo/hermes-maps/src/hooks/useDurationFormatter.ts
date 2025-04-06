import { useCallback } from 'react'

export function useDurationFormatter() {
  return useCallback(
    (seconds: number, options?: { style: 'long' | 'short' | 'narrow' }) => {
      const style = options?.style ?? 'long'
      // @ts-ignore
      const formatter = new Intl.DurationFormat('en-GB', {
        style,
        numeric: 'auto',
      })

      const minutes = Math.floor(seconds / 60)
      const hours = Math.floor(minutes / 60)
      const remainingMinutes = Math.floor(minutes % 60)
      const remainingSeconds = Math.floor(seconds % 60)

      if (hours === 0 && remainingMinutes === 0) {
        if (remainingSeconds == 0) {
          return formatter.format({
            milliseconds: Math.round(seconds * 1000),
          })
        }

        return formatter.format({
          seconds: remainingSeconds,
        })
      }

      return formatter.format({
        hours,
        minutes: remainingMinutes,
      })
    },
    []
  )
}
