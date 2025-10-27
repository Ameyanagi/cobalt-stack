'use client'

import { useEffect, useState } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { env } from '@/lib/env'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { RoleBadge } from '@/components/admin/role-badge'
import { Loader2, Search, Ban, CheckCircle, ChevronLeft, ChevronRight } from 'lucide-react'
import { Badge } from '@/components/ui/badge'

interface User {
  id: string
  username: string
  email: string
  role: 'admin' | 'user'
  email_verified: boolean
  disabled_at: string | null
  last_login_at: string | null
  created_at: string
}

interface UserListResponse {
  users: User[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

export default function AdminUsersPage() {
  const { accessToken } = useAuth()
  const [data, setData] = useState<UserListResponse | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Filters
  const [page, setPage] = useState(1)
  const [search, setSearch] = useState('')
  const [roleFilter, setRoleFilter] = useState<'all' | 'admin' | 'user'>('all')
  const [verifiedFilter, setVerifiedFilter] = useState<'all' | 'verified' | 'unverified'>('all')

  const fetchUsers = async () => {
    setIsLoading(true)
    setError(null)

    try {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: '10',
      })

      if (search) params.append('search', search)
      if (roleFilter !== 'all') params.append('role', roleFilter)
      if (verifiedFilter === 'verified') params.append('email_verified', 'true')
      if (verifiedFilter === 'unverified') params.append('email_verified', 'false')

      const response = await fetch(`${env.apiUrl}/api/v1/admin/users?${params}`, {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
        },
      })

      if (!response.ok) {
        throw new Error('Failed to fetch users')
      }

      const result = await response.json()
      setData(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch users')
    } finally {
      setIsLoading(false)
    }
  }

  useEffect(() => {
    if (accessToken) {
      fetchUsers()
    }
  }, [accessToken, page, roleFilter, verifiedFilter])

  const handleSearch = () => {
    setPage(1)
    fetchUsers()
  }

  const handleToggleUserStatus = async (userId: string, isDisabled: boolean) => {
    try {
      const endpoint = isDisabled ? 'enable' : 'disable'
      const response = await fetch(`${env.apiUrl}/api/v1/admin/users/${userId}/${endpoint}`, {
        method: 'PATCH',
        headers: {
          'Authorization': `Bearer ${accessToken}`,
        },
      })

      if (!response.ok) {
        throw new Error(`Failed to ${endpoint} user`)
      }

      // Refresh user list
      fetchUsers()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Operation failed')
    }
  }

  if (error) {
    return (
      <div className="rounded-md bg-destructive/15 p-4 text-sm text-destructive">
        {error}
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">User Management</h2>
        <p className="text-muted-foreground">
          Manage user accounts and permissions
        </p>
      </div>

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Filters</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-4 flex-wrap">
            <div className="flex-1 min-w-[200px]">
              <div className="flex gap-2">
                <Input
                  placeholder="Search by username or email..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                />
                <Button onClick={handleSearch} size="icon">
                  <Search className="h-4 w-4" />
                </Button>
              </div>
            </div>

            <div className="flex gap-2">
              <Button
                variant={roleFilter === 'all' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setRoleFilter('all')}
              >
                All Roles
              </Button>
              <Button
                variant={roleFilter === 'admin' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setRoleFilter('admin')}
              >
                Admins
              </Button>
              <Button
                variant={roleFilter === 'user' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setRoleFilter('user')}
              >
                Users
              </Button>
            </div>

            <div className="flex gap-2">
              <Button
                variant={verifiedFilter === 'all' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setVerifiedFilter('all')}
              >
                All
              </Button>
              <Button
                variant={verifiedFilter === 'verified' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setVerifiedFilter('verified')}
              >
                Verified
              </Button>
              <Button
                variant={verifiedFilter === 'unverified' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setVerifiedFilter('unverified')}
              >
                Unverified
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Users Table */}
      <Card>
        <CardHeader>
          <CardTitle>
            Users {data && `(${data.total})`}
          </CardTitle>
          <CardDescription>
            {data && `Page ${data.page} of ${data.total_pages}`}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
          ) : data && data.users.length > 0 ? (
            <div className="space-y-4">
              {/* Table Header */}
              <div className="hidden md:grid grid-cols-12 gap-4 px-4 py-2 bg-gray-50 dark:bg-gray-900 rounded-lg font-medium text-sm">
                <div className="col-span-3">User</div>
                <div className="col-span-2">Role</div>
                <div className="col-span-2">Status</div>
                <div className="col-span-2">Verified</div>
                <div className="col-span-3">Actions</div>
              </div>

              {/* Table Rows */}
              {data.users.map((user) => (
                <div
                  key={user.id}
                  className="grid grid-cols-1 md:grid-cols-12 gap-4 p-4 border border-gray-200 dark:border-gray-800 rounded-lg"
                >
                  <div className="md:col-span-3">
                    <div className="font-medium">{user.username}</div>
                    <div className="text-sm text-muted-foreground">{user.email}</div>
                  </div>

                  <div className="md:col-span-2 flex items-center">
                    <RoleBadge role={user.role} />
                  </div>

                  <div className="md:col-span-2 flex items-center">
                    {user.disabled_at ? (
                      <Badge variant="destructive">Disabled</Badge>
                    ) : (
                      <Badge variant="default" className="bg-green-600">Active</Badge>
                    )}
                  </div>

                  <div className="md:col-span-2 flex items-center">
                    {user.email_verified ? (
                      <Badge variant="default">Verified</Badge>
                    ) : (
                      <Badge variant="secondary">Unverified</Badge>
                    )}
                  </div>

                  <div className="md:col-span-3 flex items-center gap-2">
                    <Button
                      size="sm"
                      variant={user.disabled_at ? 'default' : 'destructive'}
                      onClick={() => handleToggleUserStatus(user.id, !!user.disabled_at)}
                    >
                      {user.disabled_at ? (
                        <>
                          <CheckCircle className="h-4 w-4 mr-1" />
                          Enable
                        </>
                      ) : (
                        <>
                          <Ban className="h-4 w-4 mr-1" />
                          Disable
                        </>
                      )}
                    </Button>
                  </div>
                </div>
              ))}

              {/* Pagination */}
              <div className="flex items-center justify-between pt-4">
                <div className="text-sm text-muted-foreground">
                  Showing {((data.page - 1) * data.per_page) + 1} to{' '}
                  {Math.min(data.page * data.per_page, data.total)} of {data.total} users
                </div>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage(p => Math.max(1, p - 1))}
                    disabled={data.page === 1}
                  >
                    <ChevronLeft className="h-4 w-4 mr-1" />
                    Previous
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage(p => p + 1)}
                    disabled={data.page >= data.total_pages}
                  >
                    Next
                    <ChevronRight className="h-4 w-4 ml-1" />
                  </Button>
                </div>
              </div>
            </div>
          ) : (
            <div className="text-center py-12 text-muted-foreground">
              No users found
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
