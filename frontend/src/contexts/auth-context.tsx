'use client'

import React, { createContext, useContext, useState, useEffect, useCallback } from 'react'
import { env } from '@/lib/env'

// Types for authentication state
interface User {
  id: string
  username: string
  email: string
  email_verified: boolean
}

interface AuthState {
  user: User | null
  accessToken: string | null
  isLoading: boolean
  isAuthenticated: boolean
}

interface AuthContextType extends AuthState {
  login: (accessToken: string, user: User) => void
  logout: () => Promise<void>
  refreshToken: () => Promise<boolean>
  updateUser: (user: User) => void
}

// Create context with undefined default (will be provided by AuthProvider)
const AuthContext = createContext<AuthContextType | undefined>(undefined)

/**
 * AuthProvider component that manages authentication state
 *
 * Handles:
 * - Access token storage in memory
 * - Refresh token in HttpOnly cookie (backend-managed)
 * - Automatic token refresh (via useTokenRefresh hook)
 * - User state management
 */
export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [authState, setAuthState] = useState<AuthState>({
    user: null,
    accessToken: null,
    isLoading: true,
    isAuthenticated: false,
  })

  // Auto-refresh timer ref
  const refreshTimeoutRef = React.useRef<NodeJS.Timeout | null>(null)

  /**
   * Store access token and user in memory
   */
  const login = useCallback((accessToken: string, user: User) => {
    setAuthState({
      user,
      accessToken,
      isLoading: false,
      isAuthenticated: true,
    })
  }, [])

  /**
   * Clear authentication state and revoke refresh token
   */
  const logout = useCallback(async () => {
    try {
      // Call logout endpoint to revoke refresh token
      const response = await fetch(`${env.apiUrl}/api/auth/logout`, {
        method: 'POST',
        credentials: 'include', // Send HttpOnly cookie
      })

      if (!response.ok) {
        console.error('Logout failed:', response.statusText)
      }
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      // Always clear local state
      setAuthState({
        user: null,
        accessToken: null,
        isLoading: false,
        isAuthenticated: false,
      })
    }
  }, [])

  /**
   * Refresh access token using refresh token cookie
   * Returns true if successful, false otherwise
   */
  const refreshToken = useCallback(async (): Promise<boolean> => {
    try {
      const response = await fetch(`${env.apiUrl}/api/auth/refresh`, {
        method: 'POST',
        credentials: 'include', // Send HttpOnly cookie
        headers: {
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        // Refresh failed - clear state
        setAuthState({
          user: null,
          accessToken: null,
          isLoading: false,
          isAuthenticated: false,
        })
        return false
      }

      const data = await response.json()

      // Fetch user info with new access token
      const userResponse = await fetch(`${env.apiUrl}/api/auth/me`, {
        headers: {
          'Authorization': `Bearer ${data.access_token}`,
        },
      })

      if (!userResponse.ok) {
        throw new Error('Failed to fetch user info')
      }

      const user = await userResponse.json()

      // Update state with new token and user
      setAuthState({
        user,
        accessToken: data.access_token,
        isLoading: false,
        isAuthenticated: true,
      })

      return true
    } catch (error) {
      console.error('Token refresh error:', error)
      setAuthState({
        user: null,
        accessToken: null,
        isLoading: false,
        isAuthenticated: false,
      })
      return false
    }
  }, [])

  /**
   * Update user information
   */
  const updateUser = useCallback((user: User) => {
    setAuthState(prev => ({
      ...prev,
      user,
    }))
  }, [])

  /**
   * Initialize authentication state on mount
   * Try to refresh token to restore session
   */
  useEffect(() => {
    let mounted = true

    const initialize = async () => {
      // Try to refresh token on mount (restore session)
      const success = await refreshToken()

      if (mounted && !success) {
        // No valid session
        setAuthState({
          user: null,
          accessToken: null,
          isLoading: false,
          isAuthenticated: false,
        })
      }
    }

    initialize()

    return () => {
      mounted = false
    }
  }, [refreshToken])

  /**
   * Auto-refresh token before expiry
   * Refresh 5 minutes before 30-minute expiry (at 25 minute mark)
   */
  useEffect(() => {
    if (!authState.isAuthenticated || !authState.accessToken) {
      if (refreshTimeoutRef.current) {
        clearTimeout(refreshTimeoutRef.current)
        refreshTimeoutRef.current = null
      }
      return
    }

    const REFRESH_BEFORE_EXPIRY = 25 * 60 * 1000 // 25 minutes

    const scheduleRefresh = () => {
      if (refreshTimeoutRef.current) {
        clearTimeout(refreshTimeoutRef.current)
      }

      refreshTimeoutRef.current = setTimeout(async () => {
        console.log('Auto-refreshing access token...')
        const success = await refreshToken()
        if (success) {
          scheduleRefresh()
        }
      }, REFRESH_BEFORE_EXPIRY)
    }

    scheduleRefresh()

    return () => {
      if (refreshTimeoutRef.current) {
        clearTimeout(refreshTimeoutRef.current)
      }
    }
  }, [authState.accessToken, authState.isAuthenticated, refreshToken])

  const value: AuthContextType = {
    ...authState,
    login,
    logout,
    refreshToken,
    updateUser,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

/**
 * Hook to access authentication context
 *
 * @throws Error if used outside AuthProvider
 */
export function useAuth() {
  const context = useContext(AuthContext)

  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }

  return context
}
