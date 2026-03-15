// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
}

const runsApi = {
  listWorkspace: vi.fn(),
}

const routeApi = {
  query: {} as Record<string, unknown>,
}

const routerApi = {
  push: vi.fn(),
  replace: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['loading', 'bordered', 'size', 'type', 'title', 'page', 'pageSize', 'itemCount'],
      emits: ['update:page'],
      setup(_, { slots, attrs }) {
        return () =>
          vue.h(
            'div',
            { 'data-stub': name, ...attrs },
            [slots.default?.(), slots.header?.(), slots.footer?.()].filter(Boolean),
          )
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    props: ['disabled', 'loading', 'type', 'quaternary', 'secondary'],
    setup(props, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            disabled: !!props.disabled,
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
            ...attrs,
          },
          slots.default?.(),
        )
    },
  })

  const input = vue.defineComponent({
    name: 'NInput',
    props: ['value', 'placeholder'],
    emits: ['update:value', 'keyup', 'clear'],
    setup(props, { attrs }) {
      return () =>
        vue.h('input', {
          'data-stub': 'NInput',
          value: props.value ?? '',
          placeholder: props.placeholder,
          onInput: (event: Event) => {
            const target = event.target as HTMLInputElement
            ;(attrs as { 'onUpdate:value'?: ((value: string) => void) | undefined })['onUpdate:value']?.(target.value)
          },
        })
    },
  })

  const select = vue.defineComponent({
    name: 'NSelect',
    props: ['value', 'options', 'placeholder'],
    emits: ['update:value'],
    setup(props, { attrs }) {
      return () =>
        vue.h(
          'select',
          {
            'data-stub': 'NSelect',
            value: props.value ?? '',
            onChange: (event: Event) => {
              const target = event.target as HTMLSelectElement
              ;(attrs as { 'onUpdate:value'?: ((value: string) => void) | undefined })['onUpdate:value']?.(target.value)
            },
          },
          ((props.options as Array<{ label: string; value: string }> | undefined) ?? []).map((option) =>
            vue.h('option', { value: option.value }, option.label),
          ),
        )
    },
  })

  const pagination = vue.defineComponent({
    name: 'NPagination',
    props: ['page', 'pageSize', 'itemCount'],
    emits: ['update:page'],
    setup(props, { attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NPagination',
            onClick: () => (attrs as { 'onUpdate:page'?: ((value: number) => void) | undefined })['onUpdate:page']?.((props.page as number) + 1),
          },
          `${props.page}/${props.itemCount}`,
        )
    },
  })

  const tag = vue.defineComponent({
    name: 'NTag',
    setup(_, { slots, attrs }) {
      return () => vue.h('span', { 'data-stub': 'NTag', ...attrs }, slots.default?.())
    },
  })

  return {
    NButton: button,
    NCard: stub('NCard'),
    NInput: input,
    NPagination: pagination,
    NSelect: select,
    NTag: tag,
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (typeof params?.shown === 'number') return `${key}:${params.shown}/${params.total}/${params.page}`
      return key
    },
  }),
}))

vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US', preferredScope: 'hub' }),
}))

vi.mock('@/stores/runs', () => ({
  useRunsStore: () => runsApi,
}))

vi.mock('@/components/AppEmptyState.vue', () => ({
  default: {
    name: 'AppEmptyState',
    props: ['title', 'description'],
    template: '<div data-stub="AppEmptyState">{{ title }}|{{ description }}</div>',
  },
}))

vi.mock('@/components/PageHeader.vue', () => ({
  default: {
    name: 'PageHeader',
    props: ['title', 'subtitle'],
    template: '<div data-stub="PageHeader"><slot />{{ title }}|{{ subtitle }}</div>',
  },
}))

vi.mock('@/components/list/ListPageScaffold.vue', () => ({
  default: {
    name: 'ListPageScaffold',
    template: '<div data-stub="ListPageScaffold"><slot name="toolbar" /><slot name="content" /><slot name="footer" /></div>',
  },
}))

vi.mock('@/components/list/ListToolbar.vue', () => ({
  default: {
    name: 'ListToolbar',
    template: '<div data-stub="ListToolbar"><slot name="search" /><slot name="filters" /><slot name="actions" /></div>',
  },
}))

import RunsView from './RunsView.vue'

function buttonByText(wrapper: ReturnType<typeof mount>, text: string) {
  return wrapper.findAll('button').find((button) => button.text().trim() === text)
}

describe('RunsView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    routeApi.query = {
      scope: 'agent:db-1',
      status: 'failed',
      kind: 'restore',
      q: 'timeout',
      page: '2',
      range: '7d',
    }
    runsApi.listWorkspace.mockResolvedValue({
      scope: {
        requested: 'agent:db-1',
        effective: 'agent:db-1',
      },
      filters: {
        q: 'timeout',
        status: 'failed',
        kind: 'restore',
        range: '7d',
      },
      items: [
        {
          id: 'run-1',
          job_id: 'job-1',
          job_name: 'Nightly backup',
          scope: 'agent:db-1',
          node_id: 'agent-1',
          node_name: 'db-1',
          status: 'failed',
          kind: 'restore',
          started_at: 1710000000,
          ended_at: 1710000300,
          error: 'Remote endpoint timed out',
          failure_title: 'Restore upload timed out',
        },
      ],
      page: 2,
      page_size: 20,
      total: 23,
    })
  })

  it('loads the global runs workspace using route filters and opens canonical run routes', async () => {
    const wrapper = mount(RunsView)
    await flushPromises()

    expect(runsApi.listWorkspace).toHaveBeenCalledWith({
      scope: 'agent:db-1',
      status: 'failed',
      kind: 'restore',
      range: '7d',
      q: 'timeout',
      page: 2,
      pageSize: 20,
    })
    expect(wrapper.text()).toContain('runs.list.summary:1/23/2')
    expect(wrapper.text()).toContain('Restore upload timed out')

    await buttonByText(wrapper, 'runs.actions.openRun')!.trigger('click')
    expect(routerApi.push).toHaveBeenCalledWith({
      path: '/runs/run-1',
      query: {
        from_scope: 'agent:db-1',
        from_job: 'job-1',
        from_section: 'history',
      },
    })

    await buttonByText(wrapper, 'runs.actions.openJob')!.trigger('click')
    expect(routerApi.push).toHaveBeenCalledWith({
      path: '/jobs/job-1/history',
      query: { scope: 'agent:db-1' },
    })
  })

  it('keeps scope while clearing secondary filters', async () => {
    const wrapper = mount(RunsView)
    await flushPromises()

    await buttonByText(wrapper, 'common.clear')!.trigger('click')

    expect(routerApi.replace).toHaveBeenCalledWith({
      query: {
        scope: 'agent:db-1',
        range: '7d',
      },
    })
  })
})
