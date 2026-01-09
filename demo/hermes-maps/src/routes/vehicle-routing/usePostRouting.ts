import { useCallback } from 'react'
import sample from './sample/sample.json'
import { useFetch } from '../../hooks/useFetch'

export const POST_BODY = sample

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
