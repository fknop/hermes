import { createContext, use } from 'react'
import { VehicleRoutingProblem } from '../input'
import { PollResponse } from '@/api/generated/schemas'

type RoutingJobContextType = {
  jobId: string | null
  input: VehicleRoutingProblem | null
  response: PollResponse | null
  onInputChange: (input: VehicleRoutingProblem) => void
  isStarting: boolean
  startRouting: () => Promise<void>
  stopRouting: () => Promise<void>
  isRunning: boolean
  showUnassigned: boolean
  setShowUnassigned: (show: boolean) => void
  showAllRoutes: () => void
  toggleRoute: (route: number) => void
  hideOtherRoutes: (route: number) => void
  hiddenRoutes: Set<number>
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
