import { SvgIcon } from './SvgIcon'
import { ButtonHTMLAttributes, RefObject } from 'react'
import clsx from 'clsx'

type ButtonVariant = 'primary'
type ButtonSize = 'small' | 'normal'

type ButtonProps = {
  variant: ButtonVariant
  size?: ButtonSize
  icon?: SvgIcon
} & Pick<
  ButtonHTMLAttributes<HTMLButtonElement>,
  'className' | 'onClick' | 'type' | 'children' | 'disabled'
>

/*
<button
      type="button"
      className="rounded bg-indigo-600 px-2 py-1 text-xs font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
    >
      Button text
    </button>*/

function getCommonClassNames() {
  return clsx(
    'flex items-center justify-center gap-3',
    'rounded',
    'font-medium',
    'shadow-sm',
    'focus-visible:outline',
    'focus-visible:outline-2',
    'focus-visible:outline-offset-2'
  )
}

function getVariantClassNames(
  variant: ButtonVariant,
  { disabled }: { disabled: boolean }
) {
  switch (variant) {
    case 'primary':
      return clsx('bg-slate-800', 'hover:bg-slate-700', 'text-white')
    default:
      return ''
  }
}

function getSizeClassNames(size: ButtonSize) {
  switch (size) {
    case 'small':
      return clsx('px-2 py-1 text-xs')
    case 'normal':
      return clsx('px-4 py-2 text-sm')
    default:
      return ''
  }
}

export function Button({
  ref,
  variant,
  size = 'normal',
  icon: Icon,
  type = 'button',
  children,
  className,
  disabled = false,
  ...props
}: ButtonProps & { ref?: RefObject<HTMLButtonElement> }) {
  return (
    <button
      ref={ref}
      type={type}
      className={clsx(
        getCommonClassNames(),
        getVariantClassNames(variant, { disabled }),
        getSizeClassNames(size),
        className
      )}
      disabled={disabled}
      {...props}
    >
      {Icon && <Icon className="w-4 h-4" />}
      {children}
    </button>
  )
}
