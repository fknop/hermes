import {
  ArrowsUpDownIcon,
  ArrowTurnDownRightIcon,
} from '@heroicons/react/16/solid'
import { useCallback, useEffect, useState } from 'react'
import { Source } from 'react-map-gl/mapbox'
import { Map } from './Map.tsx'
import { MultiPointLayer } from './MultiPointLayer.tsx'
import { PolylineLayer } from './PolylineLayer.tsx'
import { AddressAutocomplete } from './components/AddressAutocomplete.tsx'
import { Button } from './components/Button.tsx'
import { Checkbox } from './components/Checkbox.tsx'
import { RadioButton } from './components/RadioButton.tsx'
import { useDistanceFormatter } from './hooks/useDistanceFormatter.ts'
import { useDurationFormatter } from './hooks/useDurationFormatter.ts'
import { useFetch } from './hooks/useFetch.ts'
import { isNil } from './utils/isNil.ts'
import { MapContextMenu, MapMenuItem } from './components/MapContextMenu.tsx'
import { GeoPoint } from './GeoPoint.ts'

enum RoutingAlgorithm {
  Dijkstra = 'Dijkstra',
  Astar = 'Astar',
  BidirectionalAstar = 'BidirectionalAstar',
  Landmarks = 'Landmarks',
}

type Address = {
  coordinates: GeoPoint
  address: string
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

  const formatDuration = useDurationFormatter()
  const formatDistance = useDistanceFormatter()
  const [selectedAlgorithm, setSelectedAlgorithm] = useState<RoutingAlgorithm>(
    RoutingAlgorithm.BidirectionalAstar
  )

  const [includeDebugInfo, setIncludeDebugInfo] = useState(true)

  const [start, setStart] = useState<Address | null>(null)
  const [end, setEnd] = useState<Address | null>(null)

  const computeRoute = useCallback(
    ({ start, end }: { start: Address; end: Address }) => {
      if (!start || !end) {
        return
      }

      void routeRequest({
        method: 'POST',
        body: {
          start: start.coordinates,
          end: end.coordinates,
          include_debug_info: includeDebugInfo,
          algorithm: selectedAlgorithm,
        },
      })
    },
    [routeRequest, includeDebugInfo, selectedAlgorithm]
  )

  useEffect(() => {
    if (!isNil(start) && !isNil(end)) {
      computeRoute({ start, end })
    }
  }, [start, end])

  const routeFeature = routeData?.features.find(
    (feature) => feature.id === 'route'
  )

  const time = routeFeature?.properties?.['time']
  const distance = routeFeature?.properties?.['distance']
  const nodesVisited = routeFeature?.properties?.['nodes']

  return (
    <div className="h-screen w-screen">
      <Map>
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
              featureId="route"
              color="#1d293d"
              sourceId="geojson"
            />
          </Source>
        )}

        <MapContextMenu>
          <MapMenuItem
            onSelect={({ coordinates }) => {
              console.log(coordinates)
              setStart({
                address: `${coordinates.lat},${coordinates.lon}`,
                coordinates,
              })
            }}
          >
            From here
          </MapMenuItem>
          <MapMenuItem
            onSelect={({ coordinates }) => {
              setEnd({
                address: `${coordinates.lat},${coordinates.lon}`,
                coordinates,
              })
            }}
          >
            To here
          </MapMenuItem>
          <MapMenuItem>Query graph</MapMenuItem>
        </MapContextMenu>
      </Map>

      <div className="pointer-events-auto z-10 absolute top-0 left-0 bottom-0 bg-white  drop-shadow-xs border-r-2 border-zinc-900/20 min-w-96">
        <div className="flex flex-col gap-2.5 px-6 py-6">
          <div className="flex flex-row gap-6 items-center">
            <div className="flex flex-1 flex-col gap-3">
              <AddressAutocomplete
                value={start?.address ?? ''}
                onRetrieve={async (response) => {
                  const [lon, lat] = response.features[0].geometry.coordinates
                  setStart({
                    coordinates: { lat, lon },
                    address: response.features[0].properties.full_address,
                  })
                }}
              />
              <AddressAutocomplete
                value={end?.address ?? ''}
                onRetrieve={async (response) => {
                  const [lon, lat] = response.features[0].geometry.coordinates
                  setEnd({
                    coordinates: { lat, lon },
                    address: response.features[0].properties.full_address,
                  })
                }}
              />
            </div>

            <button
              type="button"
              onClick={() => {
                setStart(end)
                setEnd(start)
              }}
            >
              <ArrowsUpDownIcon className="size-5 text-primary" />
            </button>
          </div>

          <label className="flex flex-row items-center gap-2">
            <Checkbox
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
                <RadioButton
                  checked={selectedAlgorithm == algorithm}
                  name="algorithm"
                  value={algorithm}
                  onChange={(event) => {
                    setSelectedAlgorithm(event.target.value as RoutingAlgorithm)
                  }}
                />
                {algorithm}
              </label>
            )
          })}

          <div>
            <Button
              variant="primary"
              icon={ArrowTurnDownRightIcon}
              onClick={() => {
                if (start && end) {
                  computeRoute({ start, end })
                }
              }}
            >
              Route
            </Button>
          </div>
        </div>

        {time && <div>{formatDuration(time / 1000)}</div>}
        {distance && <div>{formatDistance(distance)}</div>}
        {nodesVisited && <div>Nodes visited: {nodesVisited}</div>}
      </div>
    </div>
  )
}
