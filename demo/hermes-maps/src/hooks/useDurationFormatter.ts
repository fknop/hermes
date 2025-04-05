import { useCallback } from 'react'

export function useDurationFormatter() {
  return useCallback((seconds: number) => {
    // @ts-ignore
    const formatter = new Intl.DurationFormat('en-GB', {
      style: 'long',
      unitDisplay: 'long',
      numeric: 'auto',
    })

    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)
    const remainingMinutes = Math.floor(minutes % 60)
    const remainingSeconds = Math.floor(seconds % 60)

    if (hours === 0 && remainingMinutes === 0) {
      return formatter.format({
        seconds: remainingSeconds,
      })
    }

    return formatter.format({
      hours,
      minutes: remainingMinutes,
    })
  }, [])
}
