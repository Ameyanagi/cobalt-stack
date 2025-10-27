/**
 * Rate Limit Indicator Component
 *
 * Displays current rate limit quotas and usage to users
 */

import { Clock, Zap, AlertTriangle } from 'lucide-react';
import { Progress } from '@/components/ui/progress';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import type { RateLimitInfo } from '@/types/chat';

interface RateLimitIndicatorProps {
  rateLimitInfo: RateLimitInfo | null;
}

export function RateLimitIndicator({ rateLimitInfo }: RateLimitIndicatorProps) {
  if (!rateLimitInfo) {
    return null;
  }

  const minuteUsage = rateLimitInfo.limit_minute - rateLimitInfo.remaining_minute;
  const minutePercent = (minuteUsage / rateLimitInfo.limit_minute) * 100;

  const dailyUsage = rateLimitInfo.limit_daily - rateLimitInfo.remaining_daily;
  const dailyPercent = (dailyUsage / rateLimitInfo.limit_daily) * 100;

  const minuteWarning = minutePercent > 80;
  const dailyWarning = dailyPercent > 80;

  const formatResetTime = (timestamp: number) => {
    const now = Math.floor(Date.now() / 1000);
    const diff = timestamp - now;
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    return `${Math.floor(diff / 3600)}h`;
  };

  return (
    <div className="border-b bg-muted/50 px-4 py-2">
      <div className="flex gap-4 items-center max-w-4xl mx-auto text-xs">
        {/* Per-minute rate limit */}
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-2 flex-1">
                <Zap className={`h-3 w-3 ${minuteWarning ? 'text-warning' : 'text-primary'}`} />
                <div className="flex-1">
                  <div className="flex justify-between text-[10px] mb-1">
                    <span>Per Minute</span>
                    <span className={minuteWarning ? 'text-warning' : ''}>
                      {rateLimitInfo.remaining_minute}/{rateLimitInfo.limit_minute}
                    </span>
                  </div>
                  <Progress
                    value={minutePercent}
                    className={`h-1 ${minuteWarning ? '[&>div]:bg-warning' : ''}`}
                  />
                </div>
              </div>
            </TooltipTrigger>
            <TooltipContent>
              <p>
                {rateLimitInfo.remaining_minute} messages remaining this minute
              </p>
              <p className="text-muted-foreground">
                Resets in {formatResetTime(rateLimitInfo.reset_minute)}
              </p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>

        {/* Daily quota */}
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-2 flex-1">
                <Clock className={`h-3 w-3 ${dailyWarning ? 'text-destructive' : 'text-primary'}`} />
                <div className="flex-1">
                  <div className="flex justify-between text-[10px] mb-1">
                    <span>Daily Quota</span>
                    <span className={dailyWarning ? 'text-destructive' : ''}>
                      {rateLimitInfo.remaining_daily}/{rateLimitInfo.limit_daily}
                    </span>
                  </div>
                  <Progress
                    value={dailyPercent}
                    className={`h-1 ${dailyWarning ? '[&>div]:bg-destructive' : ''}`}
                  />
                </div>
              </div>
            </TooltipTrigger>
            <TooltipContent>
              <p>
                {rateLimitInfo.remaining_daily} messages remaining today
              </p>
              <p className="text-muted-foreground">
                Resets in {formatResetTime(rateLimitInfo.reset_daily)}
              </p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>

        {/* Warning indicator */}
        {(minuteWarning || dailyWarning) && (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger>
                <AlertTriangle className="h-4 w-4 text-warning" />
              </TooltipTrigger>
              <TooltipContent>
                <p>Approaching rate limit</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
      </div>
    </div>
  );
}
