import { Source } from 'react-map-gl/mapbox'
import { Button } from '../../components/Button.tsx'
import { MapSidePanel } from '../../components/MapSidePanel.tsx'
import { Map } from '../../Map.tsx'
import { MultiPointLayer } from '../../MultiPointLayer.tsx'
import { transformSolutionToGeoJson } from './transformSolutionToGeoJson.tsx'
import { usePollRouting } from './usePollRouting.ts'
import { POST_BODY, usePostRouting } from './usePostRouting.ts'
import { PolylineLayer } from '../../PolylineLayer.tsx'

const colors = [
  '#2C3E50', // Dark Blue (Deep Navy)
  '#1A521A', // Forest Green (Darker)
  '#8B0000', // Dark Red (Maroon)
  '#4B0082', // Indigo (Deep Violet)
  '#B8860B', // Dark Goldenrod (Mustard-like)
  '#006400', // Dark Green (Bottle Green)
  '#D2691E', // Chocolate (Rich Brown)
  '#483D8B', // Dark Slate Blue (Muted Purple-Blue)
  '#708090', // Slate Gray (Medium Dark Gray)
  '#008B8B', // Dark Cyan (Deep Teal)
  '#696969', // Dim Gray (Neutral Gray)
  '#5F9EA0', // Cadet Blue (Dusty Blue)
  '#8B4513', // Saddle Brown (Earthy Brown)
  '#556B2F', // Dark Olive Green (Muted Green-Brown)
  '#CD5C5C', // Indian Red (Softer Red)
  '#4682B4', // Steel Blue (Medium Blue)
  '#7B68EE', // Medium Slate Blue (Slightly more vibrant purple)
  '#2E8B57', // Sea Green (Deep Green)
  '#8A2BE2', // Blue Violet (Purple with Blue undertone)
  '#FF8C00', // Dark Orange (Pumpkin-like, good for a distinct pop)
]

export default function VehicleRoutingScreen() {
  const [postRouting, { loading, data }] = usePostRouting()
  const { solution } = usePollRouting({ jobId: data?.job_id ?? null })

  const polling = solution?.status === 'Running'

  const geojson = solution
    ? transformSolutionToGeoJson(POST_BODY, solution)
    : null

  const totalTime = solution?.solution.duration

  console.log(totalTime)

  return (
    <div className="h-screen w-screen">
      <Map>
        <MapSidePanel>
          <Button
            variant="primary"
            disabled={loading || polling}
            onClick={() => {
              postRouting()
            }}
          >
            Start
          </Button>
        </MapSidePanel>

        {solution && (
          <>
            {solution.solution.routes.map((route, index) => {
              return (
                <Source
                  key={index}
                  type="geojson"
                  data={route.polyline}
                  id={`polyline-${index}`}
                >
                  <PolylineLayer
                    id={`polyline-${index}`}
                    color={colors[index % colors.length]}
                    sourceId={`polyline-${index}`}
                    lineWidth={3}
                  />
                </Source>
              )
            })}
          </>
        )}

        {geojson && (
          <>
            <Source type="geojson" data={geojson.points} id="geojson">
              {geojson.points.features.map((_, index) => {
                return (
                  <MultiPointLayer
                    color={colors[index % colors.length]}
                    featureId={index.toString()}
                    sourceId="geojson"
                    id={`activities_${index}`}
                    key={index}
                    radiusMultiplier={10}
                  />
                )
              })}
            </Source>
          </>
        )}
      </Map>
    </div>
  )
}
