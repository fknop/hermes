import { useEffect, useState } from 'react'
import { isNil } from '../../utils/isNil'
import { API_URL } from '../../constants'

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

export type SolutionResponse = {
  status: 'Pending' | 'Running' | 'Completed'
  solution: {
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
  } | null
}

export function usePollRouting({ jobId }: { jobId: string | null }) {
  const [solution, setSolution] = useState<SolutionResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const isCompleted = solution?.status === 'Completed'

  useEffect(() => {
    if (isCompleted || isNil(jobId)) {
      return
    }

    async function run() {
      try {
        const response = await fetch(`${API_URL}/vrp/poll/${jobId}`)
        if (response.status >= 400) {
          setError(`Failed ${response.status}`)
          return
        }
        const data: SolutionResponse = await response.json()
        setSolution(data)
      } catch (error) {
        console.error('Error fetching routing solution:', error)
      }
    }

    const interval = setInterval(run, 1000) // Poll every 5 seconds

    void run()

    return () => {
      clearInterval(interval)
    }
  }, [isCompleted, jobId])

  return { solution }
}
