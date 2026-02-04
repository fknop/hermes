import { Layer } from 'react-map-gl/mapbox'

export function UnassignedJobsLayer({
  id,
  sourceId,
  radiusMultiplier = 1,
  beforeId,
}: {
  id: string
  sourceId: string
  radiusMultiplier?: number
  beforeId?: string
}) {
  return (
    <>
      <Layer
        id={id}
        source={sourceId}
        type="circle"
        beforeId={beforeId}
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
        beforeId={beforeId}
        source={sourceId}
        type="symbol"
        layout={{
          'text-field': ['get', 'jobId'],
          'text-allow-overlap': false,
          'text-size': [
            'interpolate',
            ['linear'],
            ['zoom'],
            5,
            6 * radiusMultiplier,
            10,
            12 * radiusMultiplier,
            15,
            12 * radiusMultiplier,
          ],
          'text-justify': 'center',
        }}
        paint={{
          'text-color': ['get', 'textColor'],
        }}
      />
    </>
  )
}
