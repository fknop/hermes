import { Button } from '@/components/ui/button'
import { JsonFileUpload } from '../JsonFileUpload'
import { useRoutingJobContext } from './RoutingJobContext'
import { isNil } from '@/utils/isNil'
import { DebugPanel } from './DebugPanel'
import { Spinner } from '@/components/ui/spinner'
import { disabled } from 'node_modules/@base-ui/react/esm/utils/reason-parts'
import { PlayIcon, StopCircleIcon } from 'lucide-react'
import { ButtonGroup } from '@/components/ui/button-group'

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
      <div>
        {response?.statistics && response?.weights && (
          <div>
            <DebugPanel
              statistics={response.statistics}
              weights={response.weights}
            />
          </div>
        )}
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
