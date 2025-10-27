'use client'

import { useState } from 'react'
import { AlertCircle, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { env } from '@/lib/env'
import { useAuth } from '@/contexts/auth-context'

export function UnverifiedEmailBanner() {
  const { user, accessToken } = useAuth()
  const [isResending, setIsResending] = useState(false)
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)
  const [isDismissed, setIsDismissed] = useState(false)

  // Don't show banner if user is verified or banner is dismissed
  if (!user || user.email_verified || isDismissed) {
    return null
  }

  const handleResendVerification = async () => {
    setIsResending(true)
    setMessage(null)

    try {
      const response = await fetch(`${env.apiUrl}/api/v1/auth/send-verification`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({
          error: 'Failed to send verification email'
        }))
        throw new Error(errorData.error || 'Failed to send verification email')
      }

      setMessage({
        type: 'success',
        text: 'Verification email sent! Please check your inbox.'
      })
    } catch (err) {
      setMessage({
        type: 'error',
        text: err instanceof Error ? err.message : 'Failed to send verification email'
      })
    } finally {
      setIsResending(false)
    }
  }

  return (
    <div className="border-b border-yellow-200 bg-yellow-50 dark:bg-yellow-950 dark:border-yellow-900">
      <div className="container mx-auto px-4 py-3">
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-3 flex-1">
            <AlertCircle className="h-5 w-5 text-yellow-600 dark:text-yellow-500 flex-shrink-0" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-yellow-900 dark:text-yellow-100">
                Email Not Verified
              </p>
              <p className="text-sm text-yellow-700 dark:text-yellow-300">
                Please verify your email address to access all features.
              </p>
              {message && (
                <p className={`text-sm mt-1 ${
                  message.type === 'success'
                    ? 'text-green-700 dark:text-green-400'
                    : 'text-red-700 dark:text-red-400'
                }`}>
                  {message.text}
                </p>
              )}
            </div>
          </div>

          <div className="flex items-center gap-2 flex-shrink-0">
            <Button
              variant="outline"
              size="sm"
              onClick={handleResendVerification}
              disabled={isResending}
              className="bg-white dark:bg-gray-800"
            >
              {isResending ? 'Sending...' : 'Resend Email'}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsDismissed(true)}
              className="h-8 w-8 p-0"
            >
              <X className="h-4 w-4" />
              <span className="sr-only">Dismiss</span>
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
