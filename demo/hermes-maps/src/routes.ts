import { type RouteConfig, route } from '@react-router/dev/routes'

export default [
  route('/', './routes/home.tsx'),
  route('/vehicle-routing', './routes/vehicle-routing/route.tsx'),
] satisfies RouteConfig
