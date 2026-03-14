// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { computed, reactive } from 'vue'

const routeApi = reactive({
  query: { scope: 'all', range: '24h' } as Record<string, unknown>,
})
const routerApi = {
  push: vi.fn(),
  replace: vi.fn(),
}

const commandCenterStore = reactive({
  loading: false,
  snapshot: {
    generated_at: 100,
    scope: { requested: 'all', effective: 'all' },
    range: { preset: '24h', from: 1, to: 100 },
    attention: {
      state: 'ready',
      items: [
        {
          id: 'run:1',
          kind: 'run_failed',
          severity: 'critical',
          title: 'Nightly backup needs review',
          summary: 'upload failed',
          occurred_at: 88,
          scope: 'agent:edge-1',
          context: { run_id: 'run-1' },
          primary_action: { label: 'Open run', href: '/runs/run-1' },
        },
      ],
    },
    critical_activity: { state: 'ready', items: [] },
    recovery_readiness: {
      state: 'degraded',
      overall: 'degraded',
      backup: { recent_success_at: 80, active_jobs: 2, covered_jobs: 1 },
      verify: { recent_success_at: null, active_jobs: 2, covered_jobs: 0 },
      blockers: [{ kind: 'missing_verification', title: 'Verification missing', summary: 'No verify yet', href: '/runs' }],
    },
    watchlist: { state: 'empty', items: [] },
  },
  refresh: vi.fn().mockResolvedValue(undefined),
})

vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string, params?: Record<string, unknown>) => `${key}${params ? JSON.stringify(params) : ''}` }),
}))

vi.mock('@/stores/commandCenter', () => ({
  useCommandCenterStore: () => commandCenterStore,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ preferredScope: 'all', locale: 'en-US' }),
}))

vi.mock('@/lib/datetime', () => ({
  useUnixSecondsFormatter: () => ({ formatUnixSeconds: (value: number | null) => `ts:${String(value)}` }),
}))

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['title', 'type', 'bordered', 'loading'],
      emits: ['click'],
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })
  return {
    NAlert: stub('NAlert'),
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NEmpty: stub('NEmpty'),
    NSkeleton: stub('NSkeleton'),
    NTag: stub('NTag'),
    useMessage: () => ({ error: vi.fn() }),
  }
})

vi.mock('@/components/PageHeader.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'PageHeader', props: ['title', 'subtitle'], setup(_props, { slots }) { return () => vue.h('div', slots.default?.()) } }) }
})

import CommandCenterView from './CommandCenterView.vue'

describe('CommandCenterView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('refreshes on mount and renders attention plus readiness blockers', async () => {
    mount(CommandCenterView)

    expect(commandCenterStore.refresh).toHaveBeenCalledWith({ scope: 'all', range: '24h' })
  })

  it('navigates through item actions', async () => {
    const wrapper = mount(CommandCenterView)
    const buttons = wrapper.findAllComponents({ name: 'NButton' })
    await buttons[buttons.length - 1]?.vm.$emit('click')

    expect(routerApi.push).toHaveBeenCalledWith('/runs/run-1')
  })
})
