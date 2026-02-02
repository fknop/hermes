import { Dropzone } from '../../components/Dropzone'
import { isNil } from '../../utils/isNil'

const FILE_TYPES = { 'application/json': [] as string[] } as const

export type JsonFileUploadProps = {
  onFileUpload: (file: File) => void
  disabled?: boolean
}

export function JsonFileUpload({
  onFileUpload,
  disabled,
}: JsonFileUploadProps) {
  return (
    <Dropzone
      disabled={disabled}
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
