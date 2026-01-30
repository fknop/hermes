import { Temporal } from 'temporal-polyfill'

export type DurationLike = Temporal.Duration | number | string

export function toTemporalDuration(duration: DurationLike): Temporal.Duration {
  if (duration instanceof Temporal.Duration) {
    return duration
  } else if (typeof duration === 'number') {
    return Temporal.Duration.from({ seconds: Math.trunc(duration) })
  } else {
    return Temporal.Duration.from(duration)
  }
}
