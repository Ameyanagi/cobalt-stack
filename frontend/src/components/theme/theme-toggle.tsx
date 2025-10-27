'use client'

import { Moon, Sun } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTheme } from '@/contexts/theme-context'

export function ThemeToggle() {
  const { mode, toggleMode } = useTheme()

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={toggleMode}
      aria-label={`Switch to ${mode === 'light' ? 'dark' : 'light'} mode`}
    >
      {mode === 'light' ? <Moon className="h-5 w-5" /> : <Sun className="h-5 w-5" />}
    </Button>
  )
}
