import { useEffect, useState } from 'react'
import { isNil } from '../../utils/isNil'
import { API_URL } from '../../constants'
import { SolutionResponse } from './solution'

export function usePollRouting({ jobId }: { jobId: string | null }) {
  const [response, setResponse] = useState<SolutionResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const isCompleted = response?.status === 'Completed'

  useEffect(() => {
    if (isCompleted || isNil(jobId)) {
      return
    }

    async function run() {
      try {
        const response = await fetch(`${API_URL}/vrp/jobs/${jobId}/poll`)
        if (response.status >= 400) {
          setError(`Failed ${response.status}`)
          return
        }
        const data: SolutionResponse = await response.json()
        setResponse(data)
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

  return { response }
}
