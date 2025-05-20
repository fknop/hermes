import { MapPinIcon } from '@heroicons/react/24/solid'
import { Marker } from 'react-map-gl/mapbox'
import { GeoPoint } from '../types/GeoPoint'

export function LandmarkMarker({
  coordinates,
  color,
}: {
  coordinates: GeoPoint
  color: string
}) {
  return (
    <Marker
      longitude={coordinates.lon}
      latitude={coordinates.lat}
      anchor="bottom"
    >
      <MapPinIcon className="size-8 fill-current" color={color} />
    </Marker>
  )
}
