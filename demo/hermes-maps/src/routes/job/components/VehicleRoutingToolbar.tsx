import { Button } from '@/components/ui/button'
import { ButtonGroup } from '@/components/ui/button-group'
import {
  Toolbar,
  ToolbarButton,
  ToolbarMenu,
  ToolbarMenuCheckboxItem,
  ToolbarMenuContent,
  ToolbarMenuTrigger,
  ToolbarSeparator,
} from '@/components/ui/toolbar'
import { isNil } from '@/utils/isNil'
import {
  ChartGanttIcon,
  MapMinusIcon,
  PlayIcon,
  StopCircleIcon,
} from 'lucide-react'
import { JsonFileUpload } from '../JsonFileUpload'
import { DebugPanel } from './DebugPanel'
import { useRoutingJobContext } from './RoutingJobContext'
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet'
import { RoutingSchedule } from './RoutingSchedule'
import { getOperatorWeights, getSolution, getStatistics } from '../solution'
import { g } from 'node_modules/@react-router/dev/dist/routes-CZR-bKRt'

function RoutesVisibilityMenu() {
  const { input, response } = useRoutingJobContext()

  return (
    <ToolbarMenu disabled={isNil(input) || isNil(response)}>
      <ToolbarMenuTrigger
        className="gap-1"
        render={
          <Button variant="ghost" icon={MapMinusIcon}>
            Map
          </Button>
        }
      />

      <ToolbarMenuContent className="w-full">
        <ToolbarMenuCheckboxItem checked>
          Unassigned locations
        </ToolbarMenuCheckboxItem>
      </ToolbarMenuContent>
    </ToolbarMenu>
  )
}

export function VehicleRoutingToolbar() {
  const {
    isRunning,
    // onInputChange,
    input,
    isStarting,
    startRouting,
    stopRouting,
    response,
  } = useRoutingJobContext()

  const statistics = getStatistics(response)
  const weights = getOperatorWeights(response)

  return (
    <div className="p-1.5 bg-sidebar border-b border-b-sidebar-border flex items-center justify-between">
      <div className="flex flex-row items-center">
        <Toolbar>
          <RoutesVisibilityMenu />
          <ToolbarSeparator />

          <Sheet modal={false} disablePointerDismissal>
            <SheetTrigger
              render={
                <ToolbarButton
                  render={
                    <Button
                      variant="ghost"
                      size="icon"
                      disabled={isNil(getSolution(response))}
                    >
                      <ChartGanttIcon />
                    </Button>
                  }
                />
              }
            />
            <SheetContent
              side="bottom"
              className="p-4"
              showOverlay={false}
              showCloseButton={false}
            >
              <RoutingSchedule />
            </SheetContent>
          </Sheet>

          {statistics && weights && (
            <>
              <ToolbarSeparator />
              <ToolbarButton
                render={
                  <DebugPanel statistics={statistics} weights={weights} />
                }
              />
            </>
          )}
        </Toolbar>
      </div>
      <div className="flex flex-row items-center gap-2">
        {/*<JsonFileUpload
          variant="outline"
          onFileUpload={async (file) => {
            const data = await file.text()
            onInputChange(JSON.parse(data))
          }}
          disabled={isRunning}
        />*/}

        <ButtonGroup>
          <Button
            variant={isRunning ? 'secondary' : 'default'}
            disabled={isRunning || isNil(input) || isStarting}
            onClick={() => {
              startRouting()
            }}
            loading={isRunning || isStarting}
            icon={PlayIcon}
          >
            {isRunning ? 'Running...' : isStarting ? 'Starting...' : 'Start'}
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
