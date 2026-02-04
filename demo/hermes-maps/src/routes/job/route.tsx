import { getJob, useCreateJob, useStartJob } from '@/api/generated/hermes.ts'
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
import { getRouteColor } from './colors.ts'
import { ActivitiesPanel } from './components/ActivitiesPanel.tsx'
import { RoutesPanel } from './components/RoutesPanel.tsx'
import { RoutingJobContextProvider } from './components/RoutingJobContext.tsx'
import { UnassignedJobsPanel } from './components/UnassignedJobsPanel.tsx'
import { VehicleRoutingToolbar } from './components/VehicleRoutingToolbar.tsx'
import { getGeoJSONFromProblem, transformSolutionToGeoJson } from './geojson.ts'
import { VehicleRoutingProblem } from './input.ts'
import { LocationsLayer } from './LocationsLayer.tsx'
import { UnassignedJobsLayer } from './UnassignedJobsLayer.tsx'
import { usePollRouting } from './usePollRouting.ts'
import { useStopRouting } from './useStopRouting.ts'
import { getSolution } from './solution.ts'
import { ClientLoaderFunctionArgs, useLoaderData } from 'react-router'

export async function clientLoader({ params }: ClientLoaderFunctionArgs) {
  const jobId = params.jobId
  const data = await getJob(jobId)
  return { input: data.data, jobId: data.data.id }
}

export default function VehicleRoutingScreen() {
  const { input, jobId } = useLoaderData<typeof clientLoader>()

  const [showUnassigned, setShowUnassigned] = useState(false)

  const { mutateAsync: startJob, isPending: isStarting } = useStartJob()

  const stopRouting = useStopRouting()
  const { response, restartPolling } = usePollRouting({ jobId })
  const solution = getSolution(response)
  const [selectedRouteIndex, setSelectedRouteIndex] = useState<number | null>(
    null
  )
  const [hiddenRoutes, setHiddenRoutes] = useState<Set<number>>(new Set())
  const toggleRoute = useCallback((route: number) => {
    setHiddenRoutes((prev) => {
      const newSet = new Set(prev)
      if (newSet.has(route)) {
        newSet.delete(route)
      } else {
        newSet.add(route)
      }
      return newSet
    })
  }, [])

  const hideOtherRoutes = useCallback(
    (route: number) => {
      setHiddenRoutes(
        new Set(
          Array.from(
            { length: solution?.routes.length ?? 0 },
            (_, i) => i
          ).filter((i) => i !== route)
        )
      )
    },
    [response]
  )

  const showAllRoutes = useCallback(() => {
    setHiddenRoutes(new Set())
  }, [])

  const polling = response?.status === 'Running'

  const solutionGeoJson = useMemo(() => {
    return !isNil(solution) && !isNil(input)
      ? transformSolutionToGeoJson(input, solution)
      : null
  }, [solution, input])

  const { locations, depots } = useMemo(
    () => getGeoJSONFromProblem(input),
    [input]
  )

  const unassignedServices = useMemo(() => {
    if (!input || !solution) return []

    const unassignedServiceIds = new Set(solution.unassigned_jobs)

    return input.services.filter((service) =>
      unassignedServiceIds.has(service.id)
    )
  }, [input, solution])

  const selectedRoute =
    selectedRouteIndex !== null ? solution?.routes[selectedRouteIndex] : null

  const bounds = useMapboxBounds(
    useMemo(() => {
      if (!input) {
        return []
      }

      const allCoordinates: [number, number][] = input.locations.map(
        (location) => location.coordinates as [number, number]
      )

      return allCoordinates
    }, [input])
  )

  return (
    <RoutingJobContextProvider
      value={{
        jobId,
        response,
        input: input,
        isStarting: isStarting,
        startRouting: useCallback(async () => {
          await startJob({ jobId })
          restartPolling()
        }, [startJob, input, jobId, restartPolling]),
        stopRouting: useCallback(async () => {
          if (!isNil(jobId)) {
            await stopRouting({ jobId })
          }
        }, [jobId]),
        // onInputChange: setInput,
        isRunning: polling,
        showUnassigned,
        setShowUnassigned,
        showAllRoutes,
        hideOtherRoutes,
        toggleRoute,
        hiddenRoutes,
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
                      {solution && input && (
                        <>
                          <RoutesPanel
                            solution={solution}
                            selectedRouteIndex={selectedRouteIndex}
                            onRouteSelect={setSelectedRouteIndex}
                            problem={input}
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
                  onClose={() => setSelectedRouteIndex(null)}
                />
              </MapSidePanel>
            </ResizablePanel>
          )}
          {showUnassigned && (
            <ResizablePanel defaultSize={72 * 4} minSize={72 * 4}>
              <MapSidePanel side="left">
                <UnassignedJobsPanel unassignedServices={unassignedServices} />
              </MapSidePanel>
            </ResizablePanel>
          )}
          <ResizablePanel>
            <div className="flex flex-col flex-1 h-full">
              <VehicleRoutingToolbar />
              <Map bounds={bounds}>
                {solutionGeoJson && (
                  <>
                    <Source
                      type="geojson"
                      data={solutionGeoJson.assignedLocations}
                      id="assigned-locations"
                    >
                      <ActivitiesLayer
                        id="activities"
                        sourceId="assigned-locations"
                        hiddenRoutes={hiddenRoutes}
                      />
                    </Source>

                    <Source
                      type="geojson"
                      data={solutionGeoJson.unassignedLocations}
                      id="unassigned-locations"
                    >
                      <UnassignedJobsLayer
                        beforeId="activities"
                        id="unassigned-locations"
                        sourceId="unassigned-locations"
                      />
                    </Source>
                  </>
                )}

                {solution && (
                  <>
                    {solution.routes.map((route, index) => {
                      const isHidden = hiddenRoutes.has(index)

                      if (isHidden) {
                        return null
                      }

                      return (
                        <Source
                          key={index}
                          type="geojson"
                          data={route.polyline}
                          id={`polyline-${index}`}
                        >
                          <PolylineLayer
                            id={`polyline-${index}`}
                            beforeId="activities"
                            color={getRouteColor(index)}
                            sourceId={`polyline-${index}`}
                            lineWidth={3}
                          />
                        </Source>
                      )
                    })}
                  </>
                )}

                {isNil(solutionGeoJson) && (
                  <>
                    <Source
                      type="geojson"
                      data={locations.points}
                      id="locations-geojson"
                    >
                      <LocationsLayer
                        id="locations"
                        sourceId="locations-geojson"
                      />
                    </Source>
                  </>
                )}

                <Source type="geojson" data={depots.points} id="depots-geojson">
                  <LocationsLayer id="depots" sourceId="depots-geojson" />
                </Source>
              </Map>
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </RoutingJobContextProvider>
  )
}
