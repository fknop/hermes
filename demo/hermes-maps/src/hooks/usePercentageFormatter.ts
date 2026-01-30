export function usePercentageFormatter() {
  return (value: number, options?: Intl.NumberFormatOptions) => {
    const formatter = new Intl.NumberFormat('en-GB', {
      style: 'percent',
      maximumFractionDigits: 2,
      ...options,
    })

    return formatter.format(value)
  }
}
