import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { OperatorWeights, SolutionStatistics } from '../solution'
import { StatisticsPanel } from './StatisticsPanel'
import { WeightsPanel } from './WeightsPanel'
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet'
import { Button } from '@/components/ui/button'
import { ChartPieIcon } from 'lucide-react'

export function DebugPanel({
  statistics,
  weights,
}: {
  statistics: SolutionStatistics
  weights: OperatorWeights
}) {
  return (
    <Sheet modal={false} disablePointerDismissal>
      <SheetTrigger
        render={
          <Button variant="outline" size="icon">
            <ChartPieIcon />
          </Button>
        }
      />
      <SheetContent side="bottom" className="p-4" showOverlay={false}>
        <Tabs>
          <TabsList>
            <TabsTrigger value="statistics">Statistics</TabsTrigger>
            <TabsTrigger value="weights">Weights</TabsTrigger>
          </TabsList>
          <TabsContent value="statistics">
            <StatisticsPanel statistics={statistics} />
          </TabsContent>
          <TabsContent value="weights">
            <WeightsPanel weights={weights} />
          </TabsContent>
        </Tabs>
      </SheetContent>
    </Sheet>
  )
}
