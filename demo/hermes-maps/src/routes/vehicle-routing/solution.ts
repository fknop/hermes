export type Activity =
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
      id: string
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
    vehicle_id: string
    activities: Activity[]
    polyline: GeoJSON.Feature<GeoJSON.LineString>
  }[]
  unassigned_jobs: number[]
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

export type AlnsWeights = { weights: { strategy: string; weight: number }[] }

export type OperatorWeights = {
  ruin: AlnsWeights
  recreate: AlnsWeights
}

export type SolutionStatistics = {
  aggregated_ruin_statistics: { [name: string]: OperatorStatistics }
  aggregated_recreate_statistics: { [name: string]: OperatorStatistics }
}

export type SolutionPending = {
  status: 'Pending'
  solution: undefined
  statistics: undefined
  weights: undefined
}

export type SolutionRunning = {
  status: 'Running'
  solution: Solution | null
  statistics: SolutionStatistics
  weights: OperatorWeights
}

export type SolutionCompleted = {
  status: 'Completed'
  solution: Solution | null
  statistics: SolutionStatistics
  weights: OperatorWeights
}

export type SolutionResponse =
  | SolutionPending
  | SolutionRunning
  | SolutionCompleted
