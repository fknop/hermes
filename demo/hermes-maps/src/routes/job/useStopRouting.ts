import { useStopJob } from '@/api/generated/hermes'
import { useCallback } from 'react'

export function useStopRouting() {
  const { mutateAsync: stop } = useStopJob()

  return useCallback(
    async ({ jobId }: { jobId: string }) => {
      const response = await stop({ jobId })

      if (response.status >= 400) {
        throw new Error(
          `Failed to stop routing job ${jobId}: ${response.status}`
        )
      }

      return true
    },
    [stop]
  )
}
