/**
 * Environment variable validation and type-safe access
 */

function getEnvVar(key: string, defaultValue?: string): string {
  const value = process.env[key]

  if (!value && !defaultValue) {
    throw new Error(`Missing required environment variable: ${key}`)
  }

  return value || defaultValue!
}

/**
 * Get the API URL - dynamically constructs based on current host
 * This allows the app to work when accessed from any IP (localhost, LAN IP, etc.)
 *
 * Frontend port: 2727
 * Backend port: 2750
 */
function getApiUrl(): string {
  // For client-side requests, construct URL from current host
  if (typeof window !== 'undefined') {
    const protocol = window.location.protocol
    const hostname = window.location.hostname
    const backendPort = '2750'

    return `${protocol}//${hostname}:${backendPort}`
  }

  // For server-side requests (SSR), use the configured URL
  return getEnvVar('NEXT_PUBLIC_API_URL', 'http://localhost:2750')
}

export const env = {
  get apiUrl() {
    return getApiUrl()
  },
} as const

// Validate environment on module load (development only)
if (process.env.NODE_ENV === 'development' && typeof window === 'undefined') {
  console.log('Environment configuration (SSR):', {
    apiUrl: getEnvVar('NEXT_PUBLIC_API_URL', 'http://localhost:2750'),
  })
}
