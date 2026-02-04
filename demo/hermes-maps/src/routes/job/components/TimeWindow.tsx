import { useTimeFormatter } from '@/hooks/useTimeFormatter'
import { isNil } from '@/utils/isNil'

export function TimeWindow({
  start,
  end,
}: {
  start: string | null
  end: string | null
}) {
  const { formatTimeRange, formatTime } = useTimeFormatter()

  if (!isNil(start) && !isNil(end)) {
    return <span>{formatTimeRange(start, end)}</span>
  }

  if (!isNil(start)) {
    return <span>{formatTime(start)} - ∞</span>
  }

  if (!isNil(end)) {
    return <span>∞ - {formatTime(end)}</span>
  }
}
