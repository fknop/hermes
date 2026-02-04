import { AggregatedStatistics, OperatorWeights } from '@/api/generated/schemas'
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ChartPieIcon } from 'lucide-react'
import { StatisticsPanel } from './StatisticsPanel'
import { WeightsPanel } from './WeightsPanel'

export function DebugPanel({
  statistics,
  weights,
}: {
  statistics: AggregatedStatistics
  weights: OperatorWeights
}) {
  return (
    <Sheet modal={false} disablePointerDismissal>
      <SheetTrigger
        render={
          <Button variant="ghost" icon={ChartPieIcon}>
            Debug
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
