import { FilterSpecification, LineLayer } from 'mapbox-gl'
import { useMemo } from 'react'
import { Layer, LayerProps } from 'react-map-gl/mapbox'

const layerStyle: Partial<LineLayer> = {
  layout: {
    'line-cap': 'round',
    'line-join': 'round',
  },
  paint: {},
}

export function PolylineLayer({
  color,
  id,
  featureId,
  sourceId,
  lineWidth,
  beforeId,
}: {
  color: string
  id: string
  featureId?: string
  sourceId: string
  lineWidth?: number
  beforeId?: string
}) {
  const additionalProps: Pick<LayerProps, 'filter'> = useMemo(() => {
    const props: Partial<LayerProps> = {}

    if (featureId) {
      props.filter = ['==', ['get', 'id'], featureId]
    }

    return props
  }, [featureId])

  return (
    <Layer
      id={id}
      source={sourceId}
      beforeId={beforeId}
      type="line"
      {...layerStyle}
      {...additionalProps}
      paint={{
        'line-width': lineWidth ?? 4,
        ...layerStyle.paint,
        'line-color': color,
      }}
    />
  )
}
