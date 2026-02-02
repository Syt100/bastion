export const UI_THEME_IDS = [
  'mint-teal',
  'ocean-blue',
  'grape-violet',
  'sunset-amber',
  'berry-rose',
  'coral-peach',
] as const

export type UiThemeId = (typeof UI_THEME_IDS)[number]

export const DEFAULT_UI_THEME_ID: UiThemeId = 'mint-teal'

export function isUiThemeId(value: unknown): value is UiThemeId {
  return typeof value === 'string' && (UI_THEME_IDS as readonly string[]).includes(value)
}

export type UiThemePreset = {
  id: UiThemeId
  titleKey: string
  isDefault?: boolean
}

export const UI_THEME_PRESETS: UiThemePreset[] = [
  { id: 'mint-teal', titleKey: 'settings.appearance.themes.mintTeal', isDefault: true },
  { id: 'ocean-blue', titleKey: 'settings.appearance.themes.oceanBlue' },
  { id: 'grape-violet', titleKey: 'settings.appearance.themes.grapeViolet' },
  { id: 'sunset-amber', titleKey: 'settings.appearance.themes.sunsetAmber' },
  { id: 'berry-rose', titleKey: 'settings.appearance.themes.berryRose' },
  { id: 'coral-peach', titleKey: 'settings.appearance.themes.coralPeach' },
]

// Used to seed `meta[name="theme-color"]` before CSS variables are available.
// This avoids a flash of the previous page chrome color on mobile while the app boots.
export const UI_THEME_META_COLORS: Record<UiThemeId, { light: string; dark: string }> = {
  'mint-teal': { light: '#0d9488', dark: '#040b0b' },
  'ocean-blue': { light: '#0284c7', dark: '#050b14' },
  'grape-violet': { light: '#7c3aed', dark: '#070611' },
  'sunset-amber': { light: '#d97706', dark: '#0c0704' },
  'berry-rose': { light: '#e11d48', dark: '#12050b' },
  'coral-peach': { light: '#f97316', dark: '#0c0604' },
}
