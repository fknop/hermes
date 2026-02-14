import { DurationLike, toTemporalDuration } from '@/lib/DurationLike'
import { useCallback } from 'react'

export function useDurationFormatter() {
  return useCallback(
    (
      duration: DurationLike,
      options?: { style: 'long' | 'short' | 'narrow' }
    ) => {
      let temporal = toTemporalDuration(duration)

      const style = options?.style ?? 'long'
      // @ts-ignore
      const formatter = new Intl.DurationFormat('en-GB', {
        style,
        numeric: 'auto',
        minutesDisplay: temporal.total('seconds') === 0 ? 'always' : 'auto',
      })

      return formatter.format(
        temporal.round({ largestUnit: 'hours', smallestUnit: 'second' })
      )
    },
    []
  )
}
