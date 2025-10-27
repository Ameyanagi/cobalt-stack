'use client'

import { CheckCircle2, Loader2, XCircle } from 'lucide-react'
import Link from 'next/link'
import { useRouter, useSearchParams } from 'next/navigation'
import { Suspense, useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { env } from '@/lib/env'

type VerificationState = 'verifying' | 'success' | 'error'

function VerifyEmailContent() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const token = searchParams.get('token')

  const [state, setState] = useState<VerificationState>('verifying')
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const verifyEmail = async () => {
      if (!token) {
        setState('error')
        setError('Verification token is missing')
        return
      }

      try {
        const response = await fetch(`${env.apiUrl}/api/v1/auth/verify-email`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ token }),
        })

        if (!response.ok) {
          const errorData = await response.json().catch(() => ({
            error: 'Email verification failed',
          }))
          throw new Error(errorData.error || 'Email verification failed')
        }

        setState('success')
      } catch (err) {
        setState('error')
        setError(err instanceof Error ? err.message : 'Email verification failed')
      }
    }

    verifyEmail()
  }, [token])

  return (
    <div className="flex min-h-screen items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4">
            {state === 'verifying' && <Loader2 className="h-16 w-16 animate-spin text-primary" />}
            {state === 'success' && <CheckCircle2 className="h-16 w-16 text-green-600" />}
            {state === 'error' && <XCircle className="h-16 w-16 text-destructive" />}
          </div>
          <CardTitle>
            {state === 'verifying' && 'Verifying Email'}
            {state === 'success' && 'Email Verified'}
            {state === 'error' && 'Verification Failed'}
          </CardTitle>
          <CardDescription>
            {state === 'verifying' && 'Please wait while we verify your email address...'}
            {state === 'success' && 'Your email has been successfully verified!'}
            {state === 'error' && 'We could not verify your email address.'}
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          {state === 'error' && error && (
            <div className="rounded-md bg-destructive/15 p-3 text-sm text-destructive">{error}</div>
          )}

          {state === 'success' && (
            <div className="space-y-4">
              <p className="text-center text-sm text-muted-foreground">
                You can now access all features of your account.
              </p>
              <Button className="w-full" onClick={() => router.push('/dashboard')}>
                Go to Dashboard
              </Button>
            </div>
          )}

          {state === 'error' && (
            <div className="space-y-4">
              <p className="text-center text-sm text-muted-foreground">
                The verification link may have expired or is invalid.
              </p>
              <div className="flex flex-col gap-2">
                <Button className="w-full" variant="outline" asChild>
                  <Link href="/login">Back to Login</Link>
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

export default function VerifyEmailPage() {
  return (
    <Suspense
      fallback={
        <div className="flex min-h-screen items-center justify-center">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      }
    >
      <VerifyEmailContent />
    </Suspense>
  )
}
