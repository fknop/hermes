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

export type OperatorStatistics = {
  total_invocations: number
  total_improvements: number
  total_best: number
  total_duration: string
  total_score_improvement: number
  total_score_percentage_improvement: number
  avg_duration: string
  avg_score_improvement: number
  avg_score_percentage_improvement: number
}

export type SolutionStatistics = {
  aggregated_ruin_statistics: { [name: string]: OperatorStatistics }
  aggregated_recreate_statistics: { [name: string]: OperatorStatistics }
}

export type SolutionPending = {
  status: 'Pending'
  solution: Solution | null
  statistics: SolutionStatistics | null
}

export type SolutionRunning = {
  status: 'Running'
  solution: Solution | null
  statistics: SolutionStatistics | null
}

export type SolutionCompleted = {
  status: 'Completed'
  solution: Solution
  statistics: SolutionStatistics | null
}

export type SolutionResponse =
  | SolutionPending
  | SolutionRunning
  | SolutionCompleted
