import {
  autoUpdate,
  flip,
  FloatingFocusManager,
  FloatingPortal,
  offset,
  size,
  useDismiss,
  useFloating,
  useInteractions,
  useListNavigation,
  useRole,
} from '@floating-ui/react'
import { Slot } from '@radix-ui/react-slot'
import { clsx } from 'clsx'
import React, {
  HTMLAttributes,
  HTMLProps,
  PropsWithChildren,
  useCallback,
  useContext,
  useLayoutEffect,
  useRef,
  useState,
} from 'react'
import { isNil } from '../utils/isNil'

export type AutocompleteProps = PropsWithChildren<{
  onSelect?: (value: string) => void
  onEmptySelection?: (value: string | null) => void
}>

const useAutocomplete = ({
  onSelect,
  onEmptySelection,
}: {
  onSelect?: (value: string) => void
  onEmptySelection?: (value: string | null) => void
}) => {
  const [open, setOpen] = useState(false)
  const [activeIndex, setActiveIndex] = useState<number | null>(null)

  const listRef = useRef<Array<HTMLElement | null>>([])

  const { refs, strategy, x, y, context } = useFloating<HTMLInputElement>({
    whileElementsMounted: autoUpdate,
    open,
    onOpenChange: setOpen,
    middleware: [
      offset(5),
      flip({ padding: 10 }),
      size({
        apply({ rects, availableHeight, elements }) {
          Object.assign(elements.floating.style, {
            width: `${rects.reference.width}px`,
            maxHeight: `${availableHeight}px`,
          })
        },
        padding: 10,
      }),
    ],
  })

  const role = useRole(context, { role: 'listbox' })
  const dismiss = useDismiss(context)
  const listNav = useListNavigation(context, {
    listRef,
    activeIndex,
    onNavigate: setActiveIndex,
    virtual: true,
    loop: true,
  })

  const { getReferenceProps, getFloatingProps, getItemProps } = useInteractions(
    [role, dismiss, listNav]
  )

  const items = useRef<Map<number, { value: string; label: string }>>(new Map())

  const [onItemClickListeners, setOnItemClickListeners] = useState<
    Set<(value: string) => void>
  >(new Set())

  return {
    getReferenceProps,
    getFloatingProps,
    getItemProps,
    refs,
    strategy,
    x,
    y,
    open,
    setOpen,
    context,
    listRef,
    activeIndex,
    setActiveIndex,
    items,
    registerItem: useCallback(
      (index: number, { value, label }: { value: string; label: string }) => {
        items.current.set(index, { value, label })

        return () => {
          items.current.delete(index)
        }
      },
      []
    ),
    registerOnItemClick: useCallback((listener: (value: string) => void) => {
      setOnItemClickListeners((listeners) => {
        listeners.add(listener)
        return listeners
      })

      return () => {
        setOnItemClickListeners((listeners) => {
          listeners.delete(listener)
          return listeners
        })
      }
    }, []),
    onSelect(item: { label: string; value: string }) {
      onSelect?.(item.value)
      onItemClickListeners.forEach((listener) => listener(item.label))
    },
    onEmptySelection(value: string | null) {
      onEmptySelection?.(value)
      if (value) {
        onItemClickListeners.forEach((listener) => listener(value))
      }
    },
  }
}

type ContextValue = ReturnType<typeof useAutocomplete>

// @ts-ignore
const AutocompleteContext = React.createContext<ContextValue>(undefined)

export function AutocompleteInput({
  asChild,
  value,
  onValueChange,
  children,
}: PropsWithChildren<
  HTMLProps<HTMLInputElement> & {
    asChild?: boolean
    value: string
    onValueChange: (value: string) => void
  }
>) {
  const Component = asChild ? Slot : 'input'
  const {
    getReferenceProps,
    registerOnItemClick,
    refs,
    setOpen,
    activeIndex,
    setActiveIndex,
    items,
    onSelect,
    onEmptySelection,
  } = useContext(AutocompleteContext)

  function onInputChange(event: React.ChangeEvent<HTMLInputElement>) {
    const value = event.target.value
    onValueChange(value)

    if (value) {
      setOpen(true)
      // setActiveIndex(0)
    } else {
      setOpen(false)
    }
  }

  useLayoutEffect(() => {
    return registerOnItemClick(onValueChange)
  }, [registerOnItemClick, onValueChange])

  return (
    <Component
      {...getReferenceProps({
        ref: refs.setReference,
        onChange: onInputChange,
        value,
        children,
        'aria-autocomplete': 'list',
        onKeyDown(event) {
          if (event.key === 'Enter') {
            event.stopPropagation()
            event.preventDefault()
            if (activeIndex !== null && items.current.has(activeIndex)) {
              onSelect(items.current.get(activeIndex)!)
              setActiveIndex(null)
              setOpen(false)
            } else {
              if (!isNil(value) && value !== '') {
                onEmptySelection(value)
                setOpen(false)
              } else {
                onEmptySelection(null)
              }
            }
          }
        },
      })}
    />
  )
}

export const AutocompleteList = ({
  children,
  ...attributes
}: HTMLAttributes<HTMLDivElement>) => {
  const { open, refs, context, getFloatingProps, x, y, strategy } =
    useContext(AutocompleteContext)

  return (
    <FloatingPortal>
      {open && (
        <FloatingFocusManager
          context={context}
          initialFocus={-1}
          visuallyHiddenDismiss
        >
          <div
            {...attributes}
            {...getFloatingProps({
              ref: refs.setFloating,
              style: {
                position: strategy,
                top: y ?? 0,
                left: x ?? 0,
                overflowY: 'auto',
              },
              className:
                'divide-y divide-slate-900/10 z-50 shadow-xl border border-slate-900/25 bg-white rounded-md',
            })}
          >
            {children}
          </div>
        </FloatingFocusManager>
      )}
    </FloatingPortal>
  )
}

export function AutocompleteItem({
  children,
  index,
  label,
  value,
  ...attributes
}: HTMLAttributes<HTMLDivElement> & {
  value: string
  label: string
  index: number
}) {
  const {
    getItemProps,
    listRef,
    setOpen,
    refs,
    activeIndex,
    registerItem,
    onSelect,
  } = useContext(AutocompleteContext)

  useLayoutEffect(() => {
    return registerItem(index, { value, label })
  }, [index, label, value, registerItem])

  const active = activeIndex === index
  return (
    <div
      {...attributes}
      role="option"
      aria-selected={active}
      className={clsx(
        'cursor-default px-2 py-1.5',
        active ? 'bg-slate-100' : 'bg-white'
      )}
      {...getItemProps({
        ref(node) {
          listRef.current[index] = node
        },
        onClick() {
          onSelect({ label, value })
          setOpen(false)
          refs.domReference.current?.focus()
        },
        style: {
          pointerEvents: 'auto',
        },
      })}
    >
      {children}
    </div>
  )
}

export function Autocomplete({
  children,
  onSelect,
  onEmptySelection,
}: AutocompleteProps) {
  const context = useAutocomplete({
    onSelect,
    onEmptySelection: onEmptySelection,
  })

  return (
    <AutocompleteContext.Provider value={context}>
      {children}
    </AutocompleteContext.Provider>
  )
}
