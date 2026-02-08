import { usePollJob } from '@/api/generated/hermes'
import { PollResponse } from '@/api/generated/schemas'
import { isNil } from '@/utils/isNil'

export function usePollRouting({
  jobId,
  geojson,
}: {
  jobId: string | null
  geojson?: boolean
}): {
  response: PollResponse | null
  restartPolling: () => void
} {
  const { data: response, refetch } = usePollJob(
    jobId,
    { geojson },
    {
      query: {
        enabled: !isNil(jobId),
        refetchInterval: (query) => {
          const response = query.state.data

          if (response?.status !== 200) {
            return false
          }

          const isCompleted = response.data.status === 'Completed'
          const isPending = response.data.status === 'Pending'

          if (isPending) {
            return 2000 // Poll every 2 seconds if pending
          }

          return isCompleted ? false : 600
        },
      },
    }
  )

  // const [response, setResponse] = useState<SolutionResponse | null>(null)
  // const [error, setError] = useState<string | null>(null)

  // useEffect(() => {
  //   if (isCompleted || isNil(jobId)) {
  //     return
  //   }

  //   async function run() {
  //     try {
  //       const response = await fetch(`${API_URL}/vrp/jobs/${jobId}/poll`)
  //       if (response.status >= 400) {
  //         setError(`Failed ${response.status}`)
  //         return
  //       }
  //       const data: SolutionResponse = await response.json()
  //       setResponse(data)
  //     } catch (error) {
  //       console.error('Error fetching routing solution:', error)
  //     }
  //   }

  //   const interval = setInterval(run, 600)

  //   void run()

  //   return () => {
  //     clearInterval(interval)
  //   }
  // }, [isCompleted, jobId])

  return {
    response: response?.status === 200 ? response.data : null,
    restartPolling: refetch,
  }
}
