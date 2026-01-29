import { useMemo } from 'react'
import { LngLatBoundsLike } from 'react-map-gl/mapbox'

export function useMapboxBounds(
  coordinates: [number, number][]
): LngLatBoundsLike | undefined {
  return useMemo(() => {
    if (coordinates.length === 0) {
      return undefined
    }

    let minX = coordinates[0][0]
    let minY = coordinates[0][1]
    let maxX = coordinates[0][0]
    let maxY = coordinates[0][1]

    for (const coord of coordinates) {
      if (coord[0] < minX) minX = coord[0]
      if (coord[1] < minY) minY = coord[1]
      if (coord[0] > maxX) maxX = coord[0]
      if (coord[1] > maxY) maxY = coord[1]
    }
    return [
      [minX, minY],
      [maxX, maxY],
    ] as [[number, number], [number, number]]
  }, [coordinates])
}
