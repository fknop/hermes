import { Dropzone } from '../../components/Dropzone'
import { isNil } from '../../utils/isNil'

const FILE_TYPES = { 'application/json': [] as string[] } as const

export type JsonFileUploadProps = {
  onFileUpload: (file: File) => void
}

export function JsonFileUpload({ onFileUpload }: JsonFileUploadProps) {
  return (
    <Dropzone
      accept={FILE_TYPES}
      description="Upload a JSON file"
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
