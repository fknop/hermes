import { useDistanceFormatter } from '../hooks/useDistanceFormatter'
import { useDurationFormatter } from '../hooks/useDurationFormatter'

export function RouteResult({
  distance,
  time,
  nodesVisited,
  duration,
}: {
  distance: number
  time: number
  nodesVisited: number
  duration: number
}) {
  const formatDuration = useDurationFormatter()
  const formatDistance = useDistanceFormatter()

  return (
    <div className="px-6 flex flex-row justify-between">
      <div className="flex flex-col">
        <span className="font-semibold text-gray-900">
          {formatDuration(time / 1000)}
        </span>
        <span className="text-sm text-gray-500">
          {formatDistance(distance)}
        </span>
      </div>

      <div className="flex flex-col items-end self-end">
        <span className="text-xs text-gray-500">
          Nodes visited: <span className="font-semibold">{nodesVisited}</span>
        </span>
        <span className="text-xs text-gray-500">
          Duration:{' '}
          <span className="font-semibold">
            {formatDuration(duration / 1000, { style: 'narrow' })}
          </span>
        </span>
      </div>
    </div>
  )
}
