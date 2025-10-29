/**
 * Chat Container Component
 *
 * Main chat interface with session management, message display, and streaming
 */

'use client';

import { useState, useEffect, useRef } from 'react';
import { Loader2, AlertCircle, MessageSquare, LogIn } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { useAuth } from '@/contexts/auth-context';
import { useChatApi } from '@/hooks/use-chat-api';
import { useSseStream } from '@/hooks/use-sse-stream';
import { SessionSidebar } from './session-sidebar';
import { Message } from './message';
import { MessageInput } from './message-input';
import { RateLimitIndicator } from './rate-limit-indicator';
import { ModelSelector } from './model-selector';
import { AVAILABLE_MODELS, MODEL_GROUPS, DEFAULT_MODEL_ID } from '@/config/models';
import type { ChatSession, ChatMessage, RateLimitError } from '@/types/chat';

export function ChatContainer() {
  const router = useRouter();
  const { isAuthenticated, isLoading: authLoading } = useAuth();
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [currentSession, setCurrentSession] = useState<ChatSession | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [streamingMessage, setStreamingMessage] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [rateLimitError, setRateLimitError] = useState<RateLimitError | null>(null);
  const [selectedModelId, setSelectedModelId] = useState<string>(DEFAULT_MODEL_ID);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const scrollAreaRef = useRef<HTMLDivElement>(null);

  const {
    createSession,
    listSessions,
    getSessionHistory,
    deleteSession,
    sendMessage,
    rateLimitInfo,
  } = useChatApi();

  const { parseStream, isStreaming } = useSseStream();

  // Load sessions only after authentication is confirmed
  useEffect(() => {
    if (isAuthenticated && !authLoading) {
      loadSessions();
    }
  }, [isAuthenticated, authLoading]);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingMessage]);

  const loadSessions = async () => {
    try {
      const response = await listSessions();
      setSessions(response.sessions);
      // Auto-select first session if none selected
      if (!currentSession && response.sessions.length > 0) {
        // Try to select first session, but don't fail if it doesn't exist
        try {
          await handleSelectSession(response.sessions[0].id);
        } catch (selectError: any) {
          // If auto-select fails, just clear it - user can create new session
          console.warn('Failed to auto-select session:', selectError);
          setCurrentSession(null);
          setMessages([]);
        }
      }
    } catch (err: any) {
      // Don't show error if it's 401 (user will see auth prompt)
      if (err.status !== 401) {
        setError('Failed to load chat sessions');
      }
      console.error(err);
    }
  };

  const handleCreateSession = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await createSession({ title: 'New Chat' });
      await loadSessions();
      handleSelectSession(response.session_id);
    } catch (err) {
      setError('Failed to create chat session');
      console.error(err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSelectSession = async (sessionId: string) => {
    setIsLoading(true);
    setError(null);
    setRateLimitError(null);
    try {
      const response = await getSessionHistory(sessionId);
      setCurrentSession(response.session);
      setMessages(response.messages);
      setStreamingMessage('');
    } catch (err: any) {
      console.error('Failed to load session history:', err);

      // Handle 404: Session was deleted or doesn't exist
      if (err.status === 404) {
        setError('This chat session no longer exists. Please create a new one.');
        // Clear the invalid session from state
        setCurrentSession(null);
        setMessages([]);
        setStreamingMessage('');
        // Remove from sessions list
        setSessions((prevSessions) => prevSessions.filter(s => s.id !== sessionId));
      } else {
        // Other errors
        setError(err.message || 'Failed to load session history');
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleDeleteSession = async (sessionId: string) => {
    try {
      await deleteSession(sessionId);
      await loadSessions();
      // If deleted current session, clear it
      if (currentSession?.id === sessionId) {
        setCurrentSession(null);
        setMessages([]);
        setStreamingMessage('');
      }
    } catch (err) {
      setError('Failed to delete session');
      console.error(err);
    }
  };

  const handleSendMessage = async (content: string) => {
    if (!currentSession) {
      setError('Please select or create a chat session first');
      return;
    }

    setError(null);
    setRateLimitError(null);
    setStreamingMessage('');

    // Add user message immediately
    const userMessage: ChatMessage = {
      id: crypto.randomUUID(),
      session_id: currentSession.id,
      role: 'user',
      content,
      created_at: new Date().toISOString(),
    };
    setMessages((prev) => [...prev, userMessage]);

    try {
      const stream = await sendMessage(currentSession.id, {
        content,
        model_id: selectedModelId !== DEFAULT_MODEL_ID ? selectedModelId : undefined
      });

      // Create placeholder for assistant message
      const assistantMessageId = crypto.randomUUID();
      let fullContent = '';

      await parseStream(
        stream,
        (chunk) => {
          if (!chunk.done) {
            fullContent += chunk.content;
            setStreamingMessage(fullContent);
          }
        },
        () => {
          // Stream complete - convert to permanent message
          const assistantMessage: ChatMessage = {
            id: assistantMessageId,
            session_id: currentSession.id,
            role: 'assistant',
            content: fullContent,
            created_at: new Date().toISOString(),
          };
          setMessages((prev) => [...prev, assistantMessage]);
          setStreamingMessage('');
        },
        (err) => {
          setError('Failed to receive message stream');
          console.error(err);
          setStreamingMessage('');
        }
      );
    } catch (err: any) {
      // Handle rate limit errors
      if (err.type === 'rate_limit') {
        setRateLimitError(err as RateLimitError);
        setError(null);
      } else {
        setError(err.message || 'Failed to send message');
      }
      console.error(err);
    }
  };

  // Show loading while checking authentication
  if (authLoading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    );
  }

  // Show login prompt if not authenticated
  if (!isAuthenticated) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="text-center space-y-4 max-w-md p-8">
          <div className="flex justify-center">
            <div className="rounded-full bg-primary/10 p-4">
              <LogIn className="h-12 w-12 text-primary" />
            </div>
          </div>
          <h2 className="text-2xl font-semibold">Authentication Required</h2>
          <p className="text-muted-foreground">
            You need to be logged in to access the chat feature. Please login or create an account to continue.
          </p>
          <div className="flex gap-3 justify-center pt-4">
            <Button onClick={() => router.push('/login')} size="lg">
              Login
            </Button>
            <Button onClick={() => router.push('/register')} variant="outline" size="lg">
              Create Account
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen">
      <SessionSidebar
        sessions={sessions}
        currentSessionId={currentSession?.id || null}
        onSelectSession={handleSelectSession}
        onCreateSession={handleCreateSession}
        onDeleteSession={handleDeleteSession}
        isLoading={isLoading}
      />

      <div className="flex-1 flex flex-col">
        <RateLimitIndicator rateLimitInfo={rateLimitInfo} />

        {error && (
          <div className="p-4">
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          </div>
        )}

        {rateLimitError && (
          <div className="p-4">
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                <div className="font-semibold">Rate Limit Exceeded</div>
                <div className="mt-1">{rateLimitError.message}</div>
                <div className="text-xs mt-2">
                  Limit: {rateLimitError.current}/{rateLimitError.limit} (
                  {rateLimitError.limit_type === 'per_minute' ? 'per minute' : 'daily'})
                </div>
              </AlertDescription>
            </Alert>
          </div>
        )}

        <ScrollArea className="flex-1" ref={scrollAreaRef}>
          <div className="max-w-4xl mx-auto p-4 space-y-4">
            {!currentSession && !isLoading && (
              <div className="text-center text-muted-foreground py-12">
                <MessageSquare className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p className="text-lg font-medium">No chat session selected</p>
                <p className="text-sm">Create a new chat to get started</p>
              </div>
            )}

            {isLoading && messages.length === 0 && (
              <div className="flex justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
              </div>
            )}

            {messages.map((message) => (
              <Message key={message.id} message={message} />
            ))}

            {streamingMessage && (
              <Message
                message={{
                  id: 'streaming',
                  session_id: currentSession?.id || '',
                  role: 'assistant',
                  content: streamingMessage,
                  created_at: new Date().toISOString(),
                }}
                isStreaming={true}
              />
            )}

            <div ref={messagesEndRef} />
          </div>
        </ScrollArea>

        <div className="border-t p-4 space-y-3">
          <ModelSelector
            models={AVAILABLE_MODELS}
            modelGroups={MODEL_GROUPS}
            selectedModelId={selectedModelId}
            onSelectModel={setSelectedModelId}
            disabled={!currentSession || isStreaming || isLoading}
          />
          <MessageInput
            onSend={handleSendMessage}
            disabled={!currentSession || isStreaming || isLoading}
            placeholder={
              currentSession
                ? 'Type your message...'
                : 'Create or select a chat session to start'
            }
          />
        </div>
      </div>
    </div>
  );
}
