export type UiBackgroundStyle = 'aurora' | 'solid' | 'plain'

export const UI_BACKGROUND_STYLE_KEY = 'bastion.ui.backgroundStyle'

export const UI_BACKGROUND_STYLES = ['aurora', 'solid', 'plain'] as const satisfies readonly UiBackgroundStyle[]
const UI_BACKGROUND_STYLE_SET = new Set<string>(UI_BACKGROUND_STYLES)

export const DEFAULT_UI_BACKGROUND_STYLE: UiBackgroundStyle = 'aurora'

export const UI_BACKGROUND_NEUTRAL_COLORS = {
  light: '#f8fafc',
  dark: '#0b0b0b',
} as const

export function isUiBackgroundStyle(value: unknown): value is UiBackgroundStyle {
  return typeof value === 'string' && UI_BACKGROUND_STYLE_SET.has(value)
}

