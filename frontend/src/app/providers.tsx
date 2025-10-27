'use client'

/**
 * Client-side providers for the application
 */

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { useState } from 'react'
import { UnverifiedEmailBanner } from '@/components/auth/unverified-email-banner'
import { AuthProvider } from '@/contexts/auth-context'

export function Providers({ children }: { children: React.ReactNode }) {
  // Create QueryClient instance per-request to avoid state sharing
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // With SSR, we usually want to set some default staleTime
            // above 0 to avoid refetching immediately on the client
            staleTime: 60 * 1000, // 1 minute
            retry: 1,
          },
        },
      })
  )

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <UnverifiedEmailBanner />
        {children}
      </AuthProvider>
    </QueryClientProvider>
  )
}
