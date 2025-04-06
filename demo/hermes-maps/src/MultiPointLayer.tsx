import { Layer } from 'react-map-gl/mapbox'

export function MultiPointLayer({
  color,
  id,
  featureId,
  sourceId,
}: {
  color: string
  id: string
  featureId: string
  sourceId: string
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
          0.5,
          10,
          1,
          15,
          2.5,
        ],
        'circle-color': color,
      }}
    />
  )
}
