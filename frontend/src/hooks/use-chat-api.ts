/**
 * Chat API client hook
 *
 * Provides authenticated API calls to chat endpoints with rate limit handling
 */

'use client';

import { useState } from 'react';
import { useAuth } from '@/contexts/auth-context';
import type {
  CreateSessionRequest,
  CreateSessionResponse,
  SendMessageRequest,
  GetHistoryResponse,
  ListSessionsResponse,
  DeleteSessionResponse,
  RateLimitInfo,
  RateLimitError,
} from '@/types/chat';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

interface ApiError {
  error: string;
  message: string;
}

export function useChatApi() {
  const { accessToken } = useAuth();
  const [rateLimitInfo, setRateLimitInfo] = useState<RateLimitInfo | null>(null);

  const getAuthHeaders = (): HeadersInit => {
    return {
      'Content-Type': 'application/json',
      ...(accessToken && { Authorization: `Bearer ${accessToken}` }),
    };
  };

  const extractRateLimitHeaders = (headers: Headers): RateLimitInfo => {
    return {
      limit_minute: Number.parseInt(headers.get('X-RateLimit-Limit-Minute') || '0'),
      remaining_minute: Number.parseInt(headers.get('X-RateLimit-Remaining-Minute') || '0'),
      reset_minute: Number.parseInt(headers.get('X-RateLimit-Reset-Minute') || '0'),
      limit_daily: Number.parseInt(headers.get('X-RateLimit-Limit-Daily') || '0'),
      remaining_daily: Number.parseInt(headers.get('X-RateLimit-Remaining-Daily') || '0'),
      reset_daily: Number.parseInt(headers.get('X-RateLimit-Reset-Daily') || '0'),
    };
  };

  const handleResponse = async <T>(response: Response): Promise<T> => {
    // Extract rate limit headers
    const rateLimit = extractRateLimitHeaders(response.headers);
    setRateLimitInfo(rateLimit);

    if (!response.ok) {
      // Try to parse error body, but handle empty responses
      let errorData: ApiError | RateLimitError | null = null;
      try {
        const text = await response.text();
        if (text) {
          errorData = JSON.parse(text);
        }
      } catch {
        // Empty or invalid JSON body
      }

      if (response.status === 429 && errorData) {
        throw { type: 'rate_limit', status: 429, ...errorData } as RateLimitError & { type: 'rate_limit'; status: number };
      }

      // Throw error with status code for better error handling
      const error: any = new Error(errorData?.message || `Request failed with status ${response.status}`);
      error.status = response.status;
      error.statusText = response.statusText;
      throw error;
    }

    return response.json();
  };

  const createSession = async (request: CreateSessionRequest): Promise<CreateSessionResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/chat/sessions`, {
      method: 'POST',
      headers: getAuthHeaders(),
      body: JSON.stringify(request),
    });
    return handleResponse<CreateSessionResponse>(response);
  };

  const listSessions = async (): Promise<ListSessionsResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/chat/sessions`, {
      method: 'GET',
      headers: getAuthHeaders(),
    });
    return handleResponse<ListSessionsResponse>(response);
  };

  const getSessionHistory = async (sessionId: string): Promise<GetHistoryResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/chat/sessions/${sessionId}/messages`, {
      method: 'GET',
      headers: getAuthHeaders(),
    });
    return handleResponse<GetHistoryResponse>(response);
  };

  const deleteSession = async (sessionId: string): Promise<DeleteSessionResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/chat/sessions/${sessionId}`, {
      method: 'DELETE',
      headers: getAuthHeaders(),
    });
    return handleResponse<DeleteSessionResponse>(response);
  };

  const sendMessage = async (
    sessionId: string,
    request: SendMessageRequest
  ): Promise<ReadableStream<Uint8Array>> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/chat/sessions/${sessionId}/messages`, {
      method: 'POST',
      headers: getAuthHeaders(),
      body: JSON.stringify(request),
    });

    // Extract rate limit headers
    const rateLimit = extractRateLimitHeaders(response.headers);
    setRateLimitInfo(rateLimit);

    if (!response.ok) {
      // Try to parse error body, but handle empty responses
      let errorData: ApiError | RateLimitError | null = null;
      try {
        const text = await response.text();
        if (text) {
          errorData = JSON.parse(text);
        }
      } catch {
        // Empty or invalid JSON body
      }

      if (response.status === 429 && errorData) {
        throw { type: 'rate_limit', status: 429, ...errorData } as RateLimitError & { type: 'rate_limit'; status: number };
      }

      const error: any = new Error(errorData?.message || `Failed to send message (${response.status})`);
      error.status = response.status;
      error.statusText = response.statusText;
      throw error;
    }

    if (!response.body) {
      throw new Error('Response body is null');
    }

    return response.body;
  };

  return {
    createSession,
    listSessions,
    getSessionHistory,
    deleteSession,
    sendMessage,
    rateLimitInfo,
  };
}
