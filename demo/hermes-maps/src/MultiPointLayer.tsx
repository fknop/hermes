import { Layer } from 'react-map-gl/mapbox'

export function MultiPointLayer({
  color,
  id,
  featureId,
  sourceId,
  radiusMultiplier = 1,
}: {
  color: string
  id: string
  featureId: string
  sourceId: string
  radiusMultiplier?: number
}) {
  return (
    <Layer
      id={id}
      source={sourceId}
      type="circle"
      filter={['==', ['get', 'id'], featureId]}
      paint={{
        'circle-radius': [
          'interpolate',
          ['linear'],
          ['zoom'],
          5,
          0.5 * radiusMultiplier,
          10,
          1 * radiusMultiplier,
          15,
          3 * radiusMultiplier,
        ],
        'circle-color': color,
      }}
    />
  )
}
