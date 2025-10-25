'use client'

import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { ProtectedRoute } from '@/components/auth/protected-route'
import { useAuth } from '@/contexts/auth-context'
import { LogoutButton } from '@/components/auth/logout-button'
import { env } from '@/lib/env'

export default function Dashboard() {
  const { user } = useAuth()

  return (
    <ProtectedRoute>
      <main className="flex min-h-screen flex-col p-24">
        <div className="max-w-4xl w-full mx-auto space-y-8">
          {/* Header */}
          <div className="flex justify-between items-center">
            <div>
              <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
              <p className="text-muted-foreground mt-1">
                Welcome back, {user?.username}!
              </p>
            </div>
            <div className="flex gap-2">
              <Link href="/">
                <Button variant="outline">Home</Button>
              </Link>
              <LogoutButton variant="outline" />
            </div>
          </div>

          {/* User Info Card */}
          <Card>
            <CardHeader>
              <CardTitle>Account Information</CardTitle>
              <CardDescription>Your account details and status</CardDescription>
            </CardHeader>
            <CardContent>
              <dl className="space-y-4">
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">Username</dt>
                  <dd className="mt-1 text-sm">{user?.username}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">Email</dt>
                  <dd className="mt-1 text-sm">{user?.email}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">Email Verified</dt>
                  <dd className="mt-1 text-sm">
                    {user?.email_verified ? (
                      <span className="inline-flex items-center rounded-full bg-green-50 px-2 py-1 text-xs font-medium text-green-700 ring-1 ring-inset ring-green-600/20">
                        Verified
                      </span>
                    ) : (
                      <span className="inline-flex items-center rounded-full bg-yellow-50 px-2 py-1 text-xs font-medium text-yellow-800 ring-1 ring-inset ring-yellow-600/20">
                        Not Verified
                      </span>
                    )}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">User ID</dt>
                  <dd className="mt-1 text-sm font-mono text-xs">{user?.id}</dd>
                </div>
              </dl>
            </CardContent>
          </Card>

          {/* Quick Actions */}
          <div className="grid gap-4 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle>System Health</CardTitle>
                <CardDescription>Check backend API status</CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/health">
                  <Button className="w-full">View Health Status</Button>
                </Link>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>API Documentation</CardTitle>
                <CardDescription>Explore the API endpoints</CardDescription>
              </CardHeader>
              <CardContent>
                <a
                  href={`${env.apiUrl}/swagger-ui`}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Button variant="outline" className="w-full">
                    Open API Docs
                  </Button>
                </a>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>
    </ProtectedRoute>
  )
}
