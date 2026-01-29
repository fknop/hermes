import { PropsWithChildren, ReactNode } from 'react'
import { Label } from './label'
import { SvgIcon } from '../SvgIcon'

const DescriptionValue = ({ children }: PropsWithChildren) => {
  return (
    <span className="text-secondary-foreground font-light">{children}</span>
  )
}

const DescriptionLabel = ({ children }: PropsWithChildren) => {
  return (
    <Label className="text-muted-foreground font-normal text-xs inline-flex items-center gap-1">
      {children}
    </Label>
  )
}

export function DescriptionItem({
  label,
  value,
  icon: Icon,
}: {
  label: ReactNode
  value: ReactNode
  icon?: SvgIcon
}) {
  return (
    <div className="inline-flex flex-row gap-1.5">
      {Icon && (
        <div className="flex flex-col">
          <Icon className="size-3.5 mt-0.5" />
        </div>
      )}
      <div className="flex flex-col">
        <DescriptionLabel>
          <span>{label}</span>
        </DescriptionLabel>
        <DescriptionValue>{value}</DescriptionValue>
      </div>
    </div>
  )
}
