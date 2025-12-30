import { describe, expect, it, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

import { useUiStore } from './ui'

describe('useUiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
  })

  it('toggles dark mode and persists', () => {
    const ui = useUiStore()
    expect(ui.darkMode).toBe(false)

    ui.toggleDarkMode()
    expect(ui.darkMode).toBe(true)
    expect(localStorage.getItem('bastion.ui.darkMode')).toBe('true')
  })
})

