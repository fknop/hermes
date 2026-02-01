import { getContrastColor } from '@/lib/colors'
import { isNil } from '../../utils/isNil'
import { getRouteColor } from './colors'
import { VehicleRoutingProblem } from './input'
import { Solution } from './solution'

export function transformSolutionToGeoJson(
  problem: VehicleRoutingProblem,
  solution: Solution
): {
  assignedLocations: GeoJSON.FeatureCollection<GeoJSON.Point>
  unassignedLocations: GeoJSON.FeatureCollection<GeoJSON.Point>
} {
  const getLocationForServiceId = (serviceId: string): [number, number] => {
    const service = problem.services.find(
      (service) => service.id === serviceId
    )!
    const locationId = service.location_id
    return [
      problem.locations[locationId].coordinates[0],
      problem.locations[locationId].coordinates[1],
    ]
  }

  const getIndexForService = (serviceId: string): number => {
    return problem.services.findIndex((service) => service.id === serviceId)
  }

  const assignedLocations: GeoJSON.Feature<GeoJSON.Point>[] =
    solution.routes.flatMap((route, routeIndex) => {
      return route.activities
        .filter((activity) => activity.type === 'Service')
        .map((activity, index) => {
          return {
            geometry: {
              type: 'Point',
              coordinates: getLocationForServiceId(activity.id),
            },
            type: 'Feature',
            properties: {
              routeId: routeIndex.toString(),
              activityId: (index + 1).toString(),
              color: getRouteColor(routeIndex),
              textColor: getContrastColor(getRouteColor(routeIndex)),
            },
          }
        })
    })

  const unassignedLocations: GeoJSON.Feature<GeoJSON.Point>[] =
    solution.unassigned_jobs.map((serviceId) => {
      const serviceIndex = getIndexForService(serviceId)
      return {
        geometry: {
          type: 'Point',
          coordinates: getLocationForServiceId(serviceId),
        },
        type: 'Feature' as const,
        properties: {
          jobId: (serviceIndex + 1).toString(),
          color: '#e5e5e5',
          textColor: getContrastColor('#e5e5e5'),
        },
      }
    })

  return {
    assignedLocations: {
      type: 'FeatureCollection',
      features: assignedLocations,
    },
    unassignedLocations: {
      type: 'FeatureCollection',
      features: unassignedLocations,
    },
  }
}

export function getGeoJSONFromProblem(problem: VehicleRoutingProblem): {
  points: GeoJSON.FeatureCollection<GeoJSON.Point>
} {
  const depotLocationIds: number[] = problem.vehicles
    .map((vehicle) => vehicle.depot_location_id)
    .filter((id) => !isNil(id))

  const points: GeoJSON.Feature<GeoJSON.Point>[] = problem.locations.map(
    (location, index) => {
      return {
        geometry: {
          type: 'Point',
          coordinates: location.coordinates,
        },
        type: 'Feature',
        properties: {
          locationId: (index + 1).toString(),
          color: depotLocationIds.includes(index) ? 'black' : '#475569',
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
