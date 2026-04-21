import { apiFetch } from '@/api/client'

export const fetchBookDetails = async (bookId: string) => {
  try {
    const response = await apiFetch(`/api/google-books/volume/${bookId}`)
    if (response.ok) {
      return await response.json()
    } else {
      console.error('Failed to fetch book details:', await response.json())
      return null
    }
  } catch (error) {
    console.error('Failed to fetch book details:', error)
    return null
  }
}

export const searchBooks = async (query: string) => {
  try {
    const response = await apiFetch(
      `/api/google-books/search?q=${encodeURIComponent(query)}`
    )
    if (response.ok) {
      const data = await response.json()
      return data.items || []
    } else {
      console.error('Failed to fetch books:', await response.json())
      return []
    }
  } catch (error) {
    console.error('Failed to fetch books:', error)
    return []
  }
}
