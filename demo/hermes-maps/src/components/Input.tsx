import clsx from 'clsx'
import { HTMLAttributes, Ref } from 'react'

type InputProps = HTMLAttributes<HTMLInputElement> & {
  ref?: Ref<HTMLInputElement>
}

export function Input({ ref, ...props }: InputProps) {
  return (
    <input
      {...props}
      ref={ref}
      className={clsx(
        'bg-white border border-slate-900/15 ring-0',
        'w-full rounded-md px-3 py-1.5 text-base text-gray-900',
        'focus:bg-slate-50',
        'placeholder:text-gray-400 focus:outline-2 focus:-outline-offset-2',
        'focus:outline-slate-600 focus-visible:outline-slate-600',
        'sm:text-sm/6'
      )}
    />
  )
}
