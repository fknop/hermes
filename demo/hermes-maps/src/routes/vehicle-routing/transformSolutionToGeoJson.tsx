import { colors } from './colors'
import { SolutionResponse } from './usePollRouting'
import { POST_BODY } from './usePostRouting'

export function transformSolutionToGeoJson(
  problem: typeof POST_BODY,
  { solution }: SolutionResponse
): { points: GeoJSON.FeatureCollection<GeoJSON.Point> } {
  const getLocationForServiceId = (serviceId: number): [number, number] => {
    const locationId = problem.services[serviceId].location_id
    return [
      problem.locations[locationId].lon,
      problem.locations[locationId].lat,
    ]
  }

  const points: GeoJSON.Feature<GeoJSON.Point>[] = solution.routes.flatMap(
    (route, routeIndex) => {
      return route.activities
        .filter((activity) => activity.type === 'Service')
        .map((activity, index) => {
          return {
            geometry: {
              type: 'Point',
              coordinates: getLocationForServiceId(activity.service_id),
            },
            type: 'Feature',
            properties: {
              routeId: routeIndex.toString(),
              activityId: (index + 1).toString(),
              color: colors[routeIndex % solution.routes.length],
            },
          }
        })
    }
  )

  return {
    points: {
      type: 'FeatureCollection',
      features: points,
    },
  }
}
