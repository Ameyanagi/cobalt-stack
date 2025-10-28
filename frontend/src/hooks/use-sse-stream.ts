/**
 * SSE Stream parser hook
 *
 * Handles Server-Sent Events streaming from the backend LLM response
 */

import { useCallback, useState } from 'react';

export interface StreamChunk {
  content: string;
  done: boolean;
}

export function useSseStream() {
  const [isStreaming, setIsStreaming] = useState(false);

  const parseStream = useCallback(
    async (
      stream: ReadableStream<Uint8Array>,
      onChunk: (chunk: StreamChunk) => void,
      onComplete: () => void,
      onError: (error: Error) => void
    ) => {
      setIsStreaming(true);
      const reader = stream.getReader();
      const decoder = new TextDecoder();
      let buffer = ''; // Buffer for incomplete lines

      try {
        while (true) {
          const { done, value } = await reader.read();

          if (done) {
            onComplete();
            break;
          }

          // Decode and append to buffer
          buffer += decoder.decode(value, { stream: true });

          // Split by newlines, but keep the last incomplete line in buffer
          const lines = buffer.split('\n');
          buffer = lines.pop() || ''; // Keep the last incomplete line

          for (const line of lines) {
            if (!line.trim() || !line.startsWith('data: ')) {
              continue;
            }

            const data = line.slice(6); // Remove "data: " prefix

            if (data === '[DONE]') {
              onChunk({ content: '', done: true });
              continue;
            }

            try {
              const parsed = JSON.parse(data);
              if (parsed.content) {
                onChunk({ content: parsed.content, done: false });
              }
            } catch (parseError) {
              // Silently skip - this is likely a partial chunk that will be completed in the next read
              // Only log if it looks like it should be complete (starts and ends with braces)
              if (data.startsWith('{') && data.endsWith('}')) {
                console.warn('Failed to parse complete SSE chunk:', data, parseError);
              }
            }
          }
        }
      } catch (error) {
        onError(error instanceof Error ? error : new Error('Stream reading failed'));
      } finally {
        setIsStreaming(false);
        reader.releaseLock();
      }
    },
    []
  );

  return {
    parseStream,
    isStreaming,
  };
}
