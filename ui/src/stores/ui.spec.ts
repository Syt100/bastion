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

  it('defaults to mint-teal theme when not stored', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    expect(ui.themeId).toBe('mint-teal')
  })

  it('defaults to aurora background style when not stored', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    expect(ui.backgroundStyle).toBe('aurora')
  })

  it('falls back to default theme when stored value is invalid', () => {
    stubMatchMedia(false)
    localStorage.setItem('bastion.ui.themeId', 'unknown-theme')
    const ui = useUiStore()
    expect(ui.themeId).toBe('mint-teal')
  })

  it('falls back to default background style when stored value is invalid', () => {
    stubMatchMedia(false)
    localStorage.setItem('bastion.ui.backgroundStyle', 'unknown-style')
    const ui = useUiStore()
    expect(ui.backgroundStyle).toBe('aurora')
  })

  it('sets theme and persists', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    ui.setThemeId('ocean-blue')
    expect(ui.themeId).toBe('ocean-blue')
    expect(localStorage.getItem('bastion.ui.themeId')).toBe('ocean-blue')
  })

  it('sets background style and persists', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    ui.setBackgroundStyle('plain')
    expect(ui.backgroundStyle).toBe('plain')
    expect(localStorage.getItem('bastion.ui.backgroundStyle')).toBe('plain')
  })

  it('toggles dark mode and persists', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    expect(ui.darkMode).toBe(false)

    ui.toggleDarkMode()
    expect(ui.darkMode).toBe(true)
    expect(localStorage.getItem('bastion.ui.darkMode')).toBe('true')
  })

  it('persists preferred node id', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    expect(ui.preferredNodeId).toBe('hub')

    ui.setPreferredNodeId('agent1')
    expect(ui.preferredNodeId).toBe('agent1')
    expect(localStorage.getItem('bastion.ui.preferredNodeId')).toBe('agent1')
  })

  it('defaults jobs workspace layout mode to split when not stored', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    expect(ui.jobsWorkspaceLayoutMode).toBe('split')
  })

  it('defaults jobs workspace list view to list when not stored', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    expect(ui.jobsWorkspaceListView).toBe('list')
  })

  it('falls back to split layout mode when stored value is invalid', () => {
    stubMatchMedia(false)
    localStorage.setItem('bastion.ui.jobsWorkspace.layoutMode', 'unknown-layout')
    const ui = useUiStore()
    expect(ui.jobsWorkspaceLayoutMode).toBe('split')
  })

  it('falls back to list view when stored value is invalid', () => {
    stubMatchMedia(false)
    localStorage.setItem('bastion.ui.jobsWorkspace.listView', 'unknown-view')
    const ui = useUiStore()
    expect(ui.jobsWorkspaceListView).toBe('list')
  })

  it('persists jobs workspace layout mode', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    ui.setJobsWorkspaceLayoutMode('list')
    expect(ui.jobsWorkspaceLayoutMode).toBe('list')
    expect(localStorage.getItem('bastion.ui.jobsWorkspace.layoutMode')).toBe('list')
  })

  it('persists jobs workspace list view', () => {
    stubMatchMedia(false)
    const ui = useUiStore()
    ui.setJobsWorkspaceListView('table')
    expect(ui.jobsWorkspaceListView).toBe('table')
    expect(localStorage.getItem('bastion.ui.jobsWorkspace.listView')).toBe('table')
  })
})
