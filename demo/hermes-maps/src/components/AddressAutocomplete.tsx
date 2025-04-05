import { useSearchBoxCore } from '@mapbox/search-js-react'
import type {
  SearchBoxRetrieveResponse,
  SearchBoxSuggestion,
} from '@mapbox/search-js-core'
import {
  Autocomplete,
  AutocompleteInput,
  AutocompleteItem,
  AutocompleteList,
} from './Autocomplete'
import { Input } from './Input'
import { MAPBOX_ACCESS_TOKEN } from '../constants'
import { useEffect, useState } from 'react'
import { isNil } from '../utils/isNil'

const sessionToken = crypto.randomUUID()

export function AddressAutocomplete({
  onRetrieve,
  value,
}: {
  onRetrieve: (value: SearchBoxRetrieveResponse) => void
  value: string
}) {
  const [input, setInput] = useState(value)

  useEffect(() => {
    setInput(value)
  }, [value])

  const [loading, setLoading] = useState(false)
  const [suggestions, setSuggestions] = useState<SearchBoxSuggestion[]>([])
  const searchBox = useSearchBoxCore({ accessToken: MAPBOX_ACCESS_TOKEN })

  const fetchSuggestions = async (query: string) => {
    setLoading(true)

    try {
      const response = await searchBox.suggest(query, {
        sessionToken,
        limit: 7,
      })
      const suggestions = response.suggestions
      setSuggestions(suggestions)
    } catch {
      setSuggestions([])
    } finally {
      setLoading(false)
    }
  }

  return (
    <Autocomplete
      onSelect={async (id) => {
        const suggestion = suggestions.find(
          (suggestion) => suggestion.mapbox_id === id
        )
        if (isNil(suggestion)) {
          return
        }

        setInput(suggestion.full_address)
        const response = await searchBox.retrieve(suggestion, { sessionToken })

        onRetrieve(response)
      }}
    >
      <AutocompleteInput
        asChild
        value={input}
        onValueChange={async (value: string) => {
          setInput(value)
          await fetchSuggestions(value)
        }}
      >
        <Input data-1p-ignore />
      </AutocompleteInput>

      {suggestions.length > 0 && (
        <AutocompleteList>
          {suggestions.map((suggestion, index) => {
            const label = suggestion.full_address
              ? suggestion.full_address
              : `${suggestion.name}, ${suggestion.place_formatted}`

            return (
              <AutocompleteItem
                key={suggestion.mapbox_id}
                index={index}
                value={suggestion.mapbox_id}
                label={label}
              >
                {label}
              </AutocompleteItem>
            )
          })}
        </AutocompleteList>
      )}
    </Autocomplete>
  )
}
