import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Service } from '../input'
import { useRoutingJobContext } from './RoutingJobContext'
import { Badge } from '@/components/ui/badge'
import { DescriptionItem } from '@/components/ui/description-item'
import { useTimeWindowFormatter } from '@/hooks/useTimeWindowFormatter'
import { useDurationFormatter } from '@/hooks/useDurationFormatter'
import { Button } from '@/components/ui/button'
import { XIcon } from 'lucide-react'
import { Separator } from 'react-resizable-panels'

interface UnassignedJobsPanelProps {
  unassignedServices: Service[]
}

export function UnassignedJobsPanel({
  unassignedServices,
}: UnassignedJobsPanelProps) {
  const { input, setShowUnassigned } = useRoutingJobContext()
  const formatTimeWindow = useTimeWindowFormatter()
  const formatDuration = useDurationFormatter()
  if (unassignedServices.length === 0) {
    return null
  }

  const formatServiceIndex = (id: string): string | null => {
    if (!input) {
      return null
    }

    const index = input.services.findIndex((s) => s.id === id)
    return index !== -1 ? (index + 1).toString() : null
  }

  return (
    <div className="flex flex-col h-full bg-popover gap-2   overflow-auto">
      <div className="flex flex-row items-center justify-between py-3 px-3 border-b border-border">
        <h3 className="text-sm font-semibold text-foreground">
          Unassigned Jobs ({unassignedServices.length})
        </h3>
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setShowUnassigned(false)}
        >
          <XIcon className="size-4" />
        </Button>
      </div>
      <div className="bg-card flex flex-col divide-y divide-border">
        {unassignedServices.map((service) => (
          <div key={service.id} className="flex flex-col px-3 py-3">
            <div className="flex flex-row items-center justify-between">
              <span>Service #{formatServiceIndex(service.id)}</span>
              <span>
                <Badge variant="secondary" className="capitalize">
                  {service.type ?? 'Delivery'}
                </Badge>
              </span>
            </div>
            <div className="text-xs grid grid-cols-2 gap-3">
              <DescriptionItem
                label="Time windows"
                value={formatTimeWindow(
                  service.time_windows?.[0].start ?? null,
                  service.time_windows?.[0].end ?? null
                )}
              />
              <DescriptionItem
                label="Duration"
                value={formatDuration(service.duration ?? 0, {
                  style: 'narrow',
                })}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
