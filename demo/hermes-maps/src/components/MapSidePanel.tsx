import { PropsWithChildren } from 'react'

export function MapSidePanel({ children }: PropsWithChildren) {
  return (
    <div className="z-10 absolute top-0 left-0 bottom-0 bg-white drop-shadow-xs border-r-2 border-zinc-900/20 min-w-96 overflow-auto">
      <div className="flex flex-col gap-2.5 px-6 py-6">{children}</div>
    </div>
  )
}
