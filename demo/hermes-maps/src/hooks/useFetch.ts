import { useCallback, useEffect, useRef, useState } from 'react'
import { API_URL } from '../constants.ts'

type FetchFunction<
  T,
  SP extends Record<string, string | number | boolean> = {},
  Body extends unknown = unknown,
> = (options?: {
  query?: SP
  body?: Body
  method?: 'GET' | 'POST'
}) => Promise<T>

export function useFetch<
  R,
  SP extends Record<string, string | number | boolean> = {},
  Body extends unknown = unknown,
>(
  path: string
): [
  fetch: FetchFunction<R, SP, Body>,
  { loading: boolean; data: R | undefined },
] {
  const [loading, setLoading] = useState(false)
  const [data, setData] = useState<R | undefined>(undefined)
  const signalRef = useRef(new AbortController())

  // useEffect(() => {
  //   return () => {
  //     signalRef.current.abort()
  //   }
  // }, [])

  return [
    useCallback(async (options) => {
      setLoading(true)
      try {
        const url = new URL(`${API_URL}${path}`)

        if (options?.query) {
          for (const [key, value] of Object.entries(options.query)) {
            url.searchParams.set(key, value.toString())
          }
        }

        const response = await fetch(url, {
          method: options?.method ?? 'GET',
          body: options?.body
            ? typeof options.body === 'string'
              ? options.body
              : JSON.stringify(options.body)
            : undefined,
          headers: {
            'Content-Type': 'application/json',
          },
          signal: signalRef.current.signal,
        })

        const json = (await response.json()) as R
        setData(json)
        return json
      } finally {
        setLoading(false)
      }
    }, []),
    { loading, data },
  ]
}
