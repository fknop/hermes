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
} from 'recharts'
import { VRP_COLORS } from '../vehicle-routing/colors'
import { useFetch } from '../../hooks/useFetch'
import { isNil } from '../../utils/isNil'
import { Button } from '../../components/Button'

type ProblemData = {
  locations: { x: number; y: number }[]
  services: { location_id: number }[]
  vehicles: { depot_location_id: number | null }[]
}

type SolutionData = {
  routes: {
    activities: { service_id: number }[]
    distance: number
    vehicle_id: number
  }[]
  distance: number
}

export type SolutionResponse = {
  status: 'Pending' | 'Running' | 'Completed'
  solution: SolutionData | null
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
        variant="primary"
        disabled={!dataset || isRunning}
        onClick={() => {
          startBenchmark(dataset!)
        }}
      >
        Start Benchmark
      </Button>

      <Button
        variant="primary"
        disabled={!isRunning}
        onClick={() => {
          stopBenchmark()
        }}
      >
        Stop Benchmark
      </Button>

      <div className="flex flex-row gap-10">
        {problem && <ProblemChart data={problem} />}
        <div className="flex flex-col gap-4">
          {solution?.solution && (
            <span>Total distance: {solution.solution.distance}</span>
          )}
          {solution?.solution && problem && (
            <SolutionChart solution={solution.solution} problem={problem} />
          )}
        </div>
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
            fill={VRP_COLORS[index % VRP_COLORS.length]}
            line
          />
        ))}
      </ScatterChart>
    </ResponsiveContainer>
  )
}
