'use client'

import { useEffect, useRef, useState } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import rehypeHighlight from 'rehype-highlight'
import rehypeRaw from 'rehype-raw'
import mermaid from 'mermaid'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Check, Copy } from 'lucide-react'
import { Separator } from '@/components/ui/separator'
import '@/styles/markdown.css'
import '@/styles/highlight.css'

interface MarkdownViewerProps {
  content: string
}

function CopyButton({ code }: { code: string }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(code)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Button
      onClick={handleCopy}
      variant={copied ? "default" : "secondary"}
      size="sm"
      className="absolute top-2 right-2 h-8 px-3 gap-1.5 z-10"
      aria-label="Copy code"
    >
      {copied ? (
        <>
          <Check className="h-3.5 w-3.5" />
          <span className="text-xs">Copied!</span>
        </>
      ) : (
        <>
          <Copy className="h-3.5 w-3.5" />
          <span className="text-xs hidden sm:inline">Copy</span>
        </>
      )}
    </Button>
  )
}

export function MarkdownViewer({ content }: MarkdownViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    // Initialize Mermaid
    mermaid.initialize({
      startOnLoad: true,
      theme: 'default',
      securityLevel: 'loose',
      fontFamily: 'var(--font-geist-sans)',
    })

    // Render Mermaid diagrams
    if (containerRef.current) {
      const mermaidElements = containerRef.current.querySelectorAll('.language-mermaid')
      mermaidElements.forEach((element, index) => {
        const code = element.textContent || ''
        const id = `mermaid-${index}-${Date.now()}`
        const parent = element.parentElement

        if (parent) {
          const div = document.createElement('div')
          div.id = id
          div.className = 'mermaid-diagram'
          parent.replaceWith(div)

          mermaid.render(id, code).then(({ svg }) => {
            div.innerHTML = svg
          }).catch((error) => {
            console.error('Mermaid rendering error:', error)
            div.innerHTML = `<pre class="mermaid-error">Failed to render diagram</pre>`
          })
        }
      })
    }
  }, [content])

  return (
    <div ref={containerRef} className="markdown-content">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight, rehypeRaw]}
        components={{
          // Custom rendering for code blocks
          code(props) {
            const { node, className, children, ...rest } = props
            const inline = !('inline' in props) ? false : (props as any).inline
            const match = /language-(\w+)/.exec(className || '')
            const language = match ? match[1] : ''
            const codeString = String(children).replace(/\n$/, '')

            if (!inline && language === 'mermaid') {
              // Mermaid diagrams are handled by useEffect
              return (
                <pre className={className}>
                  <code className={className} {...rest}>
                    {children}
                  </code>
                </pre>
              )
            }

            if (!inline) {
              return (
                <Card className="relative my-6 overflow-hidden">
                  {language && (
                    <div className="flex items-center justify-between px-4 py-2 bg-muted/50 border-b">
                      <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                        {language}
                      </span>
                    </div>
                  )}
                  <CopyButton code={codeString} />
                  <div className="overflow-x-auto">
                    <pre className={className}>
                      <code className={className} {...rest}>
                        {children}
                      </code>
                    </pre>
                  </div>
                </Card>
              )
            }

            return (
              <code className={className} {...rest}>
                {children}
              </code>
            )
          },
          // Add anchor links to headings
          h1: ({ children, ...props }) => {
            const id = String(children).toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '')
            return <h1 id={id} {...props}><a href={`#${id}`} className="heading-anchor">{children}</a></h1>
          },
          h2: ({ children, ...props }) => {
            const id = String(children).toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '')
            return <h2 id={id} {...props}><a href={`#${id}`} className="heading-anchor">{children}</a></h2>
          },
          h3: ({ children, ...props }) => {
            const id = String(children).toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '')
            return <h3 id={id} {...props}><a href={`#${id}`} className="heading-anchor">{children}</a></h3>
          },
          // Style tables
          table: ({ children, ...props }) => (
            <Card className="my-6 overflow-hidden">
              <div className="overflow-x-auto">
                <table {...props}>{children}</table>
              </div>
            </Card>
          ),
          // Style links
          a: ({ href, children, ...props }) => {
            const isExternal = href?.startsWith('http')
            return (
              <a
                href={href}
                {...props}
                className="text-primary underline underline-offset-4 hover:text-primary/80 transition-colors"
                {...(isExternal && { target: '_blank', rel: 'noopener noreferrer' })}
              >
                {children}
                {isExternal && <span className="ml-1 inline-block text-xs">↗</span>}
              </a>
            )
          },
          // Style blockquotes
          blockquote: ({ children, ...props }) => (
            <Card className="my-6 border-l-4 border-l-primary bg-muted/30">
              <div className="p-4 italic">
                {children}
              </div>
            </Card>
          ),
          // Style horizontal rules
          hr: () => <Separator className="my-8" />,
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
