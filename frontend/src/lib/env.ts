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

export const env = {
  apiUrl: getEnvVar('NEXT_PUBLIC_API_URL', 'http://localhost:2750'),
} as const

// Validate environment on module load (development only)
if (process.env.NODE_ENV === 'development') {
  console.log('Environment configuration:', {
    apiUrl: env.apiUrl,
  })
}
