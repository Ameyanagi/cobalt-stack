'use client'

import { useAuth } from '@/contexts/auth-context'
import { useRouter, usePathname } from 'next/navigation'
import { useEffect } from 'react'
import Link from 'next/link'
import { Shield, Users, BarChart3 } from 'lucide-react'
import { cn } from '@/lib/utils'
import { ThemeToggle } from '@/components/theme/theme-toggle'
import { ThemeSelector } from '@/components/theme/theme-selector'

export default function AdminLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const { user, isLoading } = useAuth()
  const router = useRouter()
  const pathname = usePathname()

  useEffect(() => {
    if (!isLoading && (!user || user.role !== 'admin')) {
      router.push('/dashboard')
    }
  }, [user, isLoading, router])

  // Show loading state while checking auth
  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent mx-auto mb-4"></div>
          <p className="text-muted-foreground">Loading...</p>
        </div>
      </div>
    )
  }

  // Don't render admin content if not admin
  if (!user || user.role !== 'admin') {
    return null
  }

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      {/* Admin Header */}
      <header className="border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Shield className="h-6 w-6 text-primary" />
              <h1 className="text-xl font-bold">Admin Panel</h1>
            </div>
            <nav className="flex items-center gap-6">
              <Link
                href="/admin"
                className={cn(
                  "flex items-center gap-2 text-sm font-medium hover:text-primary transition-colors",
                  pathname === '/admin' && "text-primary"
                )}
              >
                <BarChart3 className="h-4 w-4" />
                Dashboard
              </Link>
              <Link
                href="/admin/users"
                className={cn(
                  "flex items-center gap-2 text-sm font-medium hover:text-primary transition-colors",
                  pathname?.startsWith('/admin/users') && "text-primary"
                )}
              >
                <Users className="h-4 w-4" />
                Users
              </Link>
              <Link
                href="/dashboard"
                className="text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                Back to App
              </Link>
              <div className="flex items-center gap-2 ml-4">
                <ThemeSelector />
                <ThemeToggle />
              </div>
            </nav>
          </div>
        </div>
      </header>

      {/* Admin Content */}
      <main className="container mx-auto px-4 py-8">
        {children}
      </main>
    </div>
  )
}
