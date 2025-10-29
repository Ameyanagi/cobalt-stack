/**
 * Model Selector Component
 *
 * Dropdown to select LLM model for chat conversations
 */

'use client';

import { Check, ChevronsUpDown, Sparkles } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Badge } from '@/components/ui/badge';
import type { LlmModel, ModelGroup } from '@/types/chat';

interface ModelSelectorProps {
  models: LlmModel[];
  modelGroups: ModelGroup[];
  selectedModelId: string;
  onSelectModel: (modelId: string) => void;
  disabled?: boolean;
}

export function ModelSelector({
  models,
  modelGroups,
  selectedModelId,
  onSelectModel,
  disabled = false,
}: ModelSelectorProps) {
  const [open, setOpen] = React.useState(false);

  const selectedModel = models.find((m) => m.id === selectedModelId);

  // Group models by their groups
  const groupedModels = React.useMemo(() => {
    const groups: Record<string, LlmModel[]> = {};

    modelGroups.forEach((group) => {
      groups[group.name] = models.filter((m) => group.models.includes(m.id));
    });

    return groups;
  }, [models, modelGroups]);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-full justify-between"
          disabled={disabled}
        >
          <div className="flex items-center gap-2">
            <Sparkles className="h-4 w-4" />
            <span className="truncate">
              {selectedModel ? selectedModel.name : 'Select model...'}
            </span>
          </div>
          <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[400px] p-0" align="start">
        <Command>
          <CommandInput placeholder="Search models..." />
          <CommandList>
            <CommandEmpty>No model found.</CommandEmpty>

            {Object.entries(groupedModels).map(([groupName, groupModels]) => (
              <CommandGroup key={groupName} heading={groupName}>
                {groupModels.map((model) => (
                  <CommandItem
                    key={model.id}
                    value={model.id}
                    onSelect={(currentValue) => {
                      onSelectModel(currentValue);
                      setOpen(false);
                    }}
                  >
                    <Check
                      className={cn(
                        'mr-2 h-4 w-4',
                        selectedModelId === model.id ? 'opacity-100' : 'opacity-0'
                      )}
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{model.name}</span>
                        <Badge variant="secondary" className="text-xs">
                          {model.provider}
                        </Badge>
                      </div>
                      <p className="text-xs text-muted-foreground truncate">
                        {model.description}
                      </p>
                      <div className="flex gap-2 mt-1">
                        {model.tags.slice(0, 3).map((tag) => (
                          <Badge key={tag} variant="outline" className="text-xs">
                            {tag}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  </CommandItem>
                ))}
              </CommandGroup>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

// Add React import
import * as React from 'react';
