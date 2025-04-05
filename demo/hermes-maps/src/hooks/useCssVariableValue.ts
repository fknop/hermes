import { useMemo } from 'react'

function getValue(variableName: string): string {
  const root = document.documentElement
  return getComputedStyle(root).getPropertyValue(variableName)
}

export function useCssVariableValue(variableName: `--${string}`): string {
  return useMemo(() => {
    return getValue(variableName)
  }, [variableName])
}
