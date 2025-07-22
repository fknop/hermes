import { Source } from 'react-map-gl/mapbox'
import { Button } from '../../components/Button.tsx'
import { MapSidePanel } from '../../components/MapSidePanel.tsx'
import { Map } from '../../Map.tsx'
import { MultiPointLayer } from '../../MultiPointLayer.tsx'
import { transformSolutionToGeoJson } from './transformSolutionToGeoJson.tsx'
import { usePollRouting } from './usePollRouting.ts'
import { POST_BODY, usePostRouting } from './usePostRouting.ts'
import { PolylineLayer } from '../../PolylineLayer.tsx'
import { ActivitiesLayer } from './ActivityLayer.tsx'
import { colors } from './colors.ts'

export default function VehicleRoutingScreen() {
  const [postRouting, { loading, data }] = usePostRouting()
  const { solution } = usePollRouting({ jobId: data?.job_id ?? null })

  const polling = solution?.status === 'Running'

  const geojson = solution
    ? transformSolutionToGeoJson(POST_BODY, solution)
    : null

  const totalTime = solution?.solution.duration

  const totalDistance = solution?.solution.routes.reduce(
    (acc, route) => acc + route.distance,
    0
  )

  if (solution) {
    console.log(totalTime, totalDistance / 1000, solution?.solution.score)
  }

  return (
    <div className="h-screen w-screen">
      <Map>
        <MapSidePanel>
          <div className="flex flex-col gap-6">
            <Button
              variant="primary"
              disabled={loading || polling}
              onClick={() => {
                postRouting()
              }}
            >
              Start
            </Button>

            <div className="flex flex-col gap-1">
              {solution?.solution.routes.map((route, index) => {
                return (
                  <div className="flex flex-col">
                    <span className="inline-flex items-center gap-2">
                      <div
                        className="h-4 w-4 rounded-full"
                        style={{
                          backgroundColor:
                            colors[index % solution.solution.routes.length],
                        }}
                      />
                      <span>Route {index + 1}</span>
                    </span>

                    <span>Duration: {route.duration}</span>
                    <span>Distance: {Math.round(route.distance) / 1000}km</span>
                    <span>Waiting duration: {route.waiting_duration}</span>
                    <span>Activities: {route.activities.length}</span>
                    <span>Load: {route.vehicle_max_load * 100}%</span>
                  </div>
                )
              })}
            </div>
          </div>
        </MapSidePanel>

        {solution && (
          <>
            {solution.solution.routes.map((route, index) => {
              return (
                <Source
                  key={index}
                  type="geojson"
                  data={route.polyline}
                  id={`polyline-${index}`}
                >
                  <PolylineLayer
                    id={`polyline-${index}`}
                    color={colors[index % colors.length]}
                    sourceId={`polyline-${index}`}
                    lineWidth={3}
                  />
                </Source>
              )
            })}
          </>
        )}

        {geojson && (
          <>
            <Source type="geojson" data={geojson.points} id="geojson">
              <ActivitiesLayer id="activities" sourceId="geojson" />
            </Source>
          </>
        )}
      </Map>
    </div>
  )
}
