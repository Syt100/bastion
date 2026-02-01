// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { reactive } from 'vue'

const messageApi = {
  error: vi.fn(),
}

const jobsStore = reactive({
  items: [] as Array<{
    id: string
    name: string
    agent_id: string | null
    schedule: string | null
    updated_at: number
    archived_at: number | null
  }>,
  loading: false,
  refresh: vi.fn().mockResolvedValue(undefined),
})

const agentsStore = reactive({
  items: [] as Array<{ id: string; name: string | null; online: boolean; revoked: boolean }>,
  refresh: vi.fn().mockResolvedValue(undefined),
})

const routeApi = reactive<{ params: Record<string, unknown>; path: string }>({ params: {}, path: '' })
const routerApi = { push: vi.fn() }

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'options', 'bordered', 'size'],
      emits: ['update:value', 'update:expanded-keys'],
      setup(_props, { slots, attrs }) {
        return () => vue.h('div', { 'data-stub': name, ...attrs }, slots.default?.())
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    setup(_props, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  return {
    NButton: button,
    NCard: stub('NCard'),
    NInput: stub('NInput'),
    NSelect: stub('NSelect'),
    NSwitch: stub('NSwitch'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('vue-i18n', async () => {
  const vue = await import('vue')
  return {
    useI18n: () => ({ t: (key: string) => key }),
    createI18n: () => ({
      global: {
        locale: vue.ref('en-US'),
        t: (key: string) => key,
      },
    }),
  }
})

vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsStore,
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsStore,
}))

import JobsWorkspaceShellView from './JobsWorkspaceShellView.vue'

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

describe('JobsWorkspaceShellView desktop scrolling', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
    routeApi.path = '/n/hub/jobs'
    routeApi.params = { nodeId: 'hub' }

    jobsStore.items = [
      {
        id: 'job1',
        name: 'Job 1',
        agent_id: null,
        schedule: null,
        updated_at: 1,
        archived_at: null,
      },
    ]
    jobsStore.loading = false
  })

  it('renders a scrollable job list container on desktop', () => {
    const wrapper = mount(JobsWorkspaceShellView, {
      global: {
        stubs: {
          PageHeader: true,
          NodeContextTag: true,
          AppEmptyState: true,
          ListToolbar: true,
          JobEditorModal: true,
          'router-view': true,
        },
      },
    })

    const list = wrapper.find('[data-testid="jobs-list-scroll"]')
    expect(list.exists()).toBe(true)
    expect(list.classes()).toContain('overflow-y-auto')
  })
})
