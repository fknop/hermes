import Mapbox from 'react-map-gl/mapbox'
import { PropsWithChildren } from 'react'
import { MAPBOX_ACCESS_TOKEN } from './constants'
import { MAPBOX_STYLE } from './constants.ts'
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
  ...PARIS_COORDINATES,
  zoom: 14,
}

export function Map({
  children,
  onClick,
}: PropsWithChildren<Pick<MapProps, 'onClick'>>) {
  return (
    <Mapbox
      projection="mercator"
      dragRotate={false}
      initialViewState={initialViewState}
      mapboxAccessToken={MAPBOX_ACCESS_TOKEN}
      style={{ flex: 1 }}
      mapStyle={MAPBOX_STYLE}
      onClick={onClick}
    >
      {children}
    </Mapbox>
  )
}
