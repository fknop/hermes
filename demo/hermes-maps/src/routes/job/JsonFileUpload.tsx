import { ButtonProps } from '@/components/ui/button'
import { Dropzone } from '../../components/Dropzone'
import { isNil } from '../../utils/isNil'

const FILE_TYPES = { 'application/json': [] as string[] } as const

export type JsonFileUploadProps = {
  onFileUpload: (file: File) => void
  disabled?: boolean
  loading?: boolean
  variant?: ButtonProps['variant']
}

export function JsonFileUpload({
  onFileUpload,
  disabled,
  loading,
  variant,
}: JsonFileUploadProps) {
  return (
    <Dropzone
      loading={loading}
      disabled={disabled}
      variant={variant}
      accept={FILE_TYPES}
      description="Create job"
      multiple={false}
      onDropAccepted={(files) => {
        const file = files[0]
        if (!isNil(file)) {
          onFileUpload(file)
        }
      }}
    />
  )
}
