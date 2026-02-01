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
import { usePollRouting } from './usePollRouting.ts'
import { usePostRouting } from './usePostRouting.ts'
import { useStopRouting } from './useStopRouting.ts'
import { UnassignedJobsLayer } from './UnassignedJobsLayer.tsx'

export default function VehicleRoutingScreen() {
  const [showUnassigned, setShowUnassigned] = useState(false)
  const [input, setInput] = useState<VehicleRoutingProblem | null>(null)
  const [postRouting, { loading, data }] = usePostRouting()
  const stopRouting = useStopRouting()
  const { response } = usePollRouting({ jobId: data?.job_id ?? null })
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
            { length: response?.solution?.routes.length ?? 0 },
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

    const unassignedServiceIds = new Set(response.solution.unassigned_jobs)

    return input.services.filter((service) =>
      unassignedServiceIds.has(service.id)
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
                      {response?.solution && input && (
                        <>
                          <RoutesPanel
                            solution={response.solution}
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

                {response && (
                  <>
                    {response.solution?.routes.map((route, index) => {
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

                {problemGeoJson && isNil(solutionGeoJson) && (
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
