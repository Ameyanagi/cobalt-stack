'use client'

import { useEffect, useState } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { env } from '@/lib/env'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Users, UserCheck, Shield, Loader2 } from 'lucide-react'

interface AdminStats {
  total_users: number
  verified_users: number
  admin_users: number
}

export default function AdminDashboardPage() {
  const { accessToken } = useAuth()
  const [stats, setStats] = useState<AdminStats | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const response = await fetch(`${env.apiUrl}/api/admin/stats`, {
          headers: {
            'Authorization': `Bearer ${accessToken}`,
          },
        })

        if (!response.ok) {
          throw new Error('Failed to fetch stats')
        }

        const data = await response.json()
        setStats(data)
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch stats')
      } finally {
        setIsLoading(false)
      }
    }

    if (accessToken) {
      fetchStats()
    }
  }, [accessToken])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="rounded-md bg-destructive/15 p-4 text-sm text-destructive">
        {error}
      </div>
    )
  }

  const verificationRate = stats
    ? stats.total_users > 0
      ? Math.round((stats.verified_users / stats.total_users) * 100)
      : 0
    : 0

  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">Dashboard</h2>
        <p className="text-gray-600 dark:text-gray-400">
          Overview of your platform statistics
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Total Users
            </CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-gray-900 dark:text-white">{stats?.total_users || 0}</div>
            <p className="text-xs text-gray-600 dark:text-gray-400">
              All registered users
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Verified Users
            </CardTitle>
            <UserCheck className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-gray-900 dark:text-white">{stats?.verified_users || 0}</div>
            <p className="text-xs text-gray-600 dark:text-gray-400">
              {verificationRate}% verification rate
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Administrators
            </CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-gray-900 dark:text-white">{stats?.admin_users || 0}</div>
            <p className="text-xs text-gray-600 dark:text-gray-400">
              Users with admin access
            </p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Quick Actions</CardTitle>
          <CardDescription>
            Common administrative tasks
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          <a
            href="/admin/users"
            className="block rounded-lg border border-gray-200 dark:border-gray-800 p-4 hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors"
          >
            <div className="flex items-center gap-3">
              <Users className="h-5 w-5 text-primary" />
              <div>
                <p className="font-medium text-gray-900 dark:text-white">Manage Users</p>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  View, disable, or enable user accounts
                </p>
              </div>
            </div>
          </a>
        </CardContent>
      </Card>
    </div>
  )
}
