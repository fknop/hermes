import { getJob, useCreateJob, useListJobs } from '@/api/generated/hermes'
import { VehicleRoutingJob } from '@/api/generated/schemas'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardAction,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { DataTable } from '@/components/ui/data-table'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useDateTimeFormatter } from '@/hooks/useDateTimeFormatter'
import { DateTimeFormat } from '@/lib/DateTimeFormat'
import { useQueryClient } from '@tanstack/react-query'
import { ColumnDef } from '@tanstack/react-table'
import { EyeIcon, MoreHorizontal } from 'lucide-react'
import { useMemo } from 'react'
import { JsonFileUpload } from '../job/JsonFileUpload'
import { useNavigate } from 'react-router'

export default function JobsRoute() {
  const { data, isPending, queryKey } = useListJobs()
  const { mutateAsync: createJob, isPending: isCreating } = useCreateJob()
  const client = useQueryClient()
  const { formatDateTime } = useDateTimeFormatter()
  const navigate = useNavigate()

  const columns = useMemo(() => {
    const columns: ColumnDef<VehicleRoutingJob>[] = [
      {
        id: 'id',
        accessorKey: 'job_id',
        header: 'Job ID',
      },
      {
        id: 'status',
        accessorKey: 'status',
        header: 'Status',
        cell(info) {
          return info.getValue()
        },
      },
      {
        id: 'createdAt',
        accessorKey: 'created_at',
        header: 'Date',
        cell(info) {
          return formatDateTime(info.row.original.created_at, {
            format: DateTimeFormat.DATETIME_MED,
          })
        },
      },
      {
        id: 'actions',
        cell({ row }) {
          return (
            <DropdownMenu>
              <DropdownMenuTrigger
                render={
                  <Button variant="ghost" className="h-8 w-8 p-0">
                    <span className="sr-only">Open menu</span>
                    <MoreHorizontal className="h-4 w-4" />
                  </Button>
                }
              />
              <DropdownMenuContent align="end">
                <DropdownMenuGroup>
                  <DropdownMenuLabel>Actions</DropdownMenuLabel>
                  <DropdownMenuItem
                    onClick={() => {
                      navigate(`/jobs/${row.original.job_id}`)
                    }}
                  >
                    <EyeIcon />
                    Open
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          )
        },
      },
    ]

    return columns
  }, [data, formatDateTime])

  return (
    <div className="h-screen w-screen p-10 flex flex-row justify-center">
      <div className="max-w-5xl flex-1">
        <Card>
          <CardHeader>
            <CardTitle>Jobs</CardTitle>
            <CardAction>
              <JsonFileUpload
                loading={isCreating}
                onFileUpload={async (acceptedFile) => {
                  const content = await acceptedFile.text()
                  const input = JSON.parse(content)
                  await createJob({ data: input })
                  client.invalidateQueries({
                    queryKey,
                  })
                }}
              />
            </CardAction>
          </CardHeader>
          <CardContent>
            <DataTable columns={columns} data={data?.data.data ?? []} />
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
