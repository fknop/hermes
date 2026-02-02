import clsx from 'clsx'
import { FileUpIcon } from 'lucide-react'
import { ReactElement, ReactNode } from 'react'
import { DropzoneOptions, useDropzone } from 'react-dropzone'
import { Button } from './ui/button'

export type DropzoneRenderProps = {
  isDragActive: boolean
}

export type DropzoneProps = DropzoneOptions & {
  description: string | ReactNode | ReactElement
  name?: string
  'data-testid'?: string
  className?: string
}

// If you experience the file picker opening twice, make sure the dropzone is not inside a <label /> tag
export const Dropzone = ({
  description,
  disabled,
  name,
  'data-testid': dataTestId,
  className,
  ...options
}: DropzoneProps) => {
  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    disabled,
    ...options,
  })

  return (
    <div className={clsx('flex flex-col gap-4', className)}>
      <div data-testid={dataTestId} {...getRootProps()}>
        <input {...getInputProps()} name={name} />

        <Button variant="outline" icon={FileUpIcon} disabled={disabled}>
          {description}
        </Button>
      </div>
    </div>
  )
}
