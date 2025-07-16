import { useEffect, useState } from 'react'
import { isNil } from '../../utils/isNil'
import { API_URL } from '../../constants'

export type SolutionResponse = {
  status: 'Pending' | 'Running' | 'Completed'
  solution: {
    routes: {
      vehicle_id: number
      activities: {
        service_id: number
        arrival_time: string
        departure_time: string
        waiting_duration: string
      }[]
    }[]
  }
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
