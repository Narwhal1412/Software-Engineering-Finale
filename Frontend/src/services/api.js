export async function getHello() {
  const response = await fetch('/api/v1/hello')

  if (!response.ok) {
    throw new Error('Unable to load data from the backend.')
  }

  return response.json()
}
