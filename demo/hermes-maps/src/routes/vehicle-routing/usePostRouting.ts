import { useCallback } from 'react'
import body from './sample/data-3.json'
import travel_costs from './sample/travel_costs-3.json'
import { useFetch } from '../../hooks/useFetch'

export const POST_BODY = {
  ...body,
  locations: body.locations.map((location, index) => ({
    ...location,
    id: index,
  })),
  travel_costs,
}

export const usePostRouting = () => {
  const [fetch, { data, loading }] = useFetch<{ job_id: string }>('/vrp')

  return [
    useCallback(async () => {
      await fetch({
        body: POST_BODY,
        method: 'POST',
      })
    }, [fetch]),
    { data, loading },
  ] as const
}
