/**
 * Environment variable validation and type-safe access
 *
 * NEXT_PUBLIC_ prefixed variables are embedded at build time
 * and available in both server and client code
 */

export const env = {
  apiUrl: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:2750',
} as const

// Validate environment on module load (development only)
if (process.env.NODE_ENV === 'development' && typeof window === 'undefined') {
  console.log('Environment configuration (SSR):', {
    apiUrl: env.apiUrl,
  })
}
