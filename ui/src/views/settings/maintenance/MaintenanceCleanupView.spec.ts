// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { createNaiveButtonStub, createNaiveStub } from '@/test-utils/naiveUiStubs'

const cleanupApi = {
  listTasks: vi.fn(),
  retryNow: vi.fn(),
  ignore: vi.fn(),
  unignore: vi.fn(),
  getTask: vi.fn(),
}

vi.mock('@/stores/incompleteCleanup', () => ({
  useIncompleteCleanupStore: () => cleanupApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/lib/media', async () => {
  const vue = await import('vue')
  return { useMediaQuery: () => vue.ref(true) }
})

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
}

vi.mock('naive-ui', async () => {
  return {
    NButton: createNaiveButtonStub(),
    NCard: createNaiveStub('NCard'),
    NCode: createNaiveStub('NCode'),
    NDataTable: createNaiveStub('NDataTable'),
    NDrawer: createNaiveStub('NDrawer'),
    NDrawerContent: createNaiveStub('NDrawerContent'),
    NInput: createNaiveStub('NInput'),
    NModal: createNaiveStub('NModal'),
    NPagination: createNaiveStub('NPagination', {
      props: ['page', 'pageSize', 'itemCount', 'pageSizes', 'disabled'],
      emits: ['update:page', 'update:page-size'],
    }),
    NPopover: createNaiveStub('NPopover'),
    NRadioButton: createNaiveStub('NRadioButton'),
    NRadioGroup: createNaiveStub('NRadioGroup'),
    NSelect: createNaiveStub('NSelect', {
      props: ['value', 'options', 'multiple', 'clearable'],
      emits: ['update:value'],
    }),
    NSpace: createNaiveStub('NSpace'),
    NSpin: createNaiveStub('NSpin'),
    NTag: createNaiveStub('NTag'),
    useMessage: () => messageApi,
  }
})

import MaintenanceCleanupView from './MaintenanceCleanupView.vue'

describe('MaintenanceCleanupView pagination consistency', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    cleanupApi.listTasks.mockImplementation(async (params: { page?: number; pageSize?: number }) => ({
      items: [],
      page: params.page ?? 1,
      page_size: params.pageSize ?? 20,
      total: 140,
    }))
  })

  it('uses shared pagination control and refreshes by page/pageSize updates', async () => {
    const wrapper = mount(MaintenanceCleanupView)
    await flushPromises()

    expect(cleanupApi.listTasks).toHaveBeenCalledTimes(1)
    expect(cleanupApi.listTasks).toHaveBeenLastCalledWith(
      expect.objectContaining({ page: 1, pageSize: 20 }),
    )

    const pagination = wrapper.findComponent({ name: 'NPagination' })
    expect(pagination.exists()).toBe(true)
    expect(wrapper.text()).toContain('common.paginationRange')

    pagination.vm.$emit('update:page', 2)
    await flushPromises()

    expect(cleanupApi.listTasks).toHaveBeenLastCalledWith(
      expect.objectContaining({ page: 2, pageSize: 20 }),
    )

    pagination.vm.$emit('update:page-size', 50)
    await flushPromises()

    expect(cleanupApi.listTasks).toHaveBeenLastCalledWith(
      expect.objectContaining({ page: 1, pageSize: 50 }),
    )
  })
})
