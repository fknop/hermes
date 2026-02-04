import {
  AggregatedStatistics,
  ApiSolution,
  OperatorWeights,
  PollResponse,
} from '@/api/generated/schemas'
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

export function getStatistics(
  response: PollResponse | null | undefined
): AggregatedStatistics | null {
  if (isNil(response)) {
    return null
  }

  switch (response.status) {
    case 'Pending':
      return null
    case 'Running':
      return response.statistics ?? null
    case 'Completed':
      return response.statistics ?? null
  }
}

export function getOperatorWeights(
  response: PollResponse | null | undefined
): OperatorWeights | null {
  if (isNil(response)) {
    return null
  }

  switch (response.status) {
    case 'Pending':
      return null
    case 'Running':
      return response.weights ?? null
    case 'Completed':
      return response.weights ?? null
  }
}
