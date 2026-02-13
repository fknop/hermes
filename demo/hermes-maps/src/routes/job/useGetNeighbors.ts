import { useGetLocationNeighbors } from '@/api/generated/hermes'
import { JobNeighborsQuery } from '@/api/generated/schemas'
import { isNil } from '@/utils/isNil'
import { useCallback, useState } from 'react'

type Params = JobNeighborsQuery & { job_id: string }

export const useGetNeighbors = (): [
  (params: Params) => void,
  { isPending: boolean; neighbors: number[] | null },
] => {
  const [parameters, setParameters] = useState<Params | null>(null)

  const enabled = !isNil(parameters)

  const { data, isPending } = useGetLocationNeighbors(
    parameters?.job_id,
    { location_id: parameters?.location_id ?? 0 },
    { query: { enabled } }
  )

  const neighbors = data?.status === 200 ? data.data : null

  return [
    useCallback((params: Params) => {
      setParameters(params)
    }, []),
    { isPending, neighbors },
  ]
}
