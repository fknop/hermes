import { Fragment, useCallback, useEffect, useMemo, useState } from 'react'
import { CVRPTW_DATASETS } from './datasets'
import {
  ClientLoaderFunctionArgs,
  useLoaderData,
  useSearchParams,
} from 'react-router'
import { API_URL } from '../../constants'
import {
  ResponsiveContainer,
  CartesianGrid,
  ScatterChart,
  Scatter,
  XAxis,
  YAxis,
  Line,
  LineChart,
} from 'recharts'
import { useFetch } from '../../hooks/useFetch'
import { isNil } from '../../utils/isNil'
import { useDurationFormatter } from '../../hooks/useDurationFormatter'
import { Button } from '@/components/ui/button'
import { getRouteColor } from '../job/colors'

type ProblemData = {
  locations: { x: number; y: number }[]
  services: { location_id: number }[]
  vehicles: { depot_location_id: number | null }[]
}
type Score = { hard_score: number; soft_score: number }

type SolutionData = {
  routes: {
    activities: { service_id: number }[]
    distance: number
    vehicle_id: number
  }[]
  distance: number
  score: Score
}

export type SolutionResponse = {
  status: 'Pending' | 'Running' | 'Completed'
  solution: SolutionData | null
  statistics: {
    global_statistics: {
      score_evolution: {
        timestamp: string
        thread: number
        score: Score
        score_analysis: {
          scores: { [key: string]: Score }
        }
      }[]
    }
    thread_statistics: {
      ruin_strategies: { [key: string]: number }
      recreate_strategies: { [key: string]: number }
    }[]
  } | null
}

const usePostBenchmark = () => {
  const [post] = useFetch<{ job_id: string }>('/vrp/benchmark')

  const [solution, setSolution] = useState<SolutionResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const isCompleted = solution?.status === 'Completed'

  const [jobId, setJobId] = useState<string | null>(null)
  const isRunning = solution?.status === 'Running' && !isNil(jobId)

  useEffect(() => {
    if (isCompleted || isNil(jobId)) {
      return
    }

    async function run() {
      try {
        const response = await fetch(`${API_URL}/vrp/benchmark/poll/${jobId}`)
        if (response.status >= 400) {
          setError(`Failed ${response.status}`)
          return
        }
        const data: SolutionResponse = await response.json()
        setSolution(data)
      } catch (error) {
        console.error('Error fetching routing solution:', error)
      }
    }

    const interval = setInterval(run, 500) // Poll every 5 seconds

    void run()

    return () => {
      clearInterval(interval)
    }
  }, [isCompleted, jobId])

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
      await fetch(`${API_URL}/vrp/benchmark/stop/${jobId}`, {
        method: 'POST',
      })
      setJobId(null)
    }, [jobId]),
    solution,
    isRunning,
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
    return (await response.json()) as ProblemData
  }

  return null
}

export default function SolomonRoute() {
  const [searchParams, setSearchParams] = useSearchParams()
  const dataset = searchParams.get('dataset')

  const problem = useLoaderData<typeof clientLoader>()

  const { startBenchmark, stopBenchmark, solution, isRunning } =
    usePostBenchmark()

  return (
    <div>
      <select
        disabled={isRunning}
        value={dataset ?? ''}
        onChange={(event) => {
          setSearchParams({ dataset: event.target.value })
        }}
      >
        <option value="">Select a dataset</option>
        {CVRPTW_DATASETS.map(({ category, datasets }) => {
          return (
            <Fragment key={category}>
              {datasets.map(({ category, problems }) => {
                return (
                  <optgroup label={category} key={category}>
                    {problems.map((problem) => (
                      <option key={problem} value={`${category}/${problem}`}>
                        {problem}
                      </option>
                    ))}
                  </optgroup>
                )
              })}
            </Fragment>
          )
        })}
      </select>

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

      <div className="flex flex-col gap-10">
        <div className="flex flex-row gap-10">
          {problem && <ProblemChart data={problem} />}
          <div className="flex flex-col gap-4">
            {solution?.solution && (
              <div className="flex flex-row gap-2">
                <span>Total distance: {solution.solution.distance}</span>
                <span>Vehicles: {solution.solution.routes.length}</span>
                <span>
                  Score: hard/{solution.solution.score.hard_score} soft/
                  {solution.solution.score.soft_score}
                </span>
              </div>
            )}
            {solution?.solution && problem && (
              <SolutionChart solution={solution.solution} problem={problem} />
            )}
          </div>
        </div>

        <ScoreEvolutionChart statistics={solution?.statistics ?? null} />
      </div>
    </div>
  )
}

function ProblemChart({ data }: { data: ProblemData }) {
  const points = data.locations.map((location) => {
    return {
      x: location.x,
      y: location.y,
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
  solution: SolutionData
  problem: ProblemData
}) {
  const routesPoints = useMemo(
    () =>
      solution.routes.map((route) => {
        const locations = []

        const vehicleId = route.vehicle_id
        const vehicle = problem.vehicles[vehicleId]

        if (!isNil(vehicle.depot_location_id)) {
          const vehicleLocation = problem.locations[vehicle.depot_location_id]

          locations.push({
            x: vehicleLocation.x,
            y: vehicleLocation.y,
          })
        }

        locations.push(
          ...route.activities.map((activity) => {
            const service = problem.services[activity.service_id]
            const location = problem.locations[service.location_id]
            return {
              x: location.x,
              y: location.y,
            }
          })
        )

        if (!isNil(vehicle.depot_location_id)) {
          const vehicleLocation = problem.locations[vehicle.depot_location_id]

          locations.push({
            x: vehicleLocation.x,
            y: vehicleLocation.y,
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

function ScoreEvolutionChart({
  statistics,
}: {
  statistics: SolutionResponse['statistics']
}) {
  const data = useMemo(() => {
    return statistics?.global_statistics.score_evolution.map(
      ({ score, timestamp }) => {
        return {
          x: new Date(timestamp).getTime() / 1000,
          soft: score.soft_score,
          hard: score.hard_score,
        }
      }
    )
  }, [statistics])

  console.log(data)

  const formatDuration = useDurationFormatter()

  const formatXAxis = (value: number) => {
    const first = data?.[0].x

    if (isNil(first)) {
      return ''
    }

    const duration = value - first
    return `${Math.round(duration * 100.0) / 100.0}s`
  }

  if (isNil(data)) {
    return null
  }

  return (
    <ResponsiveContainer width={600} height={300}>
      <LineChart data={data}>
        <CartesianGrid />
        <XAxis
          type="number"
          dataKey="x"
          scale="time"
          domain={['dataMin', 'dataMax']}
          tickFormatter={formatXAxis}
        />
        <YAxis type="number" />
        <Line type="monotone" dataKey="soft" stroke="blue" />
        <Line type="monotone" dataKey="hard" stroke="red" />
      </LineChart>
    </ResponsiveContainer>
  )
}
