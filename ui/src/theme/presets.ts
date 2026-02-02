export const UI_THEME_PRESETS = [
  {
    id: 'mint-teal',
    titleKey: 'settings.appearance.themes.mintTeal',
    isDefault: true,
    metaColors: { light: '#0d9488', dark: '#040b0b' },
  },
  {
    id: 'ocean-blue',
    titleKey: 'settings.appearance.themes.oceanBlue',
    isDefault: false,
    metaColors: { light: '#0284c7', dark: '#050b14' },
  },
  {
    id: 'grape-violet',
    titleKey: 'settings.appearance.themes.grapeViolet',
    isDefault: false,
    metaColors: { light: '#7c3aed', dark: '#070611' },
  },
  {
    id: 'sunset-amber',
    titleKey: 'settings.appearance.themes.sunsetAmber',
    isDefault: false,
    metaColors: { light: '#d97706', dark: '#0c0704' },
  },
  {
    id: 'berry-rose',
    titleKey: 'settings.appearance.themes.berryRose',
    isDefault: false,
    metaColors: { light: '#e11d48', dark: '#12050b' },
  },
  {
    id: 'coral-peach',
    titleKey: 'settings.appearance.themes.coralPeach',
    isDefault: false,
    metaColors: { light: '#f97316', dark: '#0c0604' },
  },
] as const

export type UiThemeId = (typeof UI_THEME_PRESETS)[number]['id']

export type UiThemePreset = {
  id: UiThemeId
  titleKey: string
  isDefault: boolean
  metaColors: { light: string; dark: string }
}

export const UI_THEME_IDS: readonly UiThemeId[] = UI_THEME_PRESETS.map((t) => t.id)
const UI_THEME_ID_SET = new Set<string>(UI_THEME_IDS)

export function isUiThemeId(value: unknown): value is UiThemeId {
  return typeof value === 'string' && UI_THEME_ID_SET.has(value)
}

export const DEFAULT_UI_THEME_ID: UiThemeId = (UI_THEME_PRESETS.find((t) => t.isDefault) ?? UI_THEME_PRESETS[0]).id

// Used to seed `meta[name="theme-color"]` before CSS variables are available.
// This avoids a flash of the previous page chrome color on mobile while the app boots.
export const UI_THEME_META_COLORS = Object.fromEntries(
  UI_THEME_PRESETS.map((t) => [t.id, t.metaColors]),
) as Record<UiThemeId, { light: string; dark: string }>
