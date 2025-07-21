import { Layer } from 'react-map-gl/mapbox'

export function ActivitiesLayer({
  id,
  sourceId,
  radiusMultiplier = 1,
}: {
  id: string
  sourceId: string
  radiusMultiplier?: number
}) {
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
      />
      <Layer
        id={`${id}-text`}
        source={sourceId}
        type="symbol"
        layout={{ 'text-field': ['get', 'activityId'] }}
      />
    </>
  )
}
