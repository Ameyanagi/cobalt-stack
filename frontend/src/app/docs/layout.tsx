'use client'

import { useState } from 'react'
import Link from 'next/link'
import { ThemeToggle } from '@/components/theme/theme-toggle'
import { ThemeSelector } from '@/components/theme/theme-selector'
import { DocsSidebar, DocsSidebarContent } from '@/components/docs/docs-sidebar'
import { BookOpen, Github, Zap, Menu, MoreVertical } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Sheet,
  SheetContent,
  SheetTrigger,
} from '@/components/ui/sheet'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const [open, setOpen] = useState(false)

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 sticky top-0 z-50">
        <div className="container mx-auto px-4 py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              {/* Mobile menu button */}
              <Sheet open={open} onOpenChange={setOpen}>
                <SheetTrigger asChild>
                  <Button variant="ghost" size="icon" className="lg:hidden">
                    <Menu className="h-5 w-5" />
                    <span className="sr-only">Toggle navigation menu</span>
                  </Button>
                </SheetTrigger>
                <SheetContent side="left" className="w-64 p-0">
                  <DocsSidebarContent onLinkClick={() => setOpen(false)} />
                </SheetContent>
              </Sheet>

              <Link href="/" className="flex items-center gap-2">
                <div className="h-7 w-7 rounded-lg bg-primary flex items-center justify-center">
                  <Zap className="h-4 w-4 text-primary-foreground" />
                </div>
                <span className="text-lg font-bold">Cobalt Stack</span>
              </Link>
              <span className="text-sm text-muted-foreground hidden sm:inline">|</span>
              <div className="hidden sm:flex items-center gap-2 text-sm font-medium">
                <BookOpen className="h-4 w-4" />
                Documentation
              </div>
            </div>

            <div className="flex items-center gap-2">
              {/* Desktop: Full header */}
              <div className="hidden lg:flex items-center gap-2">
                <Link href="/">
                  <Button variant="ghost" size="sm">Home</Button>
                </Link>
                <Link href="/api-docs">
                  <Button variant="ghost" size="sm">Rust API Docs</Button>
                </Link>
                <a
                  href={`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000'}/swagger-ui`}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Button variant="ghost" size="sm">Swagger API</Button>
                </a>
                <a
                  href="https://github.com/Ameyanagi/cobalt-stack"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Button variant="ghost" size="sm">
                    <Github className="h-4 w-4" />
                  </Button>
                </a>
                <ThemeSelector />
                <ThemeToggle />
              </div>

              {/* Tablet: Partial header + overflow menu */}
              <div className="hidden md:flex lg:hidden items-center gap-2">
                <Link href="/">
                  <Button variant="ghost" size="sm">Home</Button>
                </Link>
                <Link href="/api-docs">
                  <Button variant="ghost" size="sm">API Docs</Button>
                </Link>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" size="icon">
                      <MoreVertical className="h-4 w-4" />
                      <span className="sr-only">More options</span>
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem asChild>
                      <a
                        href={`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000'}/swagger-ui`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="cursor-pointer"
                      >
                        Swagger API
                      </a>
                    </DropdownMenuItem>
                    <DropdownMenuItem asChild>
                      <a
                        href="https://github.com/Ameyanagi/cobalt-stack"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="cursor-pointer"
                      >
                        GitHub
                      </a>
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
                <ThemeToggle />
              </div>

              {/* Mobile: Minimal header + overflow menu */}
              <div className="flex md:hidden items-center gap-1">
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" size="icon">
                      <MoreVertical className="h-4 w-4" />
                      <span className="sr-only">Menu</span>
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem asChild>
                      <Link href="/" className="cursor-pointer">Home</Link>
                    </DropdownMenuItem>
                    <DropdownMenuItem asChild>
                      <Link href="/api-docs" className="cursor-pointer">Rust API Docs</Link>
                    </DropdownMenuItem>
                    <DropdownMenuItem asChild>
                      <a
                        href={`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000'}/swagger-ui`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="cursor-pointer"
                      >
                        Swagger API
                      </a>
                    </DropdownMenuItem>
                    <DropdownMenuItem asChild>
                      <a
                        href="https://github.com/Ameyanagi/cobalt-stack"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="cursor-pointer"
                      >
                        GitHub
                      </a>
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
                <ThemeToggle />
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content with Sidebar */}
      <div className="flex flex-1">
        <DocsSidebar />
        <main className="flex-1 overflow-y-auto">
          {children}
        </main>
      </div>
    </div>
  )
}
