import {
  ArrowsUpDownIcon,
  ArrowTurnDownRightIcon,
} from '@heroicons/react/16/solid'
import { Address } from '../types/Address'
import { AddressAutocomplete } from './AddressAutocomplete'
import { HTMLAttributes, Ref } from 'react'
import clsx from 'clsx'
import { Button } from './Button'
import { MagnifyingGlassIcon } from '@heroicons/react/24/solid'

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
        'bg-white border not-first-of-type:-my-px border-slate-900/15 ring-0',
        'w-full first-of-type:rounded-t-lg last-of-type:border-b-0 px-3 py-1.5 text-base text-gray-900',
        'focus:bg-slate-50',
        'placeholder:text-gray-400 focus:outline-none',
        'focus:border-slate-600 focus-visible:outline-slate-600',
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
        <div className="flex flex-col divide-y divide-gray-300 flex-1">
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
          icon={ArrowsUpDownIcon}
          variant="primary"
          className="size-8 rounded-full z-10 absolute right-3 top-1/2 transform -translate-y-1/2"
          type="button"
          onClick={() => {
            onChange(end, start)
          }}
        />
      </div>
      <Button
        className="!rounded-t-none"
        variant="primary"
        size="normal"
        icon={MagnifyingGlassIcon}
        onClick={onSearch}
      >
        Search
      </Button>
    </div>
  )
}
