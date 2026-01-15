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
import { Temporal } from 'temporal-polyfill'
import { VehicleRoutingProblem } from './input.ts'
import { JsonFileUpload } from './JsonFileUpload.tsx'
import { useState } from 'react'
import { isNil } from '../../utils/isNil.ts'
import { LocationsLayer } from './LocationsLayer.tsx'
import { StatisticsPanel } from './StatisticsPanel.tsx'
import { WeightsPanel } from './WeightsPanel.tsx'

export default function VehicleRoutingScreen() {
  const [input, setInput] = useState<VehicleRoutingProblem | null>(null)
  const [postRouting, { loading, data }] = usePostRouting()
  const { response } = usePollRouting({ jobId: data?.job_id ?? null })

  const polling = response?.status === 'Running'

  const solutionGeoJson =
    response?.solution && !isNil(input)
      ? transformSolutionToGeoJson(input, response.solution)
      : null

  const problemGeoJson = !isNil(input) ? getGeoJSONFromProblem(input) : null

  const totalTime = response?.solution?.duration

  const totalDistance =
    response?.solution?.routes.reduce(
      (acc, route) => acc + route.distance,
      0
    ) ?? 0

  const totalTransportDuration = response?.solution?.routes.reduce(
    (acc, route) => acc.add(Temporal.Duration.from(route.transport_duration)),
    Temporal.Duration.from({ seconds: 0 })
  )

  return (
    <div className="h-screen w-screen">
      <Map>
        <MapSidePanel side="left">
          <JsonFileUpload
            onFileUpload={async (file) => {
              const data = await file.text()
              setInput(JSON.parse(data))
            }}
          />
          <div className="flex flex-col gap-6">
            <Button
              variant="primary"
              disabled={loading || polling || isNil(input)}
              onClick={() => {
                if (!isNil(input)) {
                  postRouting(input)
                }
              }}
            >
              Start
            </Button>

            <div className="flex flex-col gap-1">
              <div>{response?.solution?.duration}</div>
              <div>
                {response?.solution
                  ? response.solution.routes.reduce(
                      (acc, route) => acc + route.distance,
                      0
                    ) / 1000
                  : 'N/A'}
                km
              </div>
              <div>
                {response?.solution
                  ? response.solution.routes
                      .reduce(
                        (acc, route) =>
                          acc.add(
                            Temporal.Duration.from(route.transport_duration)
                          ),
                        Temporal.Duration.from({ minutes: 0 })
                      )
                      .toString()
                  : 'N/A'}
              </div>

              {response?.solution?.routes.map((route, index) => {
                return (
                  <div className="flex flex-col">
                    <span className="inline-flex items-center gap-2">
                      <div
                        className="h-4 w-4 rounded-full"
                        style={{
                          backgroundColor:
                            VRP_COLORS[
                              index % response.solution!.routes.length
                            ],
                        }}
                      />
                      <span>Route {index + 1}</span>
                    </span>

                    <span>Start: {route.activities[0].arrival_time}</span>
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

        {response?.statistics && response?.weights && (
          <MapSidePanel side="right">
            <StatisticsPanel statistics={response.statistics} />
            <WeightsPanel weights={response.weights} />
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
