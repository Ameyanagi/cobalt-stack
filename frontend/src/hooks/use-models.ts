/**
 * Models API hook
 *
 * Fetches and caches available LLM models from the backend
 */

'use client';

import { useState, useEffect } from 'react';
import type { LlmModel, ModelGroup, ListModelsResponse } from '@/types/chat';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

export interface UseModelsReturn {
  models: LlmModel[];
  groups: ModelGroup[];
  defaultModel: LlmModel | null;
  isLoading: boolean;
  error: Error | null;
}

export function useModels(): UseModelsReturn {
  const [models, setModels] = useState<LlmModel[]>([]);
  const [groups, setGroups] = useState<ModelGroup[]>([]);
  const [defaultModelId, setDefaultModelId] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const fetchModels = async () => {
      try {
        setIsLoading(true);
        setError(null);

        const response = await fetch(`${API_BASE_URL}/api/v1/chat/models`, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (!response.ok) {
          throw new Error(`Failed to fetch models: ${response.status} ${response.statusText}`);
        }

        const data: ListModelsResponse = await response.json();

        setModels(data.models);
        setGroups(data.groups);
        setDefaultModelId(data.default_model);
      } catch (err) {
        const error = err instanceof Error ? err : new Error('Failed to fetch models');
        setError(error);
        console.error('Error fetching models:', error);
      } finally {
        setIsLoading(false);
      }
    };

    fetchModels();
  }, []); // Empty dependency array - fetch once on mount

  // Find default model from loaded models
  const defaultModel = models.find(m => m.id === defaultModelId) || null;

  return {
    models,
    groups,
    defaultModel,
    isLoading,
    error,
  };
}
