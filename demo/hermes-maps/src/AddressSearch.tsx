import { Geocoder } from '@mapbox/search-js-react'
import { MAPBOX_ACCESS_TOKEN } from './constants.ts'
import { useMap } from 'react-map-gl/mapbox'
import mapboxgl from 'mapbox-gl'
import { GeocoderProps } from '@mapbox/search-js-react/dist/components/Geocoder'

const options = {
  language: 'en',
}

// https://docs.mapbox.com/mapbox-search-js/api/web/theming/#theme
const theme = {
  variables: {
    boxShadow: 'none',
    borderRadius: 'var(--radius-sm)',
    border: '1px solid var(--color-slate-300)',
    outline: 'none',
  },
}

export function AddressSearch({
  color,
  onRetrieve,
}: {
  color: string
  onRetrieve?: GeocoderProps['onRetrieve']
}) {
  const mapRef = useMap()

  return (
    // @ts-expect-error seems it's not react 19 compatible
    <Geocoder
      mapboxgl={mapboxgl}
      map={mapRef.current?.getMap()}
      accessToken={MAPBOX_ACCESS_TOKEN}
      options={options}
      marker={{ color }}
      onRetrieve={onRetrieve}
      theme={theme}
    />
  )
}
