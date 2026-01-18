import { Service } from '../input'

interface UnassignedJobsPanelProps {
  unassignedServices: Service[]
}

export function UnassignedJobsPanel({
  unassignedServices,
}: UnassignedJobsPanelProps) {
  if (unassignedServices.length === 0) {
    return null
  }

  return (
    <div className="flex flex-col gap-2">
      <h3 className="text-sm font-semibold text-zinc-700 uppercase tracking-wide">
        Unassigned Jobs ({unassignedServices.length})
      </h3>
      <div className="flex flex-col gap-1.5 p-3 bg-amber-50 border border-amber-200 rounded-lg">
        {unassignedServices.map((service) => (
          <div
            key={service.id}
            className="flex items-center justify-between text-sm"
          >
            <span className="font-medium text-amber-900">{service.id}</span>
            <div className="flex items-center gap-2">
              {service.type && (
                <span className="text-xs bg-amber-100 text-amber-700 px-2 py-0.5 rounded-full capitalize">
                  {service.type}
                </span>
              )}
              <span className="text-amber-600 text-xs">
                Location #{service.location_id}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
