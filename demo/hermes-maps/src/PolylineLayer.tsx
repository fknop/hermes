import type { Feature, FeatureCollection } from 'geojson'
import { Layer, Source } from 'react-map-gl/mapbox'
import { LineLayer } from 'mapbox-gl'

const layerStyle: Partial<LineLayer> = {
  paint: {
    'line-width': 4,
    'line-color': 'red',
  },
}

export function PolylineLayer({
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
      type="line"
      filter={['==', ['get', 'id'], featureId]}
      {...layerStyle}
      paint={{
        ...layerStyle.paint,
        'line-color': color,
      }}
    />
  )
}
