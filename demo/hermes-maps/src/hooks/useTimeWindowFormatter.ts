import { useCallback } from 'react'
import { useTimeFormatter } from './useTimeFormatter'
import { isNil } from '@/utils/isNil'

export function useTimeWindowFormatter() {
  const { formatTimeRange, formatTime } = useTimeFormatter()
  return useCallback(
    (start: string | null, end: string | null) => {
      if (!isNil(start) && !isNil(end)) {
        return formatTimeRange(start, end)
      }

      if (!isNil(start)) {
        return `${formatTime(start)} - ∞`
      }

      if (!isNil(end)) {
        return `∞ - ${formatTime(end)}`
      }
    },
    [formatTimeRange, formatTime]
  )
}
