import { LineLayer } from 'mapbox-gl'
import { Layer } from 'react-map-gl/mapbox'

const layerStyle: Partial<LineLayer> = {
  layout: {
    'line-cap': 'round',
    'line-join': 'round',
  },
  paint: {
    'line-width': 4,
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
