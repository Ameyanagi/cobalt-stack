'use client'

import { createContext, type ReactNode, useContext, useEffect, useState } from 'react'
import type { ThemeMode, ThemeName } from '@/lib/theme-config'

interface ThemeContextType {
  theme: ThemeName
  mode: ThemeMode
  setTheme: (theme: ThemeName) => void
  setMode: (mode: ThemeMode) => void
  toggleMode: () => void
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined)

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<ThemeName>('default')
  const [mode, setModeState] = useState<ThemeMode>('light')
  const [mounted, setMounted] = useState(false)

  // Load from localStorage on mount
  useEffect(() => {
    setMounted(true)
    const savedTheme = localStorage.getItem('theme') as ThemeName | null
    const savedMode = localStorage.getItem('mode') as ThemeMode | null

    if (
      savedTheme &&
      (savedTheme === 'default' || savedTheme === 'nature' || savedTheme === 'violet-bloom')
    ) {
      setThemeState(savedTheme)
    }
    if (savedMode && (savedMode === 'light' || savedMode === 'dark')) {
      setModeState(savedMode)
    }

    // Apply initial classes
    const effectiveMode = savedMode || 'light'
    const effectiveTheme = savedTheme || 'default'

    document.documentElement.classList.remove('light', 'dark')
    document.documentElement.classList.add(effectiveMode)
    document.documentElement.setAttribute('data-theme', effectiveTheme)
  }, [])

  const setTheme = (newTheme: ThemeName) => {
    setThemeState(newTheme)
    localStorage.setItem('theme', newTheme)
    document.documentElement.setAttribute('data-theme', newTheme)
  }

  const setMode = (newMode: ThemeMode) => {
    setModeState(newMode)
    localStorage.setItem('mode', newMode)
    document.documentElement.classList.remove('light', 'dark')
    document.documentElement.classList.add(newMode)
  }

  const toggleMode = () => {
    setMode(mode === 'light' ? 'dark' : 'light')
  }

  // Prevent hydration mismatch by not rendering until mounted
  if (!mounted) {
    return null
  }

  return (
    <ThemeContext.Provider value={{ theme, mode, setTheme, setMode, toggleMode }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme() {
  const context = useContext(ThemeContext)
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider')
  }
  return context
}
