import {
  ArrowsUpDownIcon,
  ArrowTurnDownRightIcon,
} from '@heroicons/react/16/solid'
import { Address } from '../types/Address'
import { AddressAutocomplete } from './AddressAutocomplete'
import { HTMLAttributes, Ref } from 'react'
import clsx from 'clsx'
import { MagnifyingGlassIcon } from '@heroicons/react/24/solid'
import { Button } from './ui/button'
import { ArrowDownUpIcon, ArrowsUpFromLine } from 'lucide-react'
import { Input } from './ui/input'

export function AddressInput({
  ref,
  ...props
}: HTMLAttributes<HTMLInputElement> & {
  ref?: Ref<HTMLInputElement>
}) {
  return (
    <input
      {...props}
      ref={ref}
      className={clsx(
        'bg-input border not-first-of-type:-my-px border-input ring-0',
        'w-full first-of-type:rounded-t-lg last-of-type:border-b-0 px-3 py-1.5 text-base text-foreground',
        'focus:bg-secondary',
        'placeholder:text-muted-foreground focus:outline-none',
        'sm:text-sm/6 not-first:z-5 focus:z-6'
      )}
    />
  )
}

export function JourneyAutocomplete({
  start,
  end,
  onChange,
  onSearch,
}: {
  start: Address | null
  end: Address | null
  onChange: (start: Address | null, end: Address | null) => void
  onSearch: () => void
}) {
  return (
    <div className="flex flex-col">
      <div className="relative">
        <div className="flex flex-col divide-y divide-border flex-1">
          <AddressAutocomplete
            placeholder="From"
            InputComponent={AddressInput}
            value={start?.address ?? ''}
            onRetrieve={async (response) => {
              const [lon, lat] = response.features[0].geometry.coordinates
              onChange(
                {
                  coordinates: { lat, lon },
                  address: response.features[0].properties.full_address,
                },
                end
              )
            }}
          />
          <AddressAutocomplete
            placeholder="To"
            InputComponent={AddressInput}
            value={end?.address ?? ''}
            onRetrieve={async (response) => {
              const [lon, lat] = response.features[0].geometry.coordinates
              onChange(start, {
                coordinates: { lat, lon },
                address: response.features[0].properties.full_address,
              })
            }}
          />
        </div>
        <Button
          variant="default"
          className="size-8 rounded-full z-10 absolute right-3 top-1/2 transform -translate-y-1/2"
          type="button"
          size="icon"
          onClick={() => {
            onChange(end, start)
          }}
        >
          <ArrowDownUpIcon />
        </Button>
      </div>
      <Button
        className="!rounded-t-none"
        variant="default"
        size="default"
        // icon={MagnifyingGlassIcon}
        onClick={onSearch}
      >
        Search
      </Button>
    </div>
  )
}
