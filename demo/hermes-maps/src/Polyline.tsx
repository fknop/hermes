import type { Feature, FeatureCollection } from 'geojson'
import { Layer, Source } from 'react-map-gl/mapbox'
import { LineLayer } from 'mapbox-gl'

const layerStyle: Partial<LineLayer> = {
  paint: {
    'line-width': 4,
    'line-color': 'red',
  },
}

export function Polyline({
  data,
  color,
  id,
}: {
  data: FeatureCollection
  color: string
  id: string
}) {
  return (
    <Source type="geojson" data={data}>
      <Layer
        id={id}
        type="line"
        {...layerStyle}
        paint={{
          ...layerStyle.paint,
          'line-color': color,
        }}
      />
    </Source>
  )
}
