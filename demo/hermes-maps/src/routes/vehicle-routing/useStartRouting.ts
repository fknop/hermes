import { useCallback } from 'react'
import { API_URL } from '../../constants'

export function useStartRouting() {
  return useCallback(async ({ jobId }: { jobId: string }) => {
    const response = await fetch(`${API_URL}/vrp/jobs/${jobId}/start`, {
      method: 'POST',
    })

    if (response.status >= 400) {
      throw new Error(
        `Failed to start routing job ${jobId}: ${response.status}`
      )
    }

    return true
  }, [])
}
