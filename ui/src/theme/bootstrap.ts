// This file is bundled and inlined into <head> by a Vite transformIndexHtml plugin.
// Keep it dependency-light and side-effect-only: it MUST run before CSS loads to avoid
// a light-mode flash when the user prefers dark mode.

import { DEFAULT_UI_THEME_ID, isUiThemeId, UI_THEME_META_COLORS } from './presets'
import { DEFAULT_UI_BACKGROUND_STYLE, isUiBackgroundStyle, UI_BACKGROUND_NEUTRAL_COLORS, UI_BACKGROUND_STYLE_KEY } from './background'

;(() => {
  try {
    const html = document.documentElement

    const darkMode = localStorage.getItem('bastion.ui.darkMode')
    if (darkMode === 'true') {
      html.classList.add('dark')
    }

    const rawThemeId = localStorage.getItem('bastion.ui.themeId')
    const themeId = isUiThemeId(rawThemeId) ? rawThemeId : DEFAULT_UI_THEME_ID
    html.dataset.theme = themeId

    const rawBg = localStorage.getItem(UI_BACKGROUND_STYLE_KEY)
    const bg = isUiBackgroundStyle(rawBg) ? rawBg : DEFAULT_UI_BACKGROUND_STYLE
    html.dataset.bg = bg

    const themeColorMeta = document.querySelector('meta[name="theme-color"]')
    if (themeColorMeta) {
      const mode = darkMode === 'true' ? 'dark' : 'light'
      themeColorMeta.setAttribute(
        'content',
        bg === 'plain' ? UI_BACKGROUND_NEUTRAL_COLORS[mode] : UI_THEME_META_COLORS[themeId][mode],
      )
    }
  } catch {
    // Ignore (e.g. storage disabled).
  }
})()
