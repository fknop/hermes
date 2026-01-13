import { useCallback } from 'react'
import { useFetch } from '../../hooks/useFetch'
import { VehicleRoutingProblem } from './input'

export const usePostRouting = () => {
  const [fetch, { data, loading }] = useFetch<{ job_id: string }>('/vrp')

  return [
    useCallback(
      async (body: VehicleRoutingProblem) => {
        await fetch({
          body,
          method: 'POST',
        })
      },
      [fetch]
    ),
    { data, loading },
  ] as const
}
