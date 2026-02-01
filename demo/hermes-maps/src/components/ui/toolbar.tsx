'use client'

import { Toolbar as ToolbarPrimitive } from '@base-ui/react/toolbar'
import { Menu as MenuPrimitive } from '@base-ui/react/menu'
import * as React from 'react'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { cn } from '@/lib/utils'
import { Separator } from './separator'
import { SvgIcon } from '../SvgIcon'
import { CheckIcon } from 'lucide-react'

function Toolbar({ className, ...props }: ToolbarPrimitive.Root.Props) {
  return (
    <ToolbarPrimitive.Root
      data-slot="menubar"
      className={cn(
        'bg-background h-9 rounded-lg border p-1 flex items-center',
        className
      )}
      {...props}
    />
  )
}

function ToolbarMenu({ ...props }: React.ComponentProps<typeof DropdownMenu>) {
  return <DropdownMenu data-slot="toolbar-menu" {...props} />
}

function ToolbarMenuTrigger({
  className,
  ...props
}: React.ComponentProps<typeof DropdownMenuTrigger>) {
  return <ToolbarButton render={<DropdownMenuTrigger {...props} />} />
}

function ToolbarMenuContent({
  className,
  align = 'start',
  alignOffset = -4,
  sideOffset = 8,
  ...props
}: React.ComponentProps<typeof DropdownMenuContent>) {
  return (
    <DropdownMenuContent
      data-slot="toolbar-content"
      align={align}
      alignOffset={alignOffset}
      sideOffset={sideOffset}
      className={cn(
        'bg-popover text-popover-foreground data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 ring-foreground/10 min-w-32 rounded-lg p-1 shadow-md ring-1 duration-100 data-[side=inline-start]:slide-in-from-right-2 data-[side=inline-end]:slide-in-from-left-2',
        className
      )}
      {...props}
    />
  )
}

function ToolbarMenuCheckboxItem({
  className,
  children,
  checked,
  icon: Icon = CheckIcon,
  ...props
}: MenuPrimitive.CheckboxItem.Props & { icon?: SvgIcon }) {
  return (
    <MenuPrimitive.CheckboxItem
      data-slot="menubar-checkbox-item"
      className={cn(
        'focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground min-h-7 gap-2 rounded-md py-1.5 pr-2 pl-8 text-xs relative flex cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0',
        className
      )}
      checked={checked}
      {...props}
    >
      <span className="left-2 size-4 [&_svg:not([class*='size-'])]:size-4 pointer-events-none absolute flex items-center justify-center">
        <MenuPrimitive.CheckboxItemIndicator>
          <Icon className="size-3" />
        </MenuPrimitive.CheckboxItemIndicator>
      </span>
      {children}
    </MenuPrimitive.CheckboxItem>
  )
}

function ToolbarButton({
  className,
  ...props
}: React.ComponentProps<typeof ToolbarPrimitive.Button>) {
  return (
    <ToolbarPrimitive.Button
      className={cn(
        'hover:bg-muted aria-expanded:bg-muted rounded-[calc(var(--radius-md)-2px)] px-2 py-[calc(--spacing(0.85))] text-xs/relaxed font-medium flex items-center outline-hidden select-none',
        className
      )}
      {...props}
    />
  )
}

function ToolbarGroup({
  ...props
}: React.ComponentProps<typeof ToolbarPrimitive.Group>) {
  return <ToolbarPrimitive.Group data-slot="toolbar-group" {...props} />
}

function ToolbarSeparator({
  ...props
}: React.ComponentProps<typeof ToolbarPrimitive.Separator>) {
  return (
    <ToolbarPrimitive.Separator
      data-slot="toolbar-separator"
      render={<Separator orientation="vertical" className="mx-1" />}
      {...props}
    />
  )
}

export {
  Toolbar,
  ToolbarButton,
  ToolbarGroup,
  ToolbarMenu,
  ToolbarMenuTrigger,
  ToolbarMenuContent,
  ToolbarSeparator,
  ToolbarMenuCheckboxItem,
}
