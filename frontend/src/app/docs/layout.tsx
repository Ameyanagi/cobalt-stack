import Link from 'next/link'
import { ThemeToggle } from '@/components/theme/theme-toggle'
import { ThemeSelector } from '@/components/theme/theme-selector'
import { DocsSidebar } from '@/components/docs/docs-sidebar'
import { BookOpen, Github, Zap } from 'lucide-react'
import { Button } from '@/components/ui/button'

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 sticky top-0 z-50">
        <div className="container mx-auto px-4 py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-6">
              <Link href="/" className="flex items-center gap-2">
                <div className="h-7 w-7 rounded-lg bg-primary flex items-center justify-center">
                  <Zap className="h-4 w-4 text-primary-foreground" />
                </div>
                <span className="text-lg font-bold">Cobalt Stack</span>
              </Link>
              <span className="text-sm text-muted-foreground">|</span>
              <div className="flex items-center gap-2 text-sm font-medium">
                <BookOpen className="h-4 w-4" />
                Documentation
              </div>
            </div>

            <div className="flex items-center gap-2">
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
