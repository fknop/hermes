import clsx from 'clsx'
import { PropsWithChildren } from 'react'

export function MapSidePanel({
  children,
  side = 'left',
}: PropsWithChildren<{ side?: 'left' | 'right' }>) {
  return (
    <div
      className={clsx(
        'h-full bg-background drop-shadow-xs border-r border-r-sidebar-border',
        {}
      )}
    >
      <div className="flex flex-col h-full">{children}</div>
    </div>
  )
}
