// Documentation navigation structure
export interface DocSection {
  title: string
  slug: string
  items?: DocPage[]
}

export interface DocPage {
  title: string
  slug: string
  file: string
}

export const docsNav: DocSection[] = [
  {
    title: 'Getting Started',
    slug: 'getting-started',
    items: [
      { title: 'Quick Start', slug: 'quick-start', file: '/docs/getting-started/quick-start.md' },
      { title: 'Installation', slug: 'installation', file: '/docs/getting-started/installation.md' },
      { title: 'Project Structure', slug: 'project-structure', file: '/docs/getting-started/project-structure.md' },
    ],
  },
  {
    title: 'Backend',
    slug: 'backend',
    items: [
      { title: 'Overview', slug: 'overview', file: '/docs/backend/README.md' },
      { title: 'Architecture', slug: 'architecture', file: '/docs/backend/architecture.md' },
      { title: 'API Handlers', slug: 'api-handlers', file: '/docs/backend/api-handlers.md' },
      { title: 'Services', slug: 'services', file: '/docs/backend/services.md' },
      { title: 'Models', slug: 'models', file: '/docs/backend/models.md' },
      { title: 'Database', slug: 'database', file: '/docs/backend/database.md' },
      { title: 'Testing', slug: 'testing', file: '/docs/backend/testing.md' },
      { title: 'Rust Doc Guide', slug: 'rust-doc-guide', file: '/docs/backend/rust-doc-guide.md' },
    ],
  },
  {
    title: 'Frontend',
    slug: 'frontend',
    items: [
      { title: 'Overview', slug: 'overview', file: '/docs/frontend/README.md' },
      { title: 'Architecture', slug: 'architecture', file: '/docs/frontend/architecture.md' },
      { title: 'Components', slug: 'components', file: '/docs/frontend/components.md' },
      { title: 'State Management', slug: 'state-management', file: '/docs/frontend/state-management.md' },
      { title: 'API Client', slug: 'api-client', file: '/docs/frontend/api-client.md' },
      { title: 'Themes', slug: 'themes', file: '/docs/frontend/themes.md' },
      { title: 'Testing', slug: 'testing', file: '/docs/frontend/testing.md' },
    ],
  },
  {
    title: 'Guides',
    slug: 'guides',
    items: [
      { title: 'Authentication', slug: 'authentication', file: '/docs/guides/authentication.md' },
      { title: 'Email Verification', slug: 'email-verification', file: '/docs/guides/email-verification.md' },
      { title: 'Admin Dashboard', slug: 'admin-dashboard', file: '/docs/guides/admin-dashboard.md' },
      { title: 'Themes', slug: 'themes-guide', file: '/docs/guides/themes.md' },
      { title: 'Database', slug: 'database-guide', file: '/docs/guides/database.md' },
      { title: 'API Client', slug: 'api-client-guide', file: '/docs/guides/api-client.md' },
      { title: 'Testing', slug: 'testing-guide', file: '/docs/guides/testing.md' },
    ],
  },
  {
    title: 'Architecture',
    slug: 'architecture',
    items: [
      { title: 'Overview', slug: 'overview', file: '/docs/architecture/overview.md' },
      { title: 'Backend Architecture', slug: 'backend-arch', file: '/docs/architecture/backend.md' },
      { title: 'Frontend Architecture', slug: 'frontend-arch', file: '/docs/architecture/frontend.md' },
    ],
  },
  {
    title: 'API Reference',
    slug: 'api',
    items: [
      { title: 'Overview', slug: 'overview', file: '/docs/api/README.md' },
      { title: 'Reference', slug: 'reference', file: '/docs/api/reference.md' },
      { title: 'Authentication', slug: 'authentication', file: '/docs/api/authentication.md' },
      { title: 'Users', slug: 'users', file: '/docs/api/users.md' },
      { title: 'Admin', slug: 'admin', file: '/docs/api/admin.md' },
    ],
  },
  {
    title: 'Deployment',
    slug: 'deployment',
    items: [
      { title: 'Overview', slug: 'overview', file: '/docs/deployment/README.md' },
      { title: 'Docker', slug: 'docker', file: '/docs/deployment/docker.md' },
      { title: 'Production', slug: 'production', file: '/docs/deployment/production.md' },
      { title: 'Environment Variables', slug: 'environment-variables', file: '/docs/deployment/environment-variables.md' },
      { title: 'Monitoring', slug: 'monitoring', file: '/docs/deployment/monitoring.md' },
    ],
  },
  {
    title: 'Contributing',
    slug: 'contributing',
    items: [
      { title: 'Overview', slug: 'overview', file: '/CONTRIBUTING.md' },
      { title: 'Code Style', slug: 'code-style', file: '/docs/contributing/code-style.md' },
      { title: 'Pull Requests', slug: 'pull-requests', file: '/docs/contributing/pull-requests.md' },
      { title: 'Testing Requirements', slug: 'testing-requirements', file: '/docs/contributing/testing-requirements.md' },
    ],
  },
  {
    title: 'Troubleshooting',
    slug: 'troubleshooting',
    items: [
      { title: 'Overview', slug: 'overview', file: '/docs/troubleshooting/README.md' },
    ],
  },
]

// Flatten all pages for easy lookup
export const allDocs = docsNav.flatMap(section =>
  section.items?.map(item => ({
    ...item,
    section: section.title,
    sectionSlug: section.slug,
  })) || []
)

// Get doc by section and slug
export function getDocBySlug(sectionSlug: string, pageSlug: string) {
  return allDocs.find(
    doc => doc.sectionSlug === sectionSlug && doc.slug === pageSlug
  )
}

// Get all docs for a section
export function getDocsBySection(sectionSlug: string) {
  return allDocs.filter(doc => doc.sectionSlug === sectionSlug)
}
