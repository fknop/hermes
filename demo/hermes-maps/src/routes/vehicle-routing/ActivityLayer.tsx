import { useMemo } from 'react'
import { Layer } from 'react-map-gl/mapbox'

export function ActivitiesLayer({
  id,
  sourceId,
  radiusMultiplier = 1,
  hiddenRoutes,
}: {
  id: string
  sourceId: string
  radiusMultiplier?: number
  hiddenRoutes: Set<number>
}) {
  const hidden = useMemo(() => {
    return Array.from(hiddenRoutes).map((routeId) => routeId.toString())
  }, [hiddenRoutes])

  return (
    <>
      <Layer
        id={id}
        source={sourceId}
        type="circle"
        paint={{
          'circle-radius': [
            'interpolate',
            ['linear'],
            ['zoom'],
            5,
            7 * radiusMultiplier,
            10,
            10 * radiusMultiplier,
            15,
            15 * radiusMultiplier,
          ],
          'circle-color': ['get', 'color'],
        }}
        filter={['!', ['in', ['get', 'routeId'], ['literal', hidden]]]}
      />
      <Layer
        id={`${id}-text`}
        source={sourceId}
        type="symbol"
        layout={{ 'text-field': ['get', 'activityId'] }}
        filter={['!', ['in', ['get', 'routeId'], ['literal', hidden]]]}
      />
    </>
  )
}
