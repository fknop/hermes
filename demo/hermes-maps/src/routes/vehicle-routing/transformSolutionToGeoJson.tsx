import { SolutionResponse } from './usePollRouting'
import { POST_BODY } from './usePostRouting'

export function transformSolutionToGeoJson(
  problem: typeof POST_BODY,
  { solution }: SolutionResponse
): { points: GeoJSON.FeatureCollection<GeoJSON.MultiPoint> } {
  const getLocationForServiceId = (serviceId: number): [number, number] => {
    const locationId = problem.services[serviceId].location_id
    return [
      problem.locations[locationId].lon,
      problem.locations[locationId].lat,
    ]
  }

  const points: GeoJSON.Feature<GeoJSON.MultiPoint>[] = solution.routes.map(
    (route, index) => {
      return {
        geometry: {
          type: 'MultiPoint',
          coordinates: route.activities
            .filter((activity) => activity.type === 'Service')
            .map((activity) => {
              return getLocationForServiceId(activity.service_id)
            }),
        },
        type: 'Feature',
        properties: {
          id: index.toString(),
        },
      }
    }
  )

  return {
    points: {
      type: 'FeatureCollection',
      features: points,
    },
  }
}
