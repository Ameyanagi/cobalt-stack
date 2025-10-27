'use client'

import { ChevronDown, ChevronRight, FileText } from 'lucide-react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { type DocSection, docsNav } from '@/lib/docs-nav'
import { cn } from '@/lib/utils'

// Sidebar content component (shared between desktop and mobile)
export function DocsSidebarContent({ onLinkClick }: { onLinkClick?: () => void }) {
  const pathname = usePathname()
  const [openSections, setOpenSections] = useState<Record<string, boolean>>(
    // Open all sections by default
    docsNav.reduce((acc, section) => ({ ...acc, [section.slug]: true }), {})
  )

  const toggleSection = (slug: string) => {
    setOpenSections((prev) => ({ ...prev, [slug]: !prev[slug] }))
  }

  return (
    <div className="p-4 space-y-2">
      {docsNav.map((section) => (
        <div key={section.slug} className="space-y-1">
          <Button
            variant="ghost"
            className="w-full justify-start px-2 py-1.5 h-auto font-semibold"
            onClick={() => toggleSection(section.slug)}
          >
            {openSections[section.slug] ? (
              <ChevronDown className="h-4 w-4 mr-2" />
            ) : (
              <ChevronRight className="h-4 w-4 mr-2" />
            )}
            {section.title}
          </Button>

          {openSections[section.slug] && section.items && (
            <div className="ml-4 space-y-1">
              {section.items.map((item) => {
                const href = `/docs/${section.slug}/${item.slug}`
                const isActive = pathname === href

                return (
                  <Link
                    key={item.slug}
                    href={href}
                    onClick={onLinkClick}
                    className={cn(
                      'flex items-center gap-2 px-2 py-1.5 text-sm rounded-md transition-colors',
                      isActive
                        ? 'bg-primary text-primary-foreground'
                        : 'text-muted-foreground hover:text-foreground hover:bg-muted'
                    )}
                  >
                    <FileText className="h-3.5 w-3.5" />
                    {item.title}
                  </Link>
                )
              })}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}

// Desktop sidebar (persistent)
export function DocsSidebar() {
  return (
    <aside className="hidden lg:block w-64 border-r bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <ScrollArea className="h-[calc(100vh-4rem)]">
        <DocsSidebarContent />
      </ScrollArea>
    </aside>
  )
}
