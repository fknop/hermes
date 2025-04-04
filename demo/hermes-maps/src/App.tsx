import { Map } from './Map.tsx'
import { AddressSearch } from './AddressSearch.tsx'
import { useFetch } from './hooks/useFetch.ts'
import { useState } from 'react'
import { Source } from 'react-map-gl/mapbox'
import { PolylineLayer } from './PolylineLayer.tsx'
import { MultiPointLayer } from './MultiPointLayer.tsx'

type GeoPoint = { lat: number; lon: number }

enum RoutingAlgorithm {
  Dijkstra = 'Dijkstra',
  Astar = 'Astar',
  BidirectionalAstar = 'BidirectionalAstar',
  Landmarks = 'Landmarks',
}

export default function App() {
  const [routeRequest, { data: routeData }] = useFetch<
    GeoJSON.FeatureCollection,
    {},
    {
      start: GeoPoint
      end: GeoPoint
      include_debug_info: boolean | null
      algorithm: RoutingAlgorithm | null
    }
  >('/route')

  const [selectedAlgorithm, setSelectedAlgorithm] = useState<RoutingAlgorithm>(
    RoutingAlgorithm.BidirectionalAstar
  )

  const [includeDebugInfo, setIncludeDebugInfo] = useState(true)

  const [start, setStart] = useState<GeoPoint | null>(null)
  const [end, setEnd] = useState<GeoPoint | null>(null)

  const computeRoute = ({ start, end }: { start: GeoPoint; end: GeoPoint }) => {
    if (!start || !end) {
      return
    }

    void routeRequest({
      method: 'POST',
      body: {
        start,
        end,
        include_debug_info: includeDebugInfo,
        algorithm: selectedAlgorithm,
      },
    })
  }

  console.log(routeData)

  return (
    <div className="h-screen w-screen">
      <Map
      // onClick={async (event) => {
      //   const { lat, lng } = event.lngLat

      //   if (event.originalEvent.button === 0) {
      //     const start = { lat, lon: lng }
      //     setStart(start)
      //     if (end) {
      //       computeRoute({ start, end })
      //     }
      //   } else if (event.originalEvent.button === 1) {
      //     const end = { lat, lon: lng }
      //     setEnd(end)
      //     if (start) {
      //       computeRoute({ start, end })
      //     }
      //   }
      // }}
      >
        {routeData && (
          <Source type="geojson" data={routeData} id="geojson">
            <MultiPointLayer
              id="forward_visited_nodes"
              featureId="forward_visited_nodes"
              color="green"
              sourceId="geojson"
            />
            <MultiPointLayer
              id="backward_visited_nodes"
              featureId="backward_visited_nodes"
              color="red"
              sourceId="geojson"
            />

            <PolylineLayer
              id="route"
              featureId="polyline"
              color="blue"
              sourceId="geojson"
            />
          </Source>
        )}

        <div className="pointer-events-auto z-50 absolute top-4 left-4 bg-zinc-100 rounded shadow-xs border border-zinc-300 min-w-96">
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

            <label className="flex flex-row items-center gap-2">
              <input
                type="checkbox"
                checked={includeDebugInfo}
                onChange={(event) => {
                  setIncludeDebugInfo(event.target.checked)
                }}
              />
              Include debug info
            </label>

            {Object.values(RoutingAlgorithm).map((algorithm) => {
              return (
                <label className="flex flex-row items-center gap-2">
                  <input
                    checked={selectedAlgorithm == algorithm}
                    type="radio"
                    name="algorithm"
                    value={algorithm}
                    onChange={(event) => {
                      setSelectedAlgorithm(
                        event.target.value as RoutingAlgorithm
                      )
                    }}
                  />
                  {algorithm}
                </label>
              )
            })}

            <button
              type="button"
              onClick={() => {
                if (start && end) {
                  computeRoute({ start, end })
                }
              }}
            >
              Route
            </button>
          </div>
        </div>
      </Map>
    </div>
  )
}
