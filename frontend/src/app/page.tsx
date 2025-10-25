'use client'

import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useAuth } from '@/contexts/auth-context'
import { LogoutButton } from '@/components/auth/logout-button'

export default function Home() {
  const { user, isAuthenticated, isLoading } = useAuth()

  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <div className="max-w-2xl w-full space-y-8">
        {/* Authentication Status */}
        <div className="flex justify-end">
          {isLoading ? (
            <div className="text-sm text-muted-foreground">Loading...</div>
          ) : isAuthenticated && user ? (
            <div className="flex items-center gap-4">
              <div className="text-sm">
                <span className="text-muted-foreground">Welcome, </span>
                <span className="font-medium">{user.username}</span>
              </div>
              <LogoutButton variant="outline" size="sm" />
            </div>
          ) : (
            <div className="flex gap-2">
              <Link href="/login">
                <Button variant="outline" size="sm">Login</Button>
              </Link>
              <Link href="/register">
                <Button size="sm">Register</Button>
              </Link>
            </div>
          )}
        </div>

        <div className="text-center">
          <h1 className="text-4xl font-bold tracking-tight mb-4">
            Cobalt Stack
          </h1>
          <p className="text-lg text-gray-600">
            Full-stack application with Rust backend (Axum + SeaORM) and Next.js 16 frontend
          </p>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Backend</CardTitle>
              <CardDescription>Rust + Axum + SeaORM</CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="list-disc list-inside space-y-1 text-sm">
                <li>Axum 0.7+ web framework</li>
                <li>PostgreSQL with SeaORM</li>
                <li>Redis caching</li>
                <li>OpenAPI documentation</li>
                <li>JWT authentication with token rotation</li>
              </ul>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Frontend</CardTitle>
              <CardDescription>Next.js 16 + TypeScript</CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="list-disc list-inside space-y-1 text-sm">
                <li>Next.js 16 App Router</li>
                <li>TailwindCSS styling</li>
                <li>shadcn/ui components</li>
                <li>Type-safe API client</li>
                <li>Secure authentication flow</li>
              </ul>
            </CardContent>
          </Card>
        </div>

        <div className="flex justify-center gap-4">
          <Link href="/health">
            <Button size="lg" variant="outline">
              Check System Health
            </Button>
          </Link>
          {isAuthenticated && (
            <Link href="/dashboard">
              <Button size="lg">
                Go to Dashboard
              </Button>
            </Link>
          )}
        </div>
      </div>
    </main>
  )
}
