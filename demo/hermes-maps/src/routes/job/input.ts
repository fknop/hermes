export type ServiceType = 'pickup' | 'delivery'
export type TravelMatrixProvider =
  | {
      config: {
        gh_profile: GraphHopperProfile
        [k: string]: unknown
      }
      type: 'graph_hopper_api'
      [k: string]: unknown
    }
  | {
      config: {
        speed_kmh: number
        [k: string]: unknown
      }
      type: 'as_the_crow_flies'
      [k: string]: unknown
    }
  | {
      config: {
        matrices: CustomMatrices
        [k: string]: unknown
      }
      type: 'custom'
      [k: string]: unknown
    }
export type GraphHopperProfile =
  | 'car'
  | 'bike'
  | 'foot'
  | 'small_truck'
  | 'truck'

export interface VehicleRoutingProblem {
  id?: string | null
  locations: Location[]
  services: Service[]
  vehicle_profiles: VehicleProfile[]
  vehicles: Vehicle[]
}
export interface Location {
  coordinates: [number, number]
}
export interface Service {
  demand?: number[] | null
  duration?: string | null
  id: string
  location_id: number
  skills?: string[] | null
  time_windows?: TimeWindow[] | null
  type?: ServiceType | null
}
export interface TimeWindow {
  end?: string | null
  start?: string | null
  [k: string]: unknown
}
export interface VehicleProfile {
  cost_provider: TravelMatrixProvider
  id: string
}
export interface CustomMatrices {
  costs: number[][]
  distances: number[][]
  times: number[][]
  [k: string]: unknown
}
export interface Vehicle {
  capacity?: number[] | null
  depot_duration?: string | null
  depot_location_id?: number | null
  id: string
  maximum_activities?: number | null
  profile: string
  return_depot_duration?: string | null
  shift?: VehicleShift | null
  should_return_to_depot?: boolean | null
  skills?: string[] | null
}
export interface VehicleShift {
  earliest_start?: string | null
  latest_end?: string | null
  latest_start?: string | null
  maximum_transport_duration?: string | null
  maximum_working_duration?: string | null
}
