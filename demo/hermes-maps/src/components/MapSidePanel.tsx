import clsx from 'clsx'
import { PropsWithChildren } from 'react'

export function MapSidePanel({
  children,
  side = 'left',
}: PropsWithChildren<{ side?: 'left' | 'right' }>) {
  return (
    <div
      className={clsx(
        'z-10 absolute top-0 bottom-0 bg-white drop-shadow-xs border-zinc-900/20 min-w-96',
        {
          'left-0 border-r-2': side === 'left',
          'right-0 border-l-2': side === 'right',
        }
      )}
    >
      <div className="flex flex-col h-full">{children}</div>
    </div>
  )
}
