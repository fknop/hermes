import { createContext, use } from 'react'
import { VehicleRoutingProblem } from '../input'
import { SolutionResponse } from '../solution'

type RoutingJobContextType = {
  jobId: string | null
  input: VehicleRoutingProblem | null
  response: SolutionResponse | null
  onInputChange: (input: VehicleRoutingProblem) => void
  startRouting: () => Promise<void>
  stopRouting: () => Promise<void>
  isRunning: boolean
}

const RoutingJobContext = createContext<RoutingJobContextType | null>(null)

export const useRoutingJobContext = (): RoutingJobContextType => {
  return use(RoutingJobContext)!
}

export function RoutingJobContextProvider({
  value,
  children,
}: {
  children: React.ReactNode
  value: RoutingJobContextType | null
}) {
  return (
    <RoutingJobContext.Provider value={value}>
      {children}
    </RoutingJobContext.Provider>
  )
}
