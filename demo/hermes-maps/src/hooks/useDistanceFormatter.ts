import { useCallback } from 'react'

export function useDistanceFormatter() {
  return useCallback((meters: number) => {
    if (meters < 1000) {
      const formatter = new Intl.NumberFormat('en-GB', {
        style: 'unit',
        unit: 'meter',
        unitDisplay: 'narrow',
        maximumFractionDigits: 0,
      })

      return formatter.format(meters)
    }

    const formatter = new Intl.NumberFormat('en-GB', {
      style: 'unit',
      unit: 'kilometer',
      unitDisplay: 'narrow',
      maximumFractionDigits: 2,
    })

    return formatter.format(meters / 1000)
  }, [])
}
