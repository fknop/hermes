import { Layer } from 'react-map-gl/mapbox'

export function LocationsLayer({
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
        layout={{
          'text-field': ['get', 'locationId'],
          'text-allow-overlap': true,
          'text-size': [
            'interpolate',
            ['linear'],
            ['zoom'],
            5,
            5 * radiusMultiplier,
            10,
            12 * radiusMultiplier,
            15,
            12 * radiusMultiplier,
          ],
          'text-justify': 'center',
        }}
        paint={{
          'text-color': 'white',
        }}
      />
    </>
  )
}
