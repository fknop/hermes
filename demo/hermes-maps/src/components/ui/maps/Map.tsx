import { PropsWithChildren, useEffect } from 'react'
import Mapbox, { LngLatBoundsLike, MapProps, useMap } from 'react-map-gl/mapbox'
import { MAPBOX_ACCESS_TOKEN, MAPBOX_STYLE } from '../../../constants.ts'

const BRUSSELS_COORDINATES = {
  latitude: 50.85045,
  longitude: 4.34878,
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
      trackResize
    >
      <AnimateBounds bounds={bounds} />
      <Resizer />
      {children}
    </Mapbox>
  )
}

const Resizer = () => {
  const mapRef = useMap()

  useEffect(() => {
    const resizeObserver = new ResizeObserver(() => {
      // const bounds = mapRef.current!.getBounds()
      // console.log(bounds)
      const center = mapRef.current!.getCenter()
      // mapRef.current!.resize()
      // mapRef.current!.fitBounds(bounds!, { duration: 0, padding: 0 })
      mapRef.current!.setCenter(center)
    })

    resizeObserver.observe(mapRef.current!.getContainer())

    return () => {
      resizeObserver.disconnect()
    }
  }, [mapRef])

  return null
}

function AnimateBounds({ bounds }: { bounds?: LngLatBoundsLike }) {
  const mapRef = useMap()

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
