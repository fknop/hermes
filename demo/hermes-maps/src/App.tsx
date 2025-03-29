import { Map } from './Map.tsx'
import { AddressSearch } from './AddressSearch.tsx'
import { useFetch } from './hooks/useFetch.ts'
import type { Feature, FeatureCollection } from 'geojson'
import { Polyline } from './Polyline.tsx'
import { useState } from 'react'

type GeoPoint = { lat: number; lon: number }

export default function App() {
  const [debugClosest, { data, loading }] = useFetch<
    {
      edge_id: number
      geojson: FeatureCollection
    },
    { lat: number; lng: number }
  >('/debug/closest')

  const [routeRequest, { data: routeData }] = useFetch<
    {
      path: FeatureCollection
    },
    {},
    { start: GeoPoint; end: GeoPoint }
  >('/route')

  console.log(routeData)

  const [start, setStart] = useState<GeoPoint | null>(null)
  const [end, setEnd] = useState<GeoPoint | null>(null)

  const computeRoute = ({ start, end }: { start: GeoPoint; end: GeoPoint }) => {
    if (!start || !end) {
      return
    }

    console.log('route request')

    void routeRequest({
      method: 'POST',
      body: {
        start,
        end,
      },
    })
  }

  return (
    <div className="h-screen w-screen">
      <Map
        onClick={async (event) => {
          const { lat, lng } = event.lngLat

          console.log(event.originalEvent.button)

          if (event.originalEvent.button === 0) {
            const start = { lat, lon: lng }
            setStart(start)
            if (end) {
              computeRoute({ start, end })
            }
          } else if (event.originalEvent.button === 1) {
            const end = { lat, lon: lng }
            setEnd(end)
            if (start) {
              computeRoute({ start, end })
            }
          }
        }}
      >
        {data && (
          <Polyline id="closest-polyline" data={data.geojson} color="gray" />
        )}

        {routeData && (
          <Polyline id="route" data={routeData.path} color="blue" />
        )}

        <div className="z-10 absolute top-4 left-4 bg-zinc-100 rounded shadow-xs border border-zinc-300 min-w-96">
          <div className="flex flex-col gap-2.5 px-3 py-3">
            <div>
              <AddressSearch
                color="blue"
                onRetrieve={async (response) => {
                  const [lon, lat] = response.geometry.coordinates
                  const start = { lon, lat }
                  setStart(start)
                  if (end) {
                    computeRoute({ start, end })
                  }
                }}
              />
            </div>
            <div>
              <AddressSearch
                color="red"
                onRetrieve={async (response) => {
                  const [lon, lat] = response.geometry.coordinates
                  const end = { lon, lat }
                  setEnd(end)
                  if (start) {
                    computeRoute({ start, end })
                  }
                }}
              />
            </div>
          </div>
        </div>
      </Map>
    </div>
  )
}
