import type { Feature, FeatureCollection } from 'geojson'
import { Layer, Source } from 'react-map-gl/mapbox'
import { CircleLayer } from 'mapbox-gl'

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
        'circle-radius': 1,
        'circle-color': color,
      }}
    />
  )
}
