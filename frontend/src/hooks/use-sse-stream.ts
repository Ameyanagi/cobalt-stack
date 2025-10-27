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

      try {
        while (true) {
          const { done, value } = await reader.read();

          if (done) {
            onComplete();
            break;
          }

          const chunk = decoder.decode(value, { stream: true });
          const lines = chunk.split('\n');

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
              console.warn('Failed to parse SSE chunk:', parseError);
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
