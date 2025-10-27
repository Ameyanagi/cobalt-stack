/**
 * Chat Container Component
 *
 * Main chat interface with session management, message display, and streaming
 */

'use client';

import { useState, useEffect, useRef } from 'react';
import { Loader2, AlertCircle, MessageSquare } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useChatApi } from '@/hooks/use-chat-api';
import { useSseStream } from '@/hooks/use-sse-stream';
import { SessionSidebar } from './session-sidebar';
import { Message } from './message';
import { MessageInput } from './message-input';
import { RateLimitIndicator } from './rate-limit-indicator';
import type { ChatSession, ChatMessage, RateLimitError } from '@/types/chat';

export function ChatContainer() {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [currentSession, setCurrentSession] = useState<ChatSession | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [streamingMessage, setStreamingMessage] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [rateLimitError, setRateLimitError] = useState<RateLimitError | null>(null);

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

  // Load sessions on mount
  useEffect(() => {
    loadSessions();
  }, []);

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
        handleSelectSession(response.sessions[0].id);
      }
    } catch (err) {
      setError('Failed to load chat sessions');
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
    } catch (err) {
      setError('Failed to load session history');
      console.error(err);
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
      const stream = await sendMessage(currentSession.id, { content });

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
  );
}
