// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, reactive } from 'vue'

// Regression test: theme switching must update html[data-theme] and force Naive UI
// theme overrides to recompute (otherwise component colors can become stale).

const uiStore = reactive({
  darkMode: false,
  themeId: 'mint-teal',
  locale: 'en-US',
  toggleDarkMode: vi.fn(),
  setDarkMode: vi.fn(),
  setThemeId: vi.fn((id: string) => {
    uiStore.themeId = id
  }),
  setLocale: vi.fn(),
  preferredNodeId: 'hub',
  setPreferredNodeId: vi.fn(),
})

vi.mock('@/stores/ui', () => ({
  useUiStore: () => uiStore,
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

describe('Theme presets', () => {
  beforeEach(() => {
    uiStore.darkMode = false
    uiStore.themeId = 'mint-teal'
    document.documentElement.className = ''
    document.documentElement.removeAttribute('data-theme')
    document.documentElement.style.cssText = ''
  })

  it('applies data-theme and recomputes theme overrides when themeId changes', async () => {
    // Provide concrete tokens so the test can observe a real color change.
    document.documentElement.style.setProperty('--app-primary', '#0d9488')

    const wrapper = mount(App, {
      global: { stubs: { 'router-view': { template: '<div />' } } },
    })

    expect(document.documentElement.dataset.theme).toBe('mint-teal')
    const overrides1 = wrapper.findComponent({ name: 'NConfigProvider' }).props('themeOverrides') as any
    expect(overrides1.common.primaryColor).toBe('#0d9488')

    uiStore.themeId = 'ocean-blue'
    document.documentElement.style.setProperty('--app-primary', '#0284c7')
    await nextTick()

    expect(document.documentElement.dataset.theme).toBe('ocean-blue')
    const overrides2 = wrapper.findComponent({ name: 'NConfigProvider' }).props('themeOverrides') as any
    expect(overrides2.common.primaryColor).toBe('#0284c7')
  })
})

