// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const routeApi = {
  params: { runId: 'run-1' } as Record<string, unknown>,
  query: {} as Record<string, unknown>,
}

const routerApi = {
  push: vi.fn(),
}

vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  return {
    NButton: vue.defineComponent({
      name: 'NButton',
      setup(_, { slots, attrs }) {
        return () =>
          vue.h(
            'button',
            {
              onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
            },
            slots.default?.(),
          )
      },
    }),
  }
})

vi.mock('@/components/PageHeader.vue', () => ({
  default: {
    name: 'PageHeader',
    props: ['title', 'subtitle'],
    template: '<div data-stub="PageHeader"><slot />{{ title }}|{{ subtitle }}</div>',
  },
}))

vi.mock('@/components/runs/RunDetailPanel.vue', () => ({
  default: {
    name: 'RunDetailPanel',
    props: ['runId'],
    template: '<div data-stub="RunDetailPanel">{{ runId }}</div>',
  },
}))

import RunDetailRouteView from './RunDetailRouteView.vue'

describe('RunDetailRouteView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    routeApi.params = { runId: 'run-1' }
    routeApi.query = {}
  })

  it('returns to the originating job section when source context is available', async () => {
    routeApi.query = {
      from_scope: 'agent:db-1',
      from_job: 'job-9',
      from_section: 'history',
    }

    const wrapper = mount(RunDetailRouteView)
    await wrapper.get('button').trigger('click')

    expect(wrapper.get('[data-stub="RunDetailPanel"]').text()).toContain('run-1')
    expect(routerApi.push).toHaveBeenCalledWith({
      path: '/jobs/job-9/history',
      query: { scope: 'agent:db-1' },
    })
  })

  it('falls back to the global runs workspace when only scope context exists', async () => {
    routeApi.query = {
      from_scope: 'hub',
    }

    const wrapper = mount(RunDetailRouteView)
    await wrapper.get('button').trigger('click')

    expect(routerApi.push).toHaveBeenCalledWith({
      path: '/runs',
      query: { scope: 'hub' },
    })
  })
})
