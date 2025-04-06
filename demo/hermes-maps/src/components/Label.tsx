export function Label({ children }: { children: React.ReactNode }) {
  return (
    <label className="inline-flex gap-2 items-center text-sm font-medium text-gray-700">
      {children}
    </label>
  )
}
