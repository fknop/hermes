import { Links, Meta, Outlet, Scripts, ScrollRestoration } from 'react-router'
import './index.css'
import { ThemeProvider } from './components/ui/theme-provider'
import { QueryClientProvider } from '@tanstack/react-query'
import { queryClient } from './api/client'

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <link
          href="https://api.tiles.mapbox.com/mapbox-gl-js/v3.10.0/mapbox-gl.css"
          rel="stylesheet"
        />
        <meta charSet="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Hermes</title>
        <Meta />
        <Links />
      </head>
      <body>
        <QueryClientProvider client={queryClient}>
          <ThemeProvider>{children}</ThemeProvider>
        </QueryClientProvider>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  )
}

export default function Root() {
  return <Outlet />
}
