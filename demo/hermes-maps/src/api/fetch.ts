import { API_URL } from '@/constants'

const getBody = <T>(c: Response | Request): Promise<T> => {
  const contentType = c.headers.get('content-type')

  if (contentType && contentType.includes('application/json')) {
    return c.json()
  }

  if (contentType && contentType.includes('application/pdf')) {
    return c.blob() as Promise<T>
  }

  return c.text() as Promise<T>
}

export async function fetchApi<T>(
  url: string,
  options: RequestInit
): Promise<T> {
  const response = await fetch(`${API_URL}${url}`, options)

  const data = await getBody<T>(response)
  return { status: response.status, data, headers: response.headers } as T
}
