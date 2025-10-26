'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import Link from 'next/link'
import { ExternalLink, BookOpen, Code2, ArrowLeft } from 'lucide-react'

export default function ApiDocsPage() {
  const router = useRouter()

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      {/* Header */}
      <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 sticky top-0 z-50">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <Link href="/" className="flex items-center gap-2">
              <ArrowLeft className="h-5 w-5" />
              <span className="font-semibold">Back to Home</span>
            </Link>
            <div className="flex items-center gap-3">
              <Link href="/docs">
                <Button variant="ghost" size="sm" className="gap-2">
                  <BookOpen className="h-4 w-4" />
                  Documentation
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </header>

      {/* Content */}
      <div className="container max-w-4xl mx-auto px-4 py-16">
        <div className="space-y-8">
          {/* Hero Section */}
          <div className="text-center space-y-4">
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border bg-primary/10 text-sm">
              <Code2 className="h-4 w-4" />
              <span>Rust API Documentation</span>
            </div>
            <h1 className="text-4xl md:text-5xl font-bold">
              Backend API Reference
            </h1>
            <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
              Complete Rust documentation for all backend modules, types, and functions
            </p>
          </div>

          {/* Main Card */}
          <Card className="border-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Code2 className="h-5 w-5 text-primary" />
                Rustdoc API Documentation
              </CardTitle>
              <CardDescription>
                Comprehensive auto-generated documentation from Rust doc comments following RFC 1574 standards
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="space-y-4">
                <div>
                  <h3 className="font-semibold mb-2">What's Included:</h3>
                  <ul className="space-y-2 text-sm text-muted-foreground">
                    <li className="flex items-start gap-2">
                      <span className="text-primary mt-0.5">•</span>
                      <span>Complete module hierarchy and organization</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-primary mt-0.5">•</span>
                      <span>All public structs, enums, and traits with detailed descriptions</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-primary mt-0.5">•</span>
                      <span>Function signatures with parameter and return type documentation</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-primary mt-0.5">•</span>
                      <span>Code examples demonstrating real-world usage</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-primary mt-0.5">•</span>
                      <span>Security notes and best practices</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-primary mt-0.5">•</span>
                      <span>Error handling patterns and types</span>
                    </li>
                  </ul>
                </div>

                <div className="pt-4 border-t">
                  <a
                    href="/rustdoc/cobalt_stack_backend/index.html"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-block"
                  >
                    <Button size="lg" className="gap-2">
                      Open API Documentation
                      <ExternalLink className="h-4 w-4" />
                    </Button>
                  </a>
                </div>
              </div>

              <div className="pt-4 border-t">
                <h3 className="font-semibold mb-3">Quick Links:</h3>
                <div className="grid gap-2">
                  <a
                    href="/rustdoc/cobalt_stack_backend/handlers/index.html"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-2 text-sm text-primary hover:underline"
                  >
                    <ExternalLink className="h-3 w-3" />
                    <span>API Handlers</span>
                  </a>
                  <a
                    href="/rustdoc/cobalt_stack_backend/services/index.html"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-2 text-sm text-primary hover:underline"
                  >
                    <ExternalLink className="h-3 w-3" />
                    <span>Services</span>
                  </a>
                  <a
                    href="/rustdoc/cobalt_stack_backend/models/index.html"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-2 text-sm text-primary hover:underline"
                  >
                    <ExternalLink className="h-3 w-3" />
                    <span>Data Models</span>
                  </a>
                  <a
                    href="/rustdoc/cobalt_stack_backend/middleware/index.html"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-2 text-sm text-primary hover:underline"
                  >
                    <ExternalLink className="h-3 w-3" />
                    <span>Middleware</span>
                  </a>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Additional Resources */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <BookOpen className="h-5 w-5 text-primary" />
                Additional Resources
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <Link href="/docs" className="block">
                <Button variant="outline" className="w-full justify-start gap-2">
                  <BookOpen className="h-4 w-4" />
                  User Documentation & Guides
                </Button>
              </Link>
              <a
                href="https://github.com/Ameyanagi/cobalt-stack"
                target="_blank"
                rel="noopener noreferrer"
                className="block"
              >
                <Button variant="outline" className="w-full justify-start gap-2">
                  <ExternalLink className="h-4 w-4" />
                  GitHub Repository
                </Button>
              </a>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
