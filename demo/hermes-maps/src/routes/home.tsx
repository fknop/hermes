import { MagnifyingGlassIcon, MapPinIcon } from '@heroicons/react/16/solid'
import { BuildingOfficeIcon } from '@heroicons/react/24/solid'
import { useCallback, useEffect, useState } from 'react'
import { Source } from 'react-map-gl/mapbox'
import { Map } from '../Map.tsx'
import { MultiPointLayer } from '../MultiPointLayer.tsx'
import { PolylineLayer } from '../PolylineLayer.tsx'
import { Checkbox } from '../components/Checkbox.tsx'
import { JourneyAutocomplete } from '../components/JourneyAutocomplete.tsx'
import { Label } from '../components/Label.tsx'
import { MapContextMenu, MapMenuItem } from '../components/MapContextMenu.tsx'
import { MapMarker } from '../components/MapMarker.tsx'
import { RadioButton } from '../components/RadioButton.tsx'
import { RouteResult } from '../components/RouteResult.tsx'
import { Slider } from '../components/Slider.tsx'
import { useFetch } from '../hooks/useFetch.ts'
import { Address } from '../types/Address.ts'
import { GeoPoint } from '../types/GeoPoint.ts'
import { isNil } from '../utils/isNil.ts'
import { LandmarkMarker } from '../components/LandmarkMarker.tsx'
import { MapSidePanel } from '../components/MapSidePanel.tsx'

enum RoutingAlgorithm {
  Dijkstra = 'Dijkstra',
  Astar = 'Astar',
  BidirectionalAstar = 'BidirectionalAstar',
  Landmarks = 'Landmarks',
  ContractionHierarchies = 'ContractionHierarchies',
}

export default function HomeScreen() {
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

  const [landmarksRequest, { data: landmarksData }] =
    useFetch<GeoJSON.FeatureCollection<GeoJSON.Point>>('/landmarks')

  const fetchLandmarks = useCallback(async () => {
    await landmarksRequest({ method: 'GET' })
  }, [landmarksRequest])

  const [selectedAlgorithm, setSelectedAlgorithm] = useState<RoutingAlgorithm>(
    RoutingAlgorithm.Dijkstra
  )

  const [includeDebugInfo, setIncludeDebugInfo] = useState(false)
  const [showLandmarks, setShowLandmarks] = useState(false)
  const [start, setStart] = useState<Address | null>(null)
  const [end, setEnd] = useState<Address | null>(null)
  const [radiusMultiplier, setRadiusMultiplier] = useState<number>(1)

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
  const duration = routeFeature?.properties?.['duration']

  return (
    <div className="h-screen w-screen">
      <Map>
        {routeData && (
          <Source type="geojson" data={routeData} id="geojson">
            <MultiPointLayer
              id="forward_visited_nodes"
              featureId="forward_visited_nodes"
              color="#00a6f4"
              sourceId="geojson"
              radiusMultiplier={radiusMultiplier}
            />
            <MultiPointLayer
              id="backward_visited_nodes"
              featureId="backward_visited_nodes"
              color="#ff6467"
              sourceId="geojson"
              radiusMultiplier={radiusMultiplier}
            />

            <PolylineLayer
              id="route"
              featureId="route"
              color="#1d293d"
              sourceId="geojson"
            />
          </Source>
        )}

        {start && (
          <MapMarker
            color="var(--color-sky-800)"
            coordinates={start.coordinates}
          />
        )}
        {end && (
          <MapMarker
            color="var(--color-orange-800)"
            coordinates={end.coordinates}
          />
        )}

        {showLandmarks &&
          landmarksData?.features.map((feature) => {
            return (
              <LandmarkMarker
                coordinates={{
                  lon: feature.geometry.coordinates[0],
                  lat: feature.geometry.coordinates[1],
                }}
                color="var(--color-green-600)"
              />
            )
          })}

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
            <span className="flex items-center gap-1 whitespace-nowrap">
              <MapPinIcon className="size-5 text-sky-800" />
              <span>From here</span>
            </span>
          </MapMenuItem>
          <MapMenuItem
            onSelect={({ coordinates }) => {
              setEnd({
                address: `${coordinates.lat},${coordinates.lon}`,
                coordinates,
              })
            }}
          >
            <span className="inline-flex items-center gap-1 whitespace-nowrap">
              <MapPinIcon className="size-5 text-orange-800" />
              <span>To here</span>
            </span>
          </MapMenuItem>
          <MapMenuItem>
            <span className="inline-flex items-center gap-1 whitespace-nowrap">
              <MagnifyingGlassIcon className="size-5" />
              <span>Query graph</span>
            </span>
          </MapMenuItem>

          <MapMenuItem
            onSelect={async () => {
              if (!showLandmarks) {
                await fetchLandmarks()
              }
              setShowLandmarks((show) => !show)
            }}
          >
            <span className="inline-flex items-center gap-1 whitespace-nowrap">
              <BuildingOfficeIcon className="size-5" />
              {showLandmarks ? 'Hide landmarks' : 'Show landmarks'}
            </span>
          </MapMenuItem>
        </MapContextMenu>
      </Map>

      <MapSidePanel>
        <JourneyAutocomplete
          start={start}
          end={end}
          onChange={(start, end) => {
            setStart(start)
            setEnd(end)
          }}
          onSearch={() => {
            if (start && end) {
              computeRoute({ start, end })
            }
          }}
        />

        {Object.values(RoutingAlgorithm).map((algorithm) => {
          return (
            <Label>
              <RadioButton
                checked={selectedAlgorithm == algorithm}
                name="algorithm"
                value={algorithm}
                onChange={(event) => {
                  setSelectedAlgorithm(event.target.value as RoutingAlgorithm)
                }}
              />
              {algorithm}
            </Label>
          )
        })}

        <Label>
          <Checkbox
            checked={includeDebugInfo}
            onChange={(event) => {
              setIncludeDebugInfo(event.target.checked)
            }}
          />
          Include debug info
        </Label>

        <Slider
          min={1}
          max={10}
          value={radiusMultiplier}
          onChange={(value) => {
            setRadiusMultiplier(value)
          }}
          defaultValue={1}
        />

        {!isNil(time) &&
          !isNil(distance) &&
          !isNil(nodesVisited) &&
          !isNil(duration) && (
            <RouteResult
              time={time}
              distance={distance}
              nodesVisited={nodesVisited}
              duration={duration}
            />
          )}
      </MapSidePanel>
    </div>
  )
}
