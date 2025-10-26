import { notFound } from 'next/navigation'
import { readFile } from 'fs/promises'
import { join } from 'path'
import { MarkdownViewer } from '@/components/docs/markdown-viewer'
import { getDocBySlug, allDocs } from '@/lib/docs-nav'
import { Button } from '@/components/ui/button'
import Link from 'next/link'
import { ChevronLeft, ChevronRight, Github, Edit } from 'lucide-react'

interface DocPageProps {
  params: Promise<{
    section: string
    slug: string
  }>
}

export async function generateStaticParams() {
  return allDocs.map((doc) => ({
    section: doc.sectionSlug,
    slug: doc.slug,
  }))
}

export async function generateMetadata({ params }: DocPageProps) {
  const { section, slug } = await params
  const doc = getDocBySlug(section, slug)

  if (!doc) {
    return {
      title: 'Page Not Found',
    }
  }

  return {
    title: `${doc.title} - ${doc.section} | Cobalt Stack Documentation`,
    description: `Documentation for ${doc.title} in Cobalt Stack`,
  }
}

async function getDocContent(filePath: string): Promise<string | null> {
  try {
    const fullPath = join(process.cwd(), '..', filePath)
    const content = await readFile(fullPath, 'utf-8')
    return content
  } catch (error) {
    console.error(`Failed to read ${filePath}:`, error)
    return null
  }
}

function getAdjacentDocs(currentSection: string, currentSlug: string) {
  const currentIndex = allDocs.findIndex(
    doc => doc.sectionSlug === currentSection && doc.slug === currentSlug
  )

  const previousDoc = currentIndex > 0 ? allDocs[currentIndex - 1] : null
  const nextDoc = currentIndex < allDocs.length - 1 ? allDocs[currentIndex + 1] : null

  return { previousDoc, nextDoc }
}

export default async function DocPage({ params }: DocPageProps) {
  const { section, slug } = await params
  const doc = getDocBySlug(section, slug)

  if (!doc) {
    notFound()
  }

  const content = await getDocContent(doc.file)

  if (!content) {
    return (
      <div className="container max-w-4xl mx-auto px-4 py-12">
        <div className="text-center space-y-4">
          <h1 className="text-4xl font-bold">Documentation Not Found</h1>
          <p className="text-muted-foreground">
            The documentation file for this page could not be loaded.
          </p>
          <Link href="/docs">
            <Button>Back to Docs</Button>
          </Link>
        </div>
      </div>
    )
  }

  const { previousDoc, nextDoc } = getAdjacentDocs(section, slug)

  return (
    <div className="container max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-6 sm:py-8">
      {/* Breadcrumb */}
      <div className="flex items-center gap-2 text-sm text-muted-foreground mb-6 overflow-x-auto">
        <Link href="/docs" className="hover:text-foreground transition-colors whitespace-nowrap">
          Docs
        </Link>
        <span className="hidden sm:inline">/</span>
        <span className="hidden sm:inline text-foreground font-medium whitespace-nowrap">{doc.section}</span>
        <span className="hidden sm:inline">/</span>
        <span className="text-foreground font-medium truncate max-w-[200px] sm:max-w-none">{doc.title}</span>
      </div>

      {/* Edit on GitHub Link */}
      <div className="flex items-center justify-between mb-6">
        <div></div>
        <a
          href={`https://github.com/Ameyanagi/cobalt-stack/edit/main${doc.file}`}
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm text-muted-foreground hover:text-foreground transition-colors flex items-center gap-2"
        >
          <Edit className="h-3.5 w-3.5" />
          Edit on GitHub
        </a>
      </div>

      {/* Content */}
      <article className="prose prose-lg max-w-none">
        <MarkdownViewer content={content} />
      </article>

      {/* Navigation */}
      <div className="flex items-center justify-between pt-12 mt-12 border-t gap-2">
        {previousDoc ? (
          <Link
            href={`/docs/${previousDoc.sectionSlug}/${previousDoc.slug}`}
            className="flex items-center gap-2 group"
          >
            {/* Mobile: Icon only */}
            <Button variant="outline" size="sm" className="gap-1 sm:hidden">
              <ChevronLeft className="h-4 w-4" />
            </Button>
            {/* Tablet/Desktop: Full button */}
            <Button variant="outline" size="lg" className="gap-2 hidden sm:flex">
              <ChevronLeft className="h-4 w-4 group-hover:-translate-x-1 transition-transform" />
              <div className="text-left">
                <div className="text-xs text-muted-foreground">Previous</div>
                <div className="font-medium truncate max-w-[150px] md:max-w-none">{previousDoc.title}</div>
              </div>
            </Button>
          </Link>
        ) : (
          <div></div>
        )}

        {nextDoc ? (
          <Link
            href={`/docs/${nextDoc.sectionSlug}/${nextDoc.slug}`}
            className="flex items-center gap-2 group"
          >
            {/* Mobile: Icon only */}
            <Button variant="outline" size="sm" className="gap-1 sm:hidden">
              <ChevronRight className="h-4 w-4" />
            </Button>
            {/* Tablet/Desktop: Full button */}
            <Button variant="outline" size="lg" className="gap-2 hidden sm:flex">
              <div className="text-right">
                <div className="text-xs text-muted-foreground">Next</div>
                <div className="font-medium truncate max-w-[150px] md:max-w-none">{nextDoc.title}</div>
              </div>
              <ChevronRight className="h-4 w-4 group-hover:translate-x-1 transition-transform" />
            </Button>
          </Link>
        ) : (
          <div></div>
        )}
      </div>

      {/* Footer */}
      <div className="text-center pt-12 mt-12 border-t text-sm text-muted-foreground">
        <p>
          Found an issue with this page?{' '}
          <a
            href={`https://github.com/Ameyanagi/cobalt-stack/issues/new?title=Docs: ${doc.title}&body=Issue in ${doc.file}`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-primary hover:underline"
          >
            Report it on GitHub
          </a>
        </p>
      </div>
    </div>
  )
}
