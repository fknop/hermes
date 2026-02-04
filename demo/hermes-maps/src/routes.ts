import { type RouteConfig, route } from '@react-router/dev/routes'
import 'react-router'

export default [
  route('/', './routes/home.tsx'),
  route('/jobs', './routes/jobs/route.tsx'),
  // route('/jobs/:jobId', './routes/job/route.tsx'),
  route('/vehicle-routing', './routes/vehicle-routing/route.tsx'),
  route('/benchmarks', './routes/benchmarks/route.tsx'),
] satisfies RouteConfig

declare module 'react-router' {
  interface AppLoadContext {}
}
