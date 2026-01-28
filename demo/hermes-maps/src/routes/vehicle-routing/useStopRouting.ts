import { useCallback } from 'react'
import { API_URL } from '../../constants'

export function useStopRouting() {
  return useCallback(async ({ jobId }: { jobId: string }) => {
    const response = await fetch(`${API_URL}/vrp/jobs/${jobId}/stop`, {
      method: 'POST',
    })

    if (response.status >= 400) {
      throw new Error(`Failed to stop routing job ${jobId}: ${response.status}`)
    }

    return true
  }, [])
}
