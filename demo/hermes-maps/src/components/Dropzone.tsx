import clsx from 'clsx'
import { ReactElement, ReactNode } from 'react'
import { DropzoneOptions, useDropzone } from 'react-dropzone'
import { motion, MotionProps } from 'motion/react'

const getDropzoneClassNames = ({ disabled }: { disabled?: boolean }) => {
  return clsx(
    'h-full w-full outline-hidden',
    'border border-dashed border-zinc-400 rounded-md',
    'cursor-pointer',
    'flex flex-row items-center justify-between',
    'px-4 py-2',
    {
      'bg-neutral-50 border-zinc-200 cursor-not-allowed': disabled,
      'bg-neutral-100': !disabled,
    }
  )
}

const rectVariants = {
  hidden: {
    x: 12,
    y: 20,
  },
  visible: {
    x: 24,
    y: 5,
    rotateX: 25,
    rotateZ: 25,
    rotateY: 1.4,
  },
}

const pathVariants = {
  hidden: {
    translateX: 1,
    translateY: 1,
  },
  visible: {
    translateX: 0,
    translateY: 1.5,
  },
}

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
      <motion.div
        data-testid={dataTestId}
        {...(getRootProps() as MotionProps)}
        className={getDropzoneClassNames({
          disabled,
        })}
        whileHover="visible"
        initial="hidden"
        transition={{ duration: 0.2, type: 'tween' }}
        animate={isDragActive ? 'visible' : 'hidden'}
      >
        <input {...getInputProps()} name={name} />
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 59 46"
          className="size-8"
        >
          <motion.path
            fill="#232221"
            d="m12.039 9.049.002 6.675V42.94h38.998a1 1 0 0 0 1-1V9.05c0-3.108-1.5-4.109-4.5-4.108h-31.55c-2.963 0-3.948 1.54-3.95 4.108Z"
            variants={pathVariants}
            transition={{ duration: 0.2, type: 'tween' }}
          />
          <motion.rect
            width="30"
            height="23"
            fill="#fff"
            stroke="#232221"
            strokeWidth="2"
            variants={rectVariants}
            transition={{ duration: 0.2, type: 'tween' }}

            // initial="initial"
            // animate="animate"
          />
          <motion.path
            fill="#fff"
            stroke="#232221"
            strokeWidth="2"
            d="m1.538 18.607 1.095 3.111 7.02 18.918a2 2 0 0 0 1.874 1.304h38.168a1 1 0 0 0 .937-1.348l-8.498-22.868c-1.095-2.228-2.058-3.784-5.095-3.784H5.183c-3.05 0-4.644 1.556-3.645 4.667Z"
            variants={pathVariants}
            transition={{ duration: 0.2, type: 'tween' }}
          />
        </svg>
        <div className="flex flex-col items-center gap-2 text-center">
          <span className="text-xs text-neutral-700">{description}</span>
        </div>
      </motion.div>
    </div>
  )
}
