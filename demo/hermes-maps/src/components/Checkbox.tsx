import clsx from 'clsx'
import React from 'react'

export function Checkbox({
  name,
  ref,
  checked,
  defaultChecked,
  onChange,
}: {
  name?: string
  ref?: React.Ref<HTMLInputElement>
  checked?: boolean
  defaultChecked?: boolean
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void
}) {
  return (
    <input
      ref={ref}
      checked={checked}
      defaultChecked={defaultChecked}
      onChange={onChange}
      name={name}
      type="checkbox"
      className={clsx(
        'bg-white checked:border-primary checked:bg-primary intermediate:border-primary intermediate:bg-primary',
        'focus-visible:outline-primary-active disabled:border-gray-300 disabled:bg-gray-100 disabled:checked:bg-gray-100',
        'focus:outline-primary-active',
        'col-start-1 row-start-1 appearance-none rounded border border-gray-300 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2  forced-colors:appearance-auto'
      )}
    />
  )
}
