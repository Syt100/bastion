import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { UiThemeId } from '@/theme/presets'
import { UI_PLAIN_SURFACE_2_COLORS, UI_PLAIN_SURFACE_COLORS } from '@/theme/background'
import type { UiBackgroundStyle } from '@/theme/background'

// NOTE: This is a regression test for a real production crash:
// naive-ui -> seemly/rgba throws on CSS variable strings like `var(--app-primary)`.

type UiStateMock = {
  darkMode: boolean
  themeId: UiThemeId
  backgroundStyle: UiBackgroundStyle
  locale: string
}

let uiState: UiStateMock = {
  darkMode: false,
  themeId: 'mint-teal',
  backgroundStyle: 'aurora',
  locale: 'en-US',
}

vi.mock('@/stores/ui', () => ({
  useUiStore: () => uiState,
}))

vi.mock('@/stores/system', () => ({
  useSystemStore: () => ({
    refresh: vi.fn().mockResolvedValue(undefined),
  }),
}))

vi.mock('vue-router', () => ({
  useRoute: () => ({ matched: [] }),
}))

vi.mock('vue-i18n', async (importOriginal) => {
  const actual = await importOriginal<typeof import('vue-i18n')>()
  return {
    ...actual,
    useI18n: () => ({
      t: (key: string) => (key === 'app.name' ? 'Bastion' : key),
    }),
  }
})

vi.mock('naive-ui', async (importOriginal) => {
  const actual = await importOriginal<typeof import('naive-ui')>()
  const vue = await import('vue')
  const NConfigProvider = vue.defineComponent({
    name: 'NConfigProvider',
    props: ['theme', 'locale', 'dateLocale', 'themeOverrides'],
    setup(_props, { slots }) {
      return () => vue.h('div', slots.default?.())
    },
  })
  const NGlobalStyle = vue.defineComponent({
    name: 'NGlobalStyle',
    setup() {
      return () => null
    },
  })
  const NMessageProvider = vue.defineComponent({
    name: 'NMessageProvider',
    setup(_props, { slots }) {
      return () => vue.h('div', slots.default?.())
    },
  })

  return { ...actual, NConfigProvider, NGlobalStyle, NMessageProvider }
})

import App from './App.vue'

describe('App theme overrides', () => {
  it('applies data-bg from store', () => {
    document.documentElement.removeAttribute('data-bg')
    uiState = { darkMode: false, themeId: 'mint-teal', backgroundStyle: 'plain', locale: 'en-US' }
    mount(App, {
      global: {
        stubs: { 'router-view': { template: '<div />' } },
      },
    })
    expect(document.documentElement.dataset.bg).toBe('plain')
    expect(document.body.dataset.bg).toBeUndefined()
  })

  it('uses neutral surfaces for plain mode (dark)', () => {
    uiState = { darkMode: true, themeId: 'ocean-blue', backgroundStyle: 'plain', locale: 'en-US' }

    const wrapper = mount(App, {
      global: {
        stubs: { 'router-view': { template: '<div />' } },
      },
    })
    const overrides = wrapper.findComponent({ name: 'NConfigProvider' }).props('themeOverrides') as any
    expect(overrides).toBeTruthy()
    expect(overrides.common.cardColor).toBe(UI_PLAIN_SURFACE_COLORS.dark)
    expect(overrides.common.tableHeaderColor).toBe(UI_PLAIN_SURFACE_2_COLORS.dark)
    expect(JSON.stringify(overrides)).not.toContain('var(')
  })

  it('keeps theme state on <html> only to avoid CSS variable shadowing', () => {
    document.body.classList.add('dark')
    document.body.setAttribute('data-bg', 'plain')
    uiState = { darkMode: true, themeId: 'ocean-blue', backgroundStyle: 'plain', locale: 'en-US' }
    mount(App, {
      global: {
        stubs: { 'router-view': { template: '<div />' } },
      },
    })
    expect(document.documentElement.classList.contains('dark')).toBe(true)
    expect(document.documentElement.dataset.bg).toBe('plain')
    expect(document.body.classList.contains('dark')).toBe(false)
    expect(document.body.getAttribute('data-bg')).toBe(null)
  })

  it('does not pass CSS var(...) strings into naive-ui theme overrides (light)', () => {
    uiState = { darkMode: false, themeId: 'mint-teal', backgroundStyle: 'aurora', locale: 'en-US' }
    const wrapper = mount(App, {
      global: {
        stubs: { 'router-view': { template: '<div />' } },
      },
    })
    const overrides = wrapper.findComponent({ name: 'NConfigProvider' }).props('themeOverrides')
    expect(overrides).toBeTruthy()
    expect(JSON.stringify(overrides)).not.toContain('var(')
  })

  it('does not pass CSS var(...) strings into naive-ui theme overrides (dark)', () => {
    uiState = { darkMode: true, themeId: 'mint-teal', backgroundStyle: 'aurora', locale: 'en-US' }
    const wrapper = mount(App, {
      global: {
        stubs: { 'router-view': { template: '<div />' } },
      },
    })
    const overrides = wrapper.findComponent({ name: 'NConfigProvider' }).props('themeOverrides')
    expect(overrides).toBeTruthy()
    expect(JSON.stringify(overrides)).not.toContain('var(')
  })
})
