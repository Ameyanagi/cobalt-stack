/**
 * Type-safe API client for Cobalt Stack backend
 */

import { env } from './env'
import type { paths, components } from '@/types/api'

/**
 * API response type helper
 */
export type ApiResponse<T> =
  | { success: true; data: T }
  | { success: false; error: string }

/**
 * Health check response type from OpenAPI schema
 */
export type HealthResponse = components['schemas']['HealthResponse']

/**
 * Base API client class
 */
class ApiClient {
  private baseUrl: string

  constructor(baseUrl: string = env.apiUrl) {
    this.baseUrl = baseUrl
  }

  /**
   * Make a type-safe GET request
   */
  private async get<T>(path: string): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        return {
          success: false,
          error: `HTTP ${response.status}: ${response.statusText}`,
        }
      }

      const data = await response.json()
      return { success: true, data }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }
    }
  }

  /**
   * Health check endpoint
   */
  healthCheck = async (): Promise<ApiResponse<HealthResponse>> => {
    return this.get<HealthResponse>('/health')
  }
}

/**
 * Singleton API client instance
 */
export const apiClient = new ApiClient()
