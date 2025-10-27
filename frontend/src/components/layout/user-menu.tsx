'use client'

import Link from 'next/link'
import { useAuth } from '@/contexts/auth-context'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Badge } from '@/components/ui/badge'
import {
  User,
  LogOut,
  LayoutDashboard,
  ChevronDown,
  Shield,
  Mail,
  CheckCircle2,
  XCircle
} from 'lucide-react'

export function UserMenu() {
  const { user, isAuthenticated, isLoading, logout } = useAuth()

  // Loading state
  if (isLoading) {
    return (
      <div className="flex items-center gap-2">
        <div className="h-8 w-8 rounded-full bg-muted animate-pulse" />
        <div className="h-4 w-20 bg-muted rounded animate-pulse hidden sm:block" />
      </div>
    )
  }

  // Not authenticated state
  if (!isAuthenticated || !user) {
    return (
      <div className="flex items-center gap-2">
        <Link href="/login">
          <Button variant="ghost" size="sm">
            Login
          </Button>
        </Link>
        <Link href="/register">
          <Button size="sm">Sign Up</Button>
        </Link>
      </div>
    )
  }

  // Authenticated state - Avatar dropdown menu
  const userInitial = user.username?.[0]?.toUpperCase() || user.email?.[0]?.toUpperCase() || 'U'
  const isAdmin = user.role === 'admin'

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          className="flex items-center gap-2 h-auto py-2 px-3"
        >
          <Avatar className="h-8 w-8">
            <AvatarFallback className="bg-primary text-primary-foreground font-medium">
              {userInitial}
            </AvatarFallback>
          </Avatar>

          {/* Username - hidden on mobile */}
          <span className="hidden sm:inline-flex text-sm font-medium">
            {user.username}
          </span>

          {/* Role badge - hidden on mobile */}
          <Badge
            variant={isAdmin ? 'destructive' : 'secondary'}
            className="hidden md:inline-flex text-xs"
          >
            {user.role}
          </Badge>

          <ChevronDown className="h-4 w-4 text-muted-foreground" />
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align="end" className="w-56">
        {/* User Info Section */}
        <DropdownMenuLabel>
          <div className="flex flex-col space-y-1">
            <div className="flex items-center gap-2">
              <User className="h-4 w-4" />
              <span className="font-medium">{user.username}</span>
            </div>
            <div className="flex items-center gap-2 text-xs text-muted-foreground font-normal">
              <Mail className="h-3 w-3" />
              <span className="truncate">{user.email}</span>
            </div>
            <div className="flex items-center gap-2 mt-1">
              <Badge
                variant={isAdmin ? 'destructive' : 'secondary'}
                className="text-xs"
              >
                <Shield className="h-3 w-3 mr-1" />
                {user.role}
              </Badge>
              {user.email_verified ? (
                <Badge variant="outline" className="text-xs text-green-600 border-green-600">
                  <CheckCircle2 className="h-3 w-3 mr-1" />
                  Verified
                </Badge>
              ) : (
                <Badge variant="outline" className="text-xs text-orange-600 border-orange-600">
                  <XCircle className="h-3 w-3 mr-1" />
                  Unverified
                </Badge>
              )}
            </div>
          </div>
        </DropdownMenuLabel>

        <DropdownMenuSeparator />

        {/* Navigation Links */}
        <DropdownMenuItem asChild>
          <Link href="/dashboard" className="cursor-pointer">
            <LayoutDashboard className="h-4 w-4 mr-2" />
            Dashboard
          </Link>
        </DropdownMenuItem>

        {isAdmin && (
          <DropdownMenuItem asChild>
            <Link href="/admin" className="cursor-pointer">
              <Shield className="h-4 w-4 mr-2" />
              Admin Panel
            </Link>
          </DropdownMenuItem>
        )}

        <DropdownMenuSeparator />

        {/* Logout */}
        <DropdownMenuItem
          className="cursor-pointer text-destructive focus:text-destructive"
          onClick={() => logout()}
        >
          <LogOut className="h-4 w-4 mr-2" />
          Logout
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
