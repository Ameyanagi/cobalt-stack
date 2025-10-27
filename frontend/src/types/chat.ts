/**
 * Chat API types
 *
 * These types match the backend DTOs for type-safe API communication
 */

export interface ChatSession {
  id: string;
  user_id: string;
  title: string | null;
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  id: string;
  session_id: string;
  role: 'user' | 'assistant';
  content: string;
  created_at: string;
}

export interface CreateSessionRequest {
  title?: string;
}

export interface CreateSessionResponse {
  session_id: string;
  message: string;
}

export interface SendMessageRequest {
  content: string;
}

export interface GetHistoryResponse {
  session: ChatSession;
  messages: ChatMessage[];
}

export interface ListSessionsResponse {
  sessions: ChatSession[];
  total: number;
}

export interface DeleteSessionResponse {
  message: string;
}

export interface RateLimitInfo {
  limit_minute: number;
  remaining_minute: number;
  reset_minute: number;
  limit_daily: number;
  remaining_daily: number;
  reset_daily: number;
}

export interface RateLimitError {
  error: string;
  limit_type: 'per_minute' | 'daily';
  limit: number;
  current: number;
  retry_after: number;
  message: string;
}
