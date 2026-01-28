import { PropsWithChildren, ReactNode } from 'react'
import { Label } from './label'

const DescriptionValue = ({ children }: PropsWithChildren) => {
  return (
    <span className="text-secondary-foreground font-medium">{children}</span>
  )
}

const DescriptionLabel = ({ children }: PropsWithChildren) => {
  return (
    <Label className="text-muted-foreground font-normal text-xs">
      {children}
    </Label>
  )
}

export function DescriptionItem({
  label,
  value,
}: {
  label: ReactNode
  value: ReactNode
}) {
  return (
    <div className="flex flex-col">
      <DescriptionLabel>{label}</DescriptionLabel>
      <DescriptionValue>{value}</DescriptionValue>
    </div>
  )
}
