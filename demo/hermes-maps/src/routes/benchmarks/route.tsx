import {
  ApiSolution,
  ApiSolutionActivity,
  VehicleRoutingJobInput,
} from '@/api/generated/schemas'
import { Button } from '@/components/ui/button'
import { Fragment, useCallback, useMemo, useState } from 'react'
import {
  ClientLoaderFunctionArgs,
  useLoaderData,
  useSearchParams,
} from 'react-router'
import {
  CartesianGrid,
  ResponsiveContainer,
  Scatter,
  ScatterChart,
  XAxis,
  YAxis,
} from 'recharts'
import { API_URL } from '../../constants'
import { useFetch } from '../../hooks/useFetch'
import { isNil } from '../../utils/isNil'
import { getRouteColor } from '../job/colors'
import { getSolution } from '../job/solution'
import { usePollRouting } from '../job/usePollRouting'
import { useStopRouting } from '../job/useStopRouting'
import { CVRPTW_DATASETS } from './datasets'
import {
  NativeSelect,
  NativeSelectOptGroup,
  NativeSelectOption,
} from '@/components/ui/native-select'

const usePostBenchmark = () => {
  const [post] = useFetch<{ job_id: string }>('/vrp/benchmark')

  const [jobId, setJobId] = useState<string | null>(null)

  const stopRouting = useStopRouting()
  const { response } = usePollRouting({ jobId, geojson: false })

  return {
    startBenchmark: useCallback(
      async (dataset: string) => {
        const response = await post({
          body: {
            category: dataset.split('/')[0],
            name: dataset.split('/')[1],
          },
          method: 'POST',
        })

        setJobId(response.job_id)
      },
      [post]
    ),
    stopBenchmark: useCallback(async () => {
      if (jobId) {
        await stopRouting({ jobId: jobId! })
        setJobId(null)
      }
    }, [jobId]),
    response,
  }
}

export async function clientLoader({
  request,
  params,
}: ClientLoaderFunctionArgs) {
  const url = new URL(request.url)
  const dataset = url.searchParams.get('dataset')

  if (dataset) {
    const serverUrl = new URL(`${API_URL}/vrp/benchmark/${dataset}`)
    const response = await fetch(serverUrl)
    return (await response.json()) as VehicleRoutingJobInput
  }

  return null
}

export default function SolomonRoute() {
  const [searchParams, setSearchParams] = useSearchParams()
  const dataset = searchParams.get('dataset')

  const problem = useLoaderData<typeof clientLoader>()

  const { startBenchmark, stopBenchmark, response } = usePostBenchmark()
  const isRunning = response?.status === 'Running'
  const solution = getSolution(response)

  return (
    <div className="p-4 flex flex-col gap-10">
      <div className="flex flex-row items-center gap-4">
        <NativeSelect
          disabled={isRunning}
          value={dataset ?? ''}
          onChange={(event) => {
            setSearchParams({ dataset: event.target.value })
          }}
        >
          <NativeSelectOption value="">Select a dataset</NativeSelectOption>
          {CVRPTW_DATASETS.map(({ category, datasets }) => {
            return (
              <Fragment key={category}>
                {datasets.map(({ category, problems }) => {
                  return (
                    <NativeSelectOptGroup label={category} key={category}>
                      {problems.map((problem) => (
                        <NativeSelectOption
                          key={problem}
                          value={`${category}/${problem}`}
                        >
                          {problem}
                        </NativeSelectOption>
                      ))}
                    </NativeSelectOptGroup>
                  )
                })}
              </Fragment>
            )
          })}
        </NativeSelect>

        <Button
          variant="default"
          disabled={!dataset || isRunning}
          onClick={() => {
            startBenchmark(dataset!)
          }}
        >
          Start Benchmark
        </Button>

        <Button
          variant="default"
          disabled={!isRunning}
          onClick={() => {
            stopBenchmark()
          }}
        >
          Stop Benchmark
        </Button>
      </div>

      <div className="flex flex-col gap-10">
        <div className="flex flex-row gap-10">
          {problem && <ProblemChart data={problem} />}
          <div className="flex flex-col gap-4">
            {solution && (
              <div className="flex flex-row gap-2">
                <span>Total distance: {solution.distance}</span>
                <span>Vehicles: {solution.routes.length}</span>
                <span>
                  Score: hard/{solution.score.hard_score} soft/
                  {solution.score.soft_score}
                </span>
              </div>
            )}
            {solution && problem && (
              <SolutionChart solution={solution} problem={problem} />
            )}
          </div>
        </div>

        {/*<ScoreEvolutionChart statistics={getStatistics(response)} />*/}
      </div>
    </div>
  )
}

function ProblemChart({ data }: { data: VehicleRoutingJobInput }) {
  const points = data.locations.map((location) => {
    return {
      x: location.coordinates[0],
      y: location.coordinates[1],
    }
  })

  return (
    <ResponsiveContainer width={600} height={600}>
      <ScatterChart>
        <CartesianGrid />
        <XAxis type="number" dataKey="x" />
        <YAxis type="number" dataKey="y" />
        <Scatter name="Services" data={points} fill="blue" />
      </ScatterChart>
    </ResponsiveContainer>
  )
}

function SolutionChart({
  solution,
  problem,
}: {
  solution: ApiSolution
  problem: VehicleRoutingJobInput
}) {
  const routesPoints = useMemo(
    () =>
      solution.routes.map((route) => {
        const locations = []

        const vehicleId = problem.vehicles.findIndex(
          (vehicle) => vehicle.id === route.vehicle_id
        )
        const vehicle = problem.vehicles[vehicleId]

        if (!isNil(vehicle.depot_location_id)) {
          const vehicleLocation = problem.locations[vehicle.depot_location_id]

          locations.push({
            x: vehicleLocation.coordinates[0],
            y: vehicleLocation.coordinates[1],
          })
        }

        locations.push(
          ...route.activities
            .filter(
              (
                activity
              ): activity is Extract<
                ApiSolutionActivity,
                { type: 'Service' }
              > => activity.type === 'Service'
            )
            .map((activity) => {
              const serviceId = problem.services.findIndex(
                (service) => service.id === activity.id
              )
              const service = problem.services[serviceId]
              const location = problem.locations[service.location_id]
              return {
                x: location.coordinates[0],
                y: location.coordinates[1],
              }
            })
        )

        if (!isNil(vehicle.depot_location_id)) {
          const vehicleLocation = problem.locations[vehicle.depot_location_id]

          locations.push({
            x: vehicleLocation.coordinates[0],
            y: vehicleLocation.coordinates[1],
          })
        }

        return locations
      }),
    [solution, problem]
  )

  return (
    <ResponsiveContainer width={600} height={600}>
      <ScatterChart>
        <CartesianGrid />
        <XAxis type="number" dataKey="x" />
        <YAxis type="number" dataKey="y" />
        {routesPoints.map((points, index) => (
          <Scatter
            key={index}
            name={`Route ${index + 1}`}
            data={points}
            fill={getRouteColor(index)}
            line
          />
        ))}
      </ScatterChart>
    </ResponsiveContainer>
  )
}

// function ScoreEvolutionChart({
//   statistics,
// }: {
//   statistics: SolutionResponse['statistics']
// }) {
//   const data = useMemo(() => {
//     return statistics?.global_statistics.score_evolution.map(
//       ({ score, timestamp }) => {
//         return {
//           x: new Date(timestamp).getTime() / 1000,
//           soft: score.soft_score,
//           hard: score.hard_score,
//         }
//       }
//     )
//   }, [statistics])

//   const formatDuration = useDurationFormatter()

//   const formatXAxis = (value: number) => {
//     const first = data?.[0].x

//     if (isNil(first)) {
//       return ''
//     }

//     const duration = value - first
//     return `${Math.round(duration * 100.0) / 100.0}s`
//   }

//   if (isNil(data)) {
//     return null
//   }

//   return (
//     <ResponsiveContainer width={600} height={300}>
//       <LineChart data={data}>
//         <CartesianGrid />
//         <XAxis
//           type="number"
//           dataKey="x"
//           scale="time"
//           domain={['dataMin', 'dataMax']}
//           tickFormatter={formatXAxis}
//         />
//         <YAxis type="number" />
//         <Line type="monotone" dataKey="soft" stroke="blue" />
//         <Line type="monotone" dataKey="hard" stroke="red" />
//       </LineChart>
//     </ResponsiveContainer>
//   )
// }
