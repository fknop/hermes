import { VehicleRoutingJobInput } from '@/api/generated/schemas'
import { Button } from '@/components/ui/button'

interface LocationsPanelProps {
  problem: VehicleRoutingJobInput
  onSelect?: (locationId: number) => void
}

export function LocationsPanel({ problem, onSelect }: LocationsPanelProps) {
  return (
    <div className="flex flex-col gap-4 mt-3">
      <div className="flex flex-col gap-2 px-3">
        <div className="flex flex-col divide-y divide-border overflow-hidden rounded-lg border border-border">
          {problem.locations.map((location, index) => (
            <div key={index}>
              {index}
              <Button variant="ghost" onClick={() => onSelect?.(index)}>
                Select {index}
              </Button>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
