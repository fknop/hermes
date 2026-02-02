import { ApiSolution, PollResponse } from '@/api/generated/schemas'
import { isNil } from '@/utils/isNil'

export function getSolution(
  response: PollResponse | null | undefined
): ApiSolution | null {
  if (isNil(response)) {
    return null
  }

  switch (response.status) {
    case 'Pending':
      return null
    case 'Running':
      return response.solution ?? null
    case 'Completed':
      return response.solution ?? null
  }
}
