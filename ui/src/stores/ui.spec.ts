import { describe, expect, it, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

import { useUiStore } from './ui'

function stubMatchMedia(matches: boolean): void {
  vi.stubGlobal(
    'matchMedia',
    ((query: string) => ({
      matches,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })) as unknown as typeof window.matchMedia,
  )
}

describe('useUiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
  })

  it('defaults to system dark mode when not stored', () => {
    stubMatchMedia(true)
    const ui = useUiStore()
    expect(ui.darkMode).toBe(true)
  })

  it('toggles dark mode and persists', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    expect(ui.darkMode).toBe(false)

    ui.toggleDarkMode()
    expect(ui.darkMode).toBe(true)
    expect(localStorage.getItem('bastion.ui.darkMode')).toBe('true')
  })
})
