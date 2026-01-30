import { Button } from '@/components/ui/button'
import { JsonFileUpload } from '../JsonFileUpload'
import { useRoutingJobContext } from './RoutingJobContext'
import { isNil } from '@/utils/isNil'
import { DebugPanel } from './DebugPanel'
import {
  EyeIcon,
  MapIcon,
  MapMinusIcon,
  PlayIcon,
  StopCircleIcon,
} from 'lucide-react'
import { ButtonGroup } from '@/components/ui/button-group'
import {
  Menubar,
  MenubarCheckboxItem,
  MenubarContent,
  MenubarMenu,
  MenubarSeparator,
  MenubarTrigger,
} from '@/components/ui/menubar'
import { useMemo } from 'react'
import { VRP_COLORS } from '../colors'
import { Separator } from '@/components/ui/separator'

function RoutesVisibilityMenu() {
  const { input, response } = useRoutingJobContext()
  const routes = useMemo(() => {
    return response?.solution?.routes ?? []
  }, [response])

  return (
    <MenubarMenu disabled={isNil(input) || isNil(response)}>
      <MenubarTrigger className="gap-1">
        <MapMinusIcon className="size-4" data-icon="inline-start" />
        <span>Routes</span>
      </MenubarTrigger>
      <MenubarContent>
        {routes.map((route, index) => {
          return (
            <MenubarCheckboxItem key={index}>
              <div
                className="size-3 rounded-full"
                style={{
                  backgroundColor: VRP_COLORS[index % VRP_COLORS.length],
                }}
              />
              <span>Route {index + 1}</span>
            </MenubarCheckboxItem>
          )
        })}
      </MenubarContent>
    </MenubarMenu>
  )
}

export function VehicleRoutingMenu() {
  const {
    isRunning,
    onInputChange,
    input,
    startRouting,
    stopRouting,
    response,
  } = useRoutingJobContext()

  return (
    <div className="p-1.5 bg-background border-b border-b-sidebar-border flex items-center justify-between">
      <div className="flex flex-row items-center">
        <Menubar>
          <RoutesVisibilityMenu />
          {response?.statistics && response?.weights && (
            <>
              <Separator orientation="vertical" className="mx-1" />
              <MenubarMenu>
                <MenubarTrigger
                  className="gap-1"
                  render={
                    <DebugPanel
                      statistics={response.statistics}
                      weights={response.weights}
                    />
                  }
                >
                  <span>Debug</span>
                </MenubarTrigger>
              </MenubarMenu>
            </>
          )}
        </Menubar>
      </div>
      <div className="flex flex-row items-center gap-2">
        <JsonFileUpload
          onFileUpload={async (file) => {
            const data = await file.text()
            onInputChange(JSON.parse(data))
          }}
          disabled={isRunning}
        />

        <ButtonGroup>
          <Button
            variant={isRunning ? 'secondary' : 'default'}
            disabled={isRunning || isNil(input)}
            onClick={() => {
              startRouting()
            }}
            loading={isRunning}
            icon={PlayIcon}
          >
            {isRunning ? 'Running...' : 'Start'}
          </Button>
          {isRunning && (
            <Button
              icon={StopCircleIcon}
              variant="destructive"
              onClick={() => {
                stopRouting()
              }}
            >
              Stop
            </Button>
          )}
        </ButtonGroup>
      </div>
    </div>
  )
}
