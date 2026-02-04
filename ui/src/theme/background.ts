export type UiBackgroundStyle = 'aurora' | 'solid' | 'plain'

export const UI_BACKGROUND_STYLE_KEY = 'bastion.ui.backgroundStyle'

export const UI_BACKGROUND_STYLES = ['aurora', 'solid', 'plain'] as const satisfies readonly UiBackgroundStyle[]
const UI_BACKGROUND_STYLE_SET = new Set<string>(UI_BACKGROUND_STYLES)

export const DEFAULT_UI_BACKGROUND_STYLE: UiBackgroundStyle = 'aurora'

export const UI_BACKGROUND_NEUTRAL_COLORS = {
  light: '#f8fafc',
  dark: '#0b0b0b',
} as const

// "Plain" (neutral) mode should keep surfaces/chrome neutral (not theme-tinted),
// especially in dark mode where theme presets use tinted surface tokens.
export const UI_PLAIN_SURFACE_COLORS = {
  light: '#ffffff',
  dark: '#111111',
} as const

export const UI_PLAIN_SURFACE_2_COLORS = {
  light: '#f1f5f9',
  dark: '#1a1a1a',
} as const

export function isUiBackgroundStyle(value: unknown): value is UiBackgroundStyle {
  return typeof value === 'string' && UI_BACKGROUND_STYLE_SET.has(value)
}
