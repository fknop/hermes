import { usePollJob } from '@/api/generated/hermes'
import { PollResponse } from '@/api/generated/schemas'

export function usePollRouting({ jobId }: { jobId: string }): {
  response: PollResponse | null
  restartPolling: () => void
} {
  const { data, refetch } = usePollJob(jobId, {
    query: {
      refetchInterval: (query) => {
        const isCompleted = query.state.data?.data.status === 'Completed'
        const isPending = query.state.data?.data.status === 'Pending'

        if (isPending) {
          return 2000 // Poll every 2 seconds if pending
        }

        return isCompleted ? false : 600
      },
    },
  })

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

  return { response: data?.data ?? null, restartPolling: refetch }
}
