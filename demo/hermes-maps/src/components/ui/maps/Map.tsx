import Mapbox, {
  LngLatBoundsLike,
  LngLatLike,
  useMap,
} from 'react-map-gl/mapbox'
import { PropsWithChildren, useEffect } from 'react'
import { MAPBOX_ACCESS_TOKEN } from '../../../constants.ts'
import { MAPBOX_STYLE } from '../../../constants.ts'
import { MapProps } from 'react-map-gl/mapbox'

const BRUSSELS_COORDINATES = {
  latitude: 50.85045,
  longitude: 4.34878,
}

const PARIS_COORDINATES = {
  latitude: 48.864716,
  longitude: 2.349014,
}

const initialViewState = {
  ...BRUSSELS_COORDINATES,
  zoom: 14,
}

export function Map({
  children,
  onClick,
  bounds,
}: PropsWithChildren<
  Pick<MapProps, 'onClick'> & { bounds?: LngLatBoundsLike }
>) {
  return (
    <Mapbox
      projection="mercator"
      dragRotate={false}
      initialViewState={{
        ...initialViewState,
        bounds,
        fitBoundsOptions: {
          padding: 40,
        },
      }}
      mapboxAccessToken={MAPBOX_ACCESS_TOKEN}
      style={{ flex: 1 }}
      mapStyle={MAPBOX_STYLE}
      onClick={onClick}
      reuseMaps
    >
      <AnimateBounds bounds={bounds} />
      {children}
    </Mapbox>
  )
}

function AnimateBounds({ bounds }: { bounds?: LngLatBoundsLike }) {
  const mapRef = useMap()
  console.log(bounds)

  useEffect(() => {
    if (bounds && mapRef.current) {
      mapRef.current.fitBounds(bounds, {
        padding: 20,
        duration: 500,
        animate: true,
        linear: true,
      })
    }
  }, [bounds, mapRef])

  return null
}
