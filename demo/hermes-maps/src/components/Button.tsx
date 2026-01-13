import { SvgIcon } from './SvgIcon'
import { ButtonHTMLAttributes, RefObject } from 'react'
import clsx from 'clsx'
import { isNil } from '../utils/isNil'

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

function getCommonClassNames({ isIconButton }: { isIconButton: boolean }) {
  return clsx(
    'flex items-center justify-center',
    isIconButton ? 'rounded-full' : 'rounded-lg',
    'font-semibold',
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
      return clsx(
        'bg-slate-800 disabled:bg-slate-700/50',
        'hover:bg-slate-700',
        'text-white disabled:text-white/50'
      )
    default:
      return ''
  }
}

function getSizeClassNames(size: ButtonSize) {
  switch (size) {
    case 'small':
      return clsx('px-2 py-1 text-xs gap-2')
    case 'normal':
      return clsx('px-4 py-2 text-sm gap-3')
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
  const isIconButton = !isNil(Icon) && isNil(children)
  return (
    <button
      ref={ref}
      type={type}
      className={clsx(
        getCommonClassNames({ isIconButton }),
        getVariantClassNames(variant, { disabled }),
        getSizeClassNames(size),
        className
      )}
      disabled={disabled}
      {...props}
    >
      {Icon && (
        <span>
          <Icon className="size-3.5 text-white" />
        </span>
      )}
      {children}
    </button>
  )
}
