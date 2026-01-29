import {
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable.tsx'
import { useMapboxBounds } from '@/hooks/useMapboxBounds.ts'
import { useCallback, useMemo, useState } from 'react'
import { Source } from 'react-map-gl/mapbox'
import { Map } from '../../components/ui/maps/Map.tsx'
import { MapSidePanel } from '../../components/ui/maps/MapSidePanel.tsx'
import { PolylineLayer } from '../../PolylineLayer.tsx'
import { isNil } from '../../utils/isNil.ts'
import { ActivitiesLayer } from './ActivityLayer.tsx'
import { VRP_COLORS } from './colors.ts'
import { ActivitiesPanel } from './components/ActivitiesPanel.tsx'
import { RoutesPanel } from './components/RoutesPanel.tsx'
import { RoutingJobContextProvider } from './components/RoutingJobContext.tsx'
import { UnassignedJobsPanel } from './components/UnassignedJobsPanel.tsx'
import { VehicleRoutingMenu } from './components/VehicleRoutingMenu.tsx'
import { getGeoJSONFromProblem, transformSolutionToGeoJson } from './geojson.ts'
import { VehicleRoutingProblem } from './input.ts'
import { LocationsLayer } from './LocationsLayer.tsx'
import { usePollRouting } from './usePollRouting.ts'
import { usePostRouting } from './usePostRouting.ts'
import { useStopRouting } from './useStopRouting.ts'

export default function VehicleRoutingScreen() {
  const [input, setInput] = useState<VehicleRoutingProblem | null>(null)
  const [postRouting, { loading, data }] = usePostRouting()
  const stopRouting = useStopRouting()
  const { response } = usePollRouting({ jobId: data?.job_id ?? null })
  const [selectedRouteIndex, setSelectedRouteIndex] = useState<number | null>(
    null
  )

  const startRouting = useCallback(async () => {
    if (!isNil(input)) {
      await postRouting(input)
    }
  }, [postRouting, input])

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

  const bounds = useMapboxBounds(
    useMemo(() => {
      if (!input) {
        return []
      }

      const allCoordinates: [number, number][] = input.locations.map(
        (location) => location.coordinates
      )

      return allCoordinates
    }, [input])
  )

  return (
    <RoutingJobContextProvider
      value={{
        jobId: data?.job_id ?? null,
        response,
        input: input,
        startRouting,
        stopRouting: useCallback(async () => {
          if (!isNil(data?.job_id)) {
            await stopRouting({ jobId: data.job_id })
          }
        }, [data?.job_id]),
        onInputChange: setInput,
        isRunning: polling,
      }}
    >
      <div className="h-screen w-screen">
        <ResizablePanelGroup orientation="horizontal">
          {response && (
            <ResizablePanel defaultSize={84 * 4} minSize={84 * 4}>
              <MapSidePanel side="left">
                <div className="flex flex-row h-full">
                  <div className="flex-1 overflow-auto pb-6">
                    <div className="flex flex-col gap-4">
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
              </MapSidePanel>
            </ResizablePanel>
          )}
          {selectedRoute && selectedRouteIndex !== null && (
            <ResizablePanel defaultSize={84 * 4} minSize={84 * 4}>
              <MapSidePanel side="left">
                <ActivitiesPanel
                  route={selectedRoute}
                  routeIndex={selectedRouteIndex}
                  color={VRP_COLORS[selectedRouteIndex % VRP_COLORS.length]}
                  onClose={() => setSelectedRouteIndex(null)}
                />
              </MapSidePanel>
            </ResizablePanel>
          )}
          <ResizablePanel>
            <div className="flex flex-col flex-1 h-full">
              <VehicleRoutingMenu />
              <Map bounds={bounds}>
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
                    <Source
                      type="geojson"
                      data={solutionGeoJson.points}
                      id="geojson"
                    >
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
                      <LocationsLayer
                        id="locations"
                        sourceId="locations-geojson"
                      />
                    </Source>
                  </>
                )}
              </Map>
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </RoutingJobContextProvider>
  )
}
