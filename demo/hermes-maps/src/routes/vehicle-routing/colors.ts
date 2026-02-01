const ROUTE_COLORS = [
  '#f59e0b',
  '#ea580c',
  '#4d7c0f',
  '#0f766e',
  '#1e3a8a',
  '#10b981',
  '#93c5fd',
  '#7e22ce',
  '#a21caf',
  '#be185d',
  '#b91c1c',
  '#bef264',
  '#a78bfa',
  '#fcd34d',
  '#fca5a5',
  '#365314',
  '#7f1d1d',
  '#134e4a',
  '#0c4a6e',
  '#4c1d95',
  '#881337',
]
export function getRouteColor(index: number) {
  return ROUTE_COLORS[index % ROUTE_COLORS.length]
}
