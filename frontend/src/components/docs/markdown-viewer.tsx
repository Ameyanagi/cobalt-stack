'use client'

import { useEffect, useRef } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import rehypeHighlight from 'rehype-highlight'
import rehypeRaw from 'rehype-raw'
import mermaid from 'mermaid'
import '@/styles/markdown.css'
import '@/styles/highlight.css'

interface MarkdownViewerProps {
  content: string
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
          code({ node, inline, className, children, ...props }) {
            const match = /language-(\w+)/.exec(className || '')
            const language = match ? match[1] : ''

            if (!inline && language === 'mermaid') {
              // Mermaid diagrams are handled by useEffect
              return (
                <pre className={className}>
                  <code className={className} {...props}>
                    {children}
                  </code>
                </pre>
              )
            }

            if (!inline) {
              return (
                <pre className={className}>
                  <code className={className} {...props}>
                    {children}
                  </code>
                </pre>
              )
            }

            return (
              <code className={className} {...props}>
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
            <div className="table-wrapper">
              <table {...props}>{children}</table>
            </div>
          ),
          // Style links
          a: ({ href, children, ...props }) => {
            const isExternal = href?.startsWith('http')
            return (
              <a
                href={href}
                {...props}
                {...(isExternal && { target: '_blank', rel: 'noopener noreferrer' })}
              >
                {children}
                {isExternal && <span className="external-link-icon">↗</span>}
              </a>
            )
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
