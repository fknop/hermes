import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { DurationLike, toTemporalDuration } from '@/lib/DurationLike'
import { useMemo } from 'react'

export function WaitingDuration({ duration }: { duration: DurationLike }) {
  const formatDuration = useDurationFormatter()
  const temporal = useMemo(() => {
    return toTemporalDuration(duration)
  }, [duration])

  return (
    <span className={temporal.total('minutes') > 10 ? 'text-amber-300' : ''}>
      {formatDuration(temporal, { style: 'narrow' })}
    </span>
  )
}
