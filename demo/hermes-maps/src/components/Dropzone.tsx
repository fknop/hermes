import clsx from 'clsx'
import { FilePlusIcon } from 'lucide-react'
import { ReactElement, ReactNode } from 'react'
import { DropzoneOptions, useDropzone } from 'react-dropzone'
import { Button, ButtonProps } from './ui/button'

export type DropzoneRenderProps = {
  isDragActive: boolean
}

export type DropzoneProps = DropzoneOptions & {
  description: string | ReactNode | ReactElement
  name?: string
  'data-testid'?: string
  className?: string
  loading?: boolean
  variant?: ButtonProps['variant']
}

// If you experience the file picker opening twice, make sure the dropzone is not inside a <label /> tag
export const Dropzone = ({
  description,
  disabled,
  loading,
  name,
  'data-testid': dataTestId,
  className,
  variant,
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

        <Button
          variant={variant}
          icon={FilePlusIcon}
          disabled={disabled}
          loading={loading}
        >
          {description}
        </Button>
      </div>
    </div>
  )
}
