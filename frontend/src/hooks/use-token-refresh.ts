import { useEffect, useRef } from 'react'
import { useAuth } from '@/contexts/auth-context'

/**
 * Hook to automatically refresh access token before expiry
 *
 * Token refresh strategy:
 * - Access tokens expire in 30 minutes
 * - Refresh 5 minutes before expiry (at 25 minute mark)
 * - Set up interval to check every minute
 */
export function useTokenRefresh() {
  const { accessToken, refreshToken, isAuthenticated } = useAuth()
  const refreshTimeoutRef = useRef<NodeJS.Timeout | null>(null)

  useEffect(() => {
    if (!isAuthenticated || !accessToken) {
      // Clear any existing timeout
      if (refreshTimeoutRef.current) {
        clearTimeout(refreshTimeoutRef.current)
        refreshTimeoutRef.current = null
      }
      return
    }

    // Schedule token refresh for 25 minutes (5 minutes before 30 min expiry)
    const REFRESH_BEFORE_EXPIRY = 25 * 60 * 1000 // 25 minutes in ms

    const scheduleRefresh = () => {
      if (refreshTimeoutRef.current) {
        clearTimeout(refreshTimeoutRef.current)
      }

      refreshTimeoutRef.current = setTimeout(async () => {
        console.log('Auto-refreshing access token...')
        const success = await refreshToken()

        if (success) {
          // Schedule next refresh
          scheduleRefresh()
        } else {
          console.error('Token refresh failed - user will need to re-login')
        }
      }, REFRESH_BEFORE_EXPIRY)
    }

    // Start the refresh cycle
    scheduleRefresh()

    // Cleanup on unmount
    return () => {
      if (refreshTimeoutRef.current) {
        clearTimeout(refreshTimeoutRef.current)
      }
    }
  }, [accessToken, isAuthenticated, refreshToken])
}
