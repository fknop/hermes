import {
  ButtonHTMLAttributes,
  Children,
  cloneElement,
  forwardRef,
  isValidElement,
  ReactElement,
  Ref,
  useEffect,
  useRef,
  useState,
} from 'react'
import {
  useFloating,
  autoUpdate,
  flip,
  offset,
  shift,
  useRole,
  useDismiss,
  useInteractions,
  useListNavigation,
  useTypeahead,
  FloatingPortal,
  FloatingFocusManager,
  FloatingOverlay,
} from '@floating-ui/react'
import { MapMouseEvent, useMap } from 'react-map-gl/mapbox'
import clsx from 'clsx'
import { GeoPoint } from '../types/GeoPoint'

type MenuItemProps = Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  'onSelect'
> & {
  // label: string
  onSelect?: ({ coordinates }: { coordinates: GeoPoint }) => void
} & { ref?: Ref<HTMLButtonElement> }

export function MapMenuItem({
  ref,
  // label,
  children,
  disabled,
  onSelect, // used by the menu, not passed to the button
  ...props
}: MenuItemProps) {
  return (
    <button
      {...props}
      className={clsx(
        'text-popover-foreground',
        'px-2 py-0.5 rounded text-sm',
        'flex flex-row justify-start',
        'focus:bg-accent focus:text-accent-foreground focus:outline-primary-active focus-visible:outline-primary-active'
      )}
      ref={ref}
      role="menuitem"
      disabled={disabled}
    >
      {children}
    </button>
  )
}

type MenuProps = {
  label?: string
  nested?: boolean
  children: ReactElement<MenuItemProps>[]
} & Omit<React.HTMLProps<HTMLButtonElement>, 'children'>

export const MapContextMenu = ({
  children,
  ref,
}: MenuProps & { ref?: Ref<HTMLButtonElement> }) => {
  const map = useMap()

  const [activeIndex, setActiveIndex] = useState<number | null>(null)
  const [isOpen, setIsOpen] = useState(false)
  const [coordinates, setCoordinates] = useState<GeoPoint | null>(null)

  const listItemsRef = useRef<Array<HTMLButtonElement | null>>([])
  const listContentRef = useRef(
    Children.map(children, (child: ReactElement<MenuItemProps>) =>
      isValidElement(child) ? child.props.children : null
    ) as Array<string | null>
  )
  const allowMouseUpCloseRef = useRef(false)

  const { refs, floatingStyles, context } = useFloating({
    open: isOpen,
    onOpenChange: setIsOpen,
    middleware: [
      offset({ mainAxis: 5, alignmentAxis: 4 }),
      flip({
        fallbackPlacements: ['left-start'],
      }),
      shift({ padding: 10 }),
    ],
    placement: 'right-start',
    strategy: 'fixed',
    whileElementsMounted: autoUpdate,
  })

  const role = useRole(context, { role: 'menu' })
  const dismiss = useDismiss(context)
  const listNavigation = useListNavigation(context, {
    listRef: listItemsRef,
    onNavigate: setActiveIndex,
    activeIndex,
  })
  const typeahead = useTypeahead(context, {
    enabled: isOpen,
    listRef: listContentRef,
    onMatch: setActiveIndex,
    activeIndex,
  })

  const { getFloatingProps, getItemProps } = useInteractions([
    role,
    dismiss,
    listNavigation,
    typeahead,
  ])

  useEffect(() => {
    let timeout: number

    function onContextMenu(event: MapMouseEvent) {
      event.preventDefault()

      refs.setPositionReference({
        getBoundingClientRect() {
          return {
            width: 0,
            height: 0,
            x: event.originalEvent.clientX,
            y: event.originalEvent.clientY,
            top: event.originalEvent.clientY,
            right: event.originalEvent.clientX,
            bottom: event.originalEvent.clientY,
            left: event.originalEvent.clientX,
          }
        },
      })

      setCoordinates({ lat: event.lngLat.lat, lon: event.lngLat.lng })
      setIsOpen(true)
      clearTimeout(timeout)

      allowMouseUpCloseRef.current = false
      timeout = window.setTimeout(() => {
        allowMouseUpCloseRef.current = true
      }, 300)
    }

    function onMouseUp() {
      if (allowMouseUpCloseRef.current) {
        setIsOpen(false)
        setCoordinates(null)
      }
    }

    map.current?.on('contextmenu', onContextMenu)

    // document.addEventListener('contextmenu', onContextMenu)
    document.addEventListener('mouseup', onMouseUp)
    return () => {
      map.current?.off('contextmenu', onContextMenu)
      // document.removeEventListener('contextmenu', onContextMenu)
      document.removeEventListener('mouseup', onMouseUp)
      clearTimeout(timeout)
    }
  }, [refs])

  return (
    <FloatingPortal>
      {isOpen && (
        <FloatingOverlay lockScroll>
          <FloatingFocusManager context={context} initialFocus={refs.floating}>
            <div
              className={clsx(
                'p-1 rounded-md shadow-md',
                'border border-slate-900/25',
                'flex flex-col justify-start',
                'bg-popover',
                'outline:none focus:outline-none',
                'min-w-36'
              )}
              ref={refs.setFloating}
              style={floatingStyles}
              {...getFloatingProps()}
            >
              {Children.map(
                children,
                (child: ReactElement<MenuItemProps>, index) =>
                  isValidElement(child) &&
                  cloneElement(
                    child,
                    getItemProps({
                      tabIndex: activeIndex === index ? 0 : -1,
                      ref(node: HTMLButtonElement) {
                        listItemsRef.current[index] = node
                      },
                      onClick() {
                        if (coordinates) {
                          child.props.onSelect?.({ coordinates })
                        }
                        setIsOpen(false)
                        setCoordinates(null)
                      },
                      onMouseUp() {
                        if (coordinates) {
                          child.props.onSelect?.({ coordinates })
                        }
                        setIsOpen(false)
                        setCoordinates(null)
                      },
                    })
                  )
              )}
            </div>
          </FloatingFocusManager>
        </FloatingOverlay>
      )}
    </FloatingPortal>
  )
}
