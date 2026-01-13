type Activity =
  | {
      type: 'Start'
      arrival_time: string
      departure_time: string
    }
  | {
      type: 'End'
      arrival_time: string
      departure_time: string
    }
  | {
      type: 'Service'
      service_id: number
      arrival_time: string
      departure_time: string
      waiting_duration: string
    }

export type Solution = {
  score: { soft_score: number; hard_score: number }
  duration: string
  routes: {
    distance: number
    vehicle_max_load: number
    duration: string
    transport_duration: string
    total_demand: number[]
    waiting_duration: string
    vehicle_id: number
    activities: Activity[]
    polyline: GeoJSON.Feature<GeoJSON.LineString>
  }[]
}

export type SolutionPending = {
  status: 'Pending'
  solution: Solution | null
}

export type SolutionRunning = {
  status: 'Running'
  solution: Solution | null
}

export type SolutionCompleted = {
  status: 'Completed'
  solution: Solution
}

export type SolutionResponse =
  | SolutionPending
  | SolutionRunning
  | SolutionCompleted
