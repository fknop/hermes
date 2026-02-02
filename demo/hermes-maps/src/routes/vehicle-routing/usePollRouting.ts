import { usePollJob } from '@/api/generated/hermes'
import { isNil } from '../../utils/isNil'
import { PollResponse } from '@/api/generated/schemas'

export function usePollRouting({ jobId }: { jobId: string | null }): {
  response: PollResponse | null
} {
  console.log({ jobId })
  const { data } = usePollJob(jobId, {
    query: {
      enabled: !isNil(jobId),
      refetchInterval: 600,
    },
  })

  // const [response, setResponse] = useState<SolutionResponse | null>(null)
  // const [error, setError] = useState<string | null>(null)
  const isCompleted = data?.data.status === 'Completed'

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

  return { response: data?.data ?? null }
}
