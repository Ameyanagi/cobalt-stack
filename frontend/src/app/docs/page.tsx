import {
  AlertCircle,
  ArrowRight,
  BookOpen,
  Code2,
  FileCode,
  Palette,
  Server,
  Shield,
  Users,
  Wrench,
} from 'lucide-react'
import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { docsNav } from '@/lib/docs-nav'

const sectionIcons: Record<string, any> = {
  'getting-started': BookOpen,
  backend: Server,
  frontend: Code2,
  guides: Wrench,
  architecture: FileCode,
  api: Shield,
  deployment: Server,
  contributing: Users,
  troubleshooting: AlertCircle,
}

export default function DocsIndexPage() {
  return (
    <div className="container max-w-6xl mx-auto px-4 py-12">
      {/* Hero */}
      <div className="text-center mb-12">
        <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border bg-muted/50 text-sm mb-6">
          <BookOpen className="h-4 w-4" />
          <span>Comprehensive Documentation</span>
        </div>

        <h1 className="text-5xl font-bold mb-4">
          Cobalt Stack
          <br />
          <span className="bg-gradient-to-r from-primary to-accent bg-clip-text text-transparent">
            Documentation
          </span>
        </h1>

        <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
          Everything you need to build production-ready applications with Rust and Next.js
        </p>
      </div>

      {/* Quick Links */}
      <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6 mb-12">
        <Link href="/docs/getting-started/quick-start">
          <Card className="border-2 hover:border-primary/50 transition-all h-full cursor-pointer">
            <CardHeader>
              <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center mb-2">
                <BookOpen className="h-5 w-5 text-primary" />
              </div>
              <CardTitle className="flex items-center justify-between">
                Quick Start
                <ArrowRight className="h-4 w-4" />
              </CardTitle>
              <CardDescription>Get up and running in under 10 minutes</CardDescription>
            </CardHeader>
          </Card>
        </Link>

        <Link href="/docs/backend/architecture">
          <Card className="border-2 hover:border-primary/50 transition-all h-full cursor-pointer">
            <CardHeader>
              <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center mb-2">
                <Server className="h-5 w-5 text-primary" />
              </div>
              <CardTitle className="flex items-center justify-between">
                Backend Docs
                <ArrowRight className="h-4 w-4" />
              </CardTitle>
              <CardDescription>Rust, Axum, and Domain-Driven Design</CardDescription>
            </CardHeader>
          </Card>
        </Link>

        <Link href="/docs/frontend/architecture">
          <Card className="border-2 hover:border-primary/50 transition-all h-full cursor-pointer">
            <CardHeader>
              <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center mb-2">
                <Code2 className="h-5 w-5 text-primary" />
              </div>
              <CardTitle className="flex items-center justify-between">
                Frontend Docs
                <ArrowRight className="h-4 w-4" />
              </CardTitle>
              <CardDescription>Next.js 15, React 19, and TypeScript</CardDescription>
            </CardHeader>
          </Card>
        </Link>
      </div>

      {/* All Sections */}
      <div className="space-y-8">
        <h2 className="text-3xl font-bold">Browse Documentation</h2>

        <div className="grid md:grid-cols-2 gap-6">
          {docsNav.map((section) => {
            const Icon = sectionIcons[section.slug] || FileCode
            return (
              <Card key={section.slug}>
                <CardHeader>
                  <div className="flex items-center gap-3 mb-2">
                    <div className="h-8 w-8 rounded-lg bg-primary/10 flex items-center justify-center">
                      <Icon className="h-4 w-4 text-primary" />
                    </div>
                    <CardTitle>{section.title}</CardTitle>
                  </div>
                </CardHeader>
                <CardContent>
                  <ul className="space-y-2">
                    {section.items?.slice(0, 5).map((item) => (
                      <li key={item.slug}>
                        <Link
                          href={`/docs/${section.slug}/${item.slug}`}
                          className="text-sm text-muted-foreground hover:text-foreground transition-colors flex items-center gap-2"
                        >
                          <ArrowRight className="h-3 w-3" />
                          {item.title}
                        </Link>
                      </li>
                    ))}
                    {section.items && section.items.length > 5 && (
                      <li className="text-sm text-muted-foreground">
                        + {section.items.length - 5} more...
                      </li>
                    )}
                  </ul>
                </CardContent>
              </Card>
            )
          })}
        </div>
      </div>
    </div>
  )
}
