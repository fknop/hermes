export const DateTimeFormat = {
  DATETIME_MED: {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
  },
} satisfies { [key: string]: Intl.DateTimeFormatOptions }
