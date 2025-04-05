import clsx from 'clsx'
import React from 'react'

export function RadioButton({
  name,
  value,
  ref,
  checked,
  defaultChecked,
  onChange,
}: {
  name: string
  value: string
  ref?: React.Ref<HTMLInputElement>
  checked?: boolean
  defaultChecked?: boolean
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void
}) {
  return (
    <input
      defaultChecked={defaultChecked}
      name={name}
      value={value}
      ref={ref}
      onChange={onChange}
      checked={checked}
      type="radio"
      className={clsx(
        'bg-white before:bg-white checked:border-primary checked:bg-primary focus-visible:outline-primary focus:outline-primary',
        'relative size-4 appearance-none rounded-full border border-gray-300 before:absolute before:inset-1 before:rounded-full focus-visible:outline-2 focus-visible:outline-offset-2  disabled:border-gray-300 disabled:bg-gray-100 disabled:before:bg-gray-400 forced-colors:appearance-auto forced-colors:before:hidden [&:not(:checked)]:before:hidden'
      )}
    />
  )
}
