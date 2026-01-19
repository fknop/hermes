import { Source } from 'react-map-gl/mapbox'
import { Button } from '../../components/Button.tsx'
import { MapSidePanel } from '../../components/MapSidePanel.tsx'
import { Map } from '../../Map.tsx'
import { transformSolutionToGeoJson, getGeoJSONFromProblem } from './geojson.ts'
import { usePollRouting } from './usePollRouting.ts'
import { usePostRouting } from './usePostRouting.ts'
import { PolylineLayer } from '../../PolylineLayer.tsx'
import { ActivitiesLayer } from './ActivityLayer.tsx'
import { VRP_COLORS } from './colors.ts'
import { VehicleRoutingProblem } from './input.ts'
import { JsonFileUpload } from './JsonFileUpload.tsx'
import { useMemo, useState } from 'react'
import { isNil } from '../../utils/isNil.ts'
import { LocationsLayer } from './LocationsLayer.tsx'
import { StatisticsPanel } from './StatisticsPanel.tsx'
import { WeightsPanel } from './WeightsPanel.tsx'
import { RoutesPanel } from './components/RoutesPanel.tsx'
import { UnassignedJobsPanel } from './components/UnassignedJobsPanel.tsx'
import { ActivitiesPanel } from './components/ActivitiesPanel.tsx'

export default function VehicleRoutingScreen() {
  const [input, setInput] = useState<VehicleRoutingProblem | null>(null)
  const [postRouting, { loading, data }] = usePostRouting()
  const { response } = usePollRouting({ jobId: data?.job_id ?? null })
  const [selectedRouteIndex, setSelectedRouteIndex] = useState<number | null>(
    null
  )

  const polling = response?.status === 'Running'

  const solutionGeoJson =
    response?.solution && !isNil(input)
      ? transformSolutionToGeoJson(input, response.solution)
      : null

  const problemGeoJson = !isNil(input) ? getGeoJSONFromProblem(input) : null

  const unassignedServices = useMemo(() => {
    if (!input || !response?.solution) return []

    const unassignedServiceIds = response.solution.unassigned_jobs

    return input.services.filter((_, index) =>
      unassignedServiceIds.includes(index)
    )
  }, [input, response?.solution])

  const selectedRoute =
    selectedRouteIndex !== null
      ? response?.solution?.routes[selectedRouteIndex]
      : null

  return (
    <div className="h-screen w-screen">
      <Map>
        <div className="z-10 absolute top-0 bottom-0 left-0 flex">
          <MapSidePanel side="left">
            <div className="flex flex-row h-full">
              <div className="flex flex-col h-full overflow-hidden w-full flex-shrink-0">
                <div className="flex flex-col gap-4 px-6 py-6 flex-shrink-0">
                  <JsonFileUpload
                    onFileUpload={async (file) => {
                      const data = await file.text()
                      setInput(JSON.parse(data))
                    }}
                  />

                  <Button
                    variant="primary"
                    disabled={loading || polling || isNil(input)}
                    onClick={() => {
                      if (!isNil(input)) {
                        postRouting(input)
                      }
                    }}
                  >
                    {polling ? 'Running...' : 'Start'}
                  </Button>
                </div>
                <div className="flex-1 overflow-auto pb-6">
                  <div className="flex flex-col gap-6">
                    {response?.solution && input && (
                      <>
                        <RoutesPanel
                          solution={response.solution}
                          selectedRouteIndex={selectedRouteIndex}
                          onRouteSelect={setSelectedRouteIndex}
                          problem={input}
                        />
                        <UnassignedJobsPanel
                          unassignedServices={unassignedServices}
                        />
                      </>
                    )}
                  </div>
                </div>
              </div>
              {selectedRoute && selectedRouteIndex !== null && (
                <ActivitiesPanel
                  route={selectedRoute}
                  routeIndex={selectedRouteIndex}
                  color={VRP_COLORS[selectedRouteIndex % VRP_COLORS.length]}
                  onClose={() => setSelectedRouteIndex(null)}
                />
              )}
            </div>
          </MapSidePanel>
        </div>

        {response?.statistics && response?.weights && (
          <MapSidePanel side="right">
            <div className="p-4">
              <StatisticsPanel statistics={response.statistics} />
              <WeightsPanel weights={response.weights} />
            </div>
          </MapSidePanel>
        )}

        {response && (
          <>
            {response.solution?.routes.map((route, index) => {
              return (
                <Source
                  key={index}
                  type="geojson"
                  data={route.polyline}
                  id={`polyline-${index}`}
                >
                  <PolylineLayer
                    id={`polyline-${index}`}
                    color={VRP_COLORS[index % VRP_COLORS.length]}
                    sourceId={`polyline-${index}`}
                    lineWidth={3}
                  />
                </Source>
              )
            })}
          </>
        )}

        {solutionGeoJson && (
          <>
            <Source type="geojson" data={solutionGeoJson.points} id="geojson">
              <ActivitiesLayer id="activities" sourceId="geojson" />
            </Source>
          </>
        )}

        {problemGeoJson && (
          <>
            <Source
              type="geojson"
              data={problemGeoJson.points}
              id="locations-geojson"
            >
              <LocationsLayer id="locations" sourceId="locations-geojson" />
            </Source>
          </>
        )}
      </Map>
    </div>
  )
}
