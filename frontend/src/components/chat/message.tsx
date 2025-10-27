/**
 * Chat Message Component
 *
 * Displays individual chat messages with markdown rendering and syntax highlighting
 */

import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';
import { User, Bot } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { ChatMessage as ChatMessageType } from '@/types/chat';

interface MessageProps {
  message: ChatMessageType;
  isStreaming?: boolean;
}

export function Message({ message, isStreaming = false }: MessageProps) {
  const isUser = message.role === 'user';

  return (
    <div
      className={cn(
        'flex gap-3 p-4 rounded-lg',
        isUser ? 'bg-muted ml-12' : 'bg-background mr-12',
        isStreaming && 'animate-pulse'
      )}
    >
      <div
        className={cn(
          'flex h-8 w-8 shrink-0 items-center justify-center rounded-full',
          isUser ? 'bg-primary text-primary-foreground' : 'bg-accent text-accent-foreground'
        )}
      >
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
      </div>

      <div className="flex-1 space-y-2 overflow-hidden">
        <div className="font-semibold text-sm">{isUser ? 'You' : 'Assistant'}</div>
        <div className="prose prose-sm dark:prose-invert max-w-none">
          {isUser ? (
            <p className="whitespace-pre-wrap break-words">{message.content}</p>
          ) : (
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight, rehypeRaw]}
              components={{
                pre: ({ children }) => (
                  <pre className="bg-muted p-4 rounded-lg overflow-x-auto">{children}</pre>
                ),
                code: ({ inline, className, children, ...props }) => {
                  const match = /language-(\w+)/.exec(className || '');
                  return !inline ? (
                    <code className={className} {...props}>
                      {children}
                    </code>
                  ) : (
                    <code className="bg-muted px-1 py-0.5 rounded text-sm" {...props}>
                      {children}
                    </code>
                  );
                },
                a: ({ href, children }) => (
                  <a
                    href={href}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline"
                  >
                    {children}
                  </a>
                ),
              }}
            >
              {message.content}
            </ReactMarkdown>
          )}
        </div>
      </div>
    </div>
  );
}
