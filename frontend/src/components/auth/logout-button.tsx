'use client'

import { useRouter } from 'next/navigation'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { useAuth } from '@/contexts/auth-context'

interface LogoutButtonProps {
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
  size?: 'default' | 'sm' | 'lg' | 'icon'
  className?: string
  children?: React.ReactNode
}

/**
 * Logout button component
 *
 * Usage:
 * ```tsx
 * <LogoutButton />
 * <LogoutButton variant="outline">Sign Out</LogoutButton>
 * <LogoutButton variant="ghost" size="sm">Logout</LogoutButton>
 * ```
 *
 * Features:
 * - Handles async logout operation
 * - Shows loading state during logout
 * - Redirects to login page after logout
 * - Customizable appearance via props
 */
export function LogoutButton({
  variant = 'outline',
  size = 'default',
  className,
  children = 'Logout',
}: LogoutButtonProps) {
  const { logout } = useAuth()
  const router = useRouter()
  const [isLoading, setIsLoading] = useState(false)

  const handleLogout = async () => {
    setIsLoading(true)

    try {
      await logout()
      // Redirect to login page after successful logout
      router.push('/login')
    } catch (error) {
      console.error('Logout error:', error)
      // Still redirect even if logout call fails (local state is cleared)
      router.push('/login')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <Button
      onClick={handleLogout}
      disabled={isLoading}
      variant={variant}
      size={size}
      className={className}
    >
      {isLoading ? 'Logging out...' : children}
    </Button>
  )
}
