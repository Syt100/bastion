// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { createNaiveButtonStub, createNaiveInputStub, createNaiveStub } from '@/test-utils/naiveUiStubs'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
}

const integrationsApi = {
  getStorage: vi.fn(),
}

const secretsApi = {
  getWebdav: vi.fn(),
  upsertWebdav: vi.fn(),
  deleteWebdav: vi.fn(),
}

const agentsApi = {
  listLabelIndex: vi.fn().mockResolvedValue([]),
}

const bulkOpsApi = {
  previewWebdavSecretDistribute: vi.fn(),
  create: vi.fn(),
}

const routerApi = {
  push: vi.fn(),
}

const routeApi = {
  params: {} as Record<string, unknown>,
  query: { scope: 'hub' } as Record<string, unknown>,
}

vi.mock('@/stores/integrations', () => ({
  useIntegrationsStore: () => integrationsApi,
}))

vi.mock('@/stores/secrets', () => ({
  useSecretsStore: () => secretsApi,
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/bulkOperations', () => ({
  useBulkOperationsStore: () => bulkOpsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US', preferredScope: 'hub' }),
}))

vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('@/lib/media', async () => {
  const vue = await import('vue')
  return { useMediaQuery: () => vue.computed(() => false) }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/components/AppModalShell.vue', () => ({
  default: {
    props: ['show', 'title'],
    emits: ['update:show'],
    template: '<div data-stub="AppModalShell"><slot /><slot name="footer" /></div>',
  },
}))

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const formItem = vue.defineComponent({
    name: 'NFormItem',
    props: ['label'],
    setup(props, { slots }) {
      return () =>
        vue.h('div', { 'data-stub': 'NFormItem' }, [
          props.label ? vue.h('div', { 'data-stub': 'NFormItemLabel' }, String(props.label)) : null,
          slots.default?.(),
        ])
    },
  })

  const stub = (name: string) =>
    createNaiveStub(name, {
      props: ['value', 'columns', 'data', 'loading', 'title'],
      emits: ['update:value', 'update:checked'],
    })

  return {
    NAlert: stub('NAlert'),
    NButton: createNaiveButtonStub(),
    NCard: stub('NCard'),
    NCheckbox: stub('NCheckbox'),
    NDataTable: stub('NDataTable'),
    NForm: stub('NForm'),
    NFormItem: formItem,
    NInput: createNaiveInputStub(),
    NPopconfirm: stub('NPopconfirm'),
    NRadioButton: stub('NRadioButton'),
    NRadioGroup: stub('NRadioGroup'),
    NSelect: stub('NSelect'),
    NSpace: stub('NSpace'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

import SettingsStorageView from './SettingsStorageView.vue'

describe('SettingsStorageView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    routeApi.params = {}
    routeApi.query = { scope: 'hub' }
    integrationsApi.getStorage.mockResolvedValue({
      node_id: 'hub',
      summary: { items_total: 1, in_use_total: 1, invalid_total: 1 },
      items: [
        {
          name: 'primary',
          updated_at: 100,
          usage_total: 1,
          usage: [
            {
              job_id: 'job-1',
              job_name: 'Nightly DB',
              latest_run_id: 'run-1',
              latest_run_status: 'failed',
              latest_run_at: 90,
            },
          ],
          health: {
            state: 'attention',
            latest_run_id: 'run-1',
            latest_run_status: 'failed',
            latest_run_at: 90,
          },
        },
      ],
    })
  })

  it('renders storage usage and health context from integrations details', async () => {
    const wrapper = mount(SettingsStorageView)
    await flushPromises()

    expect(integrationsApi.getStorage).toHaveBeenCalledWith('hub')
    expect(wrapper.text()).toContain('primary')
    expect(wrapper.text()).toContain('integrations.storage.invalidSummary')
    expect(wrapper.text()).toContain('integrations.storage.usageCount')
    expect(wrapper.text()).toContain('integrations.storage.health.attention')
    expect(wrapper.text()).toContain('Nightly DB')
  })
})
