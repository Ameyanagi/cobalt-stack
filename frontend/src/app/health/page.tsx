'use client'

/**
 * Health Check Page
 * Displays the current health status of the backend API
 */

import { useQuery } from '@tanstack/react-query'
import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { apiClient } from '@/lib/api-client'

export default function HealthPage() {
  const { data, isLoading, isError, error, refetch } = useQuery({
    queryKey: ['health'],
    queryFn: apiClient.healthCheck,
    refetchInterval: 30000, // Refetch every 30 seconds
  })

  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <div className="max-w-2xl w-full space-y-8">
        <div className="text-center">
          <h1 className="text-4xl font-bold tracking-tight mb-4">System Health</h1>
          <p className="text-lg text-gray-600">
            Real-time health status of the Cobalt Stack backend
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>API Health Status</CardTitle>
            <CardDescription>Current status of the backend service</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {isLoading && (
              <div className="text-center py-8">
                <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]" />
                <p className="mt-4 text-gray-600">Checking health status...</p>
              </div>
            )}

            {isError && (
              <div className="text-center py-8">
                <div className="text-red-500 text-4xl mb-4">⚠️</div>
                <p className="text-red-600 font-semibold mb-2">Failed to connect to API</p>
                <p className="text-gray-600 text-sm">
                  {error instanceof Error ? error.message : 'Unknown error'}
                </p>
              </div>
            )}

            {data && (
              <>
                {data.success ? (
                  <div className="text-center py-8">
                    <div className="text-green-500 text-4xl mb-4">✓</div>
                    <p className="text-green-600 font-semibold text-xl mb-2">
                      Service is {data.data.status}
                    </p>
                    <p className="text-gray-600 text-sm">All systems operational</p>
                  </div>
                ) : (
                  <div className="text-center py-8">
                    <div className="text-red-500 text-4xl mb-4">✗</div>
                    <p className="text-red-600 font-semibold mb-2">Service Error</p>
                    <p className="text-gray-600 text-sm">{data.error}</p>
                  </div>
                )}
              </>
            )}

            <div className="flex justify-center space-x-4 pt-4">
              <Button onClick={() => refetch()} variant="outline">
                Refresh Status
              </Button>
              <Link href="/">
                <Button>Back to Home</Button>
              </Link>
            </div>
          </CardContent>
        </Card>

        <div className="text-center text-sm text-gray-500">
          <p>Status refreshes automatically every 30 seconds</p>
        </div>
      </div>
    </main>
  )
}
