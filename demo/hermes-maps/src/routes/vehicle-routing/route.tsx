import { Source } from 'react-map-gl/mapbox'
import { Button } from '../../components/Button.tsx'
import { MapSidePanel } from '../../components/MapSidePanel.tsx'
import { Map } from '../../Map.tsx'
import { MultiPointLayer } from '../../MultiPointLayer.tsx'
import { transformSolutionToGeoJson } from './transformSolutionToGeoJson.tsx'
import { usePollRouting } from './usePollRouting.ts'
import { POST_BODY, usePostRouting } from './usePostRouting.ts'

const colors = [
  '#e6194b',
  '#3cb44b',
  '#ffe119',
  '#4363d8',
  '#f58231',
  '#911eb4',
  '#46f0f0',
  '#f032e6',
  '#bcf60c',
  '#fabebe',
  '#008080',
  '#e6beff',
  '#9a6324',
  '#fffac8',
  '#800000',
  '#aaffc3',
  '#808000',
  '#ffd8b1',
  '#000075',
  '#808080',
  '#ffffff',
  '#000000',
]

export default function VehicleRoutingScreen() {
  const [postRouting, { loading, data }] = usePostRouting()
  const { solution } = usePollRouting({ jobId: data?.job_id ?? null })

  const polling = solution?.status === 'Running'

  const geojson = solution
    ? transformSolutionToGeoJson(POST_BODY, solution)
    : null

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
