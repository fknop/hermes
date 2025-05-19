import * as RadixSlider from '@radix-ui/react-slider'

export function Slider({
  defaultValue,
  max = 100,
  min = 1,
  value,
  onChange,
}: {
  defaultValue?: number
  min?: number
  max?: number
  value: number
  onChange: (value: number) => void
}) {
  return (
    <RadixSlider.Root
      className="relative flex h-6 w-full touch-none select-none items-center rounded-full"
      defaultValue={defaultValue ? [defaultValue] : undefined}
      max={max}
      min={min}
      step={1}
      value={[value]}
      onValueChange={(values) => onChange(values[0])}
    >
      <RadixSlider.Track className="relative h-[3px] grow rounded-full bg-neutral-200">
        <RadixSlider.Range className="absolute h-full rounded-full bg-primary-active" />
      </RadixSlider.Track>
      <RadixSlider.Thumb
        className="block size-4 rounded-[10px] bg-white border-2 border-primary hover:bg-primary-hover  focus:outline-none"
        aria-label="Volume"
      />
    </RadixSlider.Root>
  )
}
