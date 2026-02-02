import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

// NOTE: This is a regression test for a real production crash:
// naive-ui -> seemly/rgba throws on CSS variable strings like `var(--app-primary)`.

let uiState = {
  darkMode: false,
  themeId: 'mint-teal' as const,
  locale: 'en-US' as const,
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
  it('does not pass CSS var(...) strings into naive-ui theme overrides (light)', () => {
    uiState = { darkMode: false, themeId: 'mint-teal', locale: 'en-US' }
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
    uiState = { darkMode: true, themeId: 'mint-teal', locale: 'en-US' }
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
