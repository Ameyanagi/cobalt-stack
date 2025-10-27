/**
 * Chat API client hook
 *
 * Provides authenticated API calls to chat endpoints with rate limit handling
 */

import { useState } from 'react';
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
  const [rateLimitInfo, setRateLimitInfo] = useState<RateLimitInfo | null>(null);

  const getAuthHeaders = (): HeadersInit => {
    const token = localStorage.getItem('accessToken');
    return {
      'Content-Type': 'application/json',
      ...(token && { Authorization: `Bearer ${token}` }),
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
      const error: ApiError | RateLimitError = await response.json();
      if (response.status === 429) {
        throw { type: 'rate_limit', ...error } as RateLimitError & { type: 'rate_limit' };
      }
      throw new Error(error.message || 'API request failed');
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
    const response = await fetch(`${API_BASE_URL}/api/v1/chat/sessions/${sessionId}/history`, {
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
      const error: ApiError | RateLimitError = await response.json();
      if (response.status === 429) {
        throw { type: 'rate_limit', ...error } as RateLimitError & { type: 'rate_limit' };
      }
      throw new Error(error.message || 'Failed to send message');
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
