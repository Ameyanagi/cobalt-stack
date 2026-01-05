export type ThemeMode = 'light' | 'dark'

export type ThemeName = 'default' | 'nature' | 'violet-bloom'

export interface Theme {
  id: ThemeName
  name: string
  description: string
  preview: {
    primary: string
    secondary: string
    accent: string
  }
}

export interface ThemeConfig {
  theme: ThemeName
  mode: ThemeMode
}

export const themes: Record<ThemeName, Theme> = {
  default: {
    id: 'default',
    name: 'Cobalt',
    description: 'Clean and professional cobalt blue theme',
    preview: {
      primary: '#0047AB',
      secondary: '#6B8EC9',
      accent: '#2E5A9E',
    },
  },
  nature: {
    id: 'nature',
    name: 'Nature',
    description: 'Calming green tones inspired by nature',
    preview: {
      primary: '#22c55e',
      secondary: '#84cc16',
      accent: '#10b981',
    },
  },
  'violet-bloom': {
    id: 'violet-bloom',
    name: 'Violet Bloom',
    description: 'Elegant purple palette with soft edges',
    preview: {
      primary: '#8b5cf6',
      secondary: '#a855f7',
      accent: '#c084fc',
    },
  },
}
