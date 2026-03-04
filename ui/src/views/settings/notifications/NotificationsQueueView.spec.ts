// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { createNaiveButtonStub, createNaiveStub } from '@/test-utils/naiveUiStubs'

const notificationsApi = {
  listQueue: vi.fn(),
  retryNow: vi.fn(),
  cancel: vi.fn(),
}

const routeApi = {
  query: {} as Record<string, unknown>,
}

vi.mock('@/stores/notifications', () => ({
  useNotificationsStore: () => notificationsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string, params?: Record<string, unknown>) => {
    if (key === 'settings.notifications.queue.total' && params && 'total' in params) {
      return `${key}:${params.total}`
    }
    return key
  } }),
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
    NDataTable: createNaiveStub('NDataTable'),
    NPagination: createNaiveStub('NPagination', {
      props: ['page', 'pageSize', 'itemCount', 'pageSizes', 'disabled'],
      emits: ['update:page', 'update:page-size'],
    }),
    NSelect: createNaiveStub('NSelect', {
      props: ['value', 'options', 'multiple', 'clearable', 'loading'],
      emits: ['update:value'],
    }),
    NSpace: createNaiveStub('NSpace'),
    NSpin: createNaiveStub('NSpin'),
    NTag: createNaiveStub('NTag'),
    useMessage: () => messageApi,
  }
})

import NotificationsQueueView from './NotificationsQueueView.vue'

describe('NotificationsQueueView pagination consistency', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    routeApi.query = {}
    notificationsApi.listQueue.mockImplementation(async (params: { page?: number; pageSize?: number }) => ({
      items: [],
      page: params.page ?? 1,
      page_size: params.pageSize ?? 20,
      total: 125,
    }))
  })

  it('uses pagination control and refreshes by page/pageSize updates', async () => {
    const wrapper = mount(NotificationsQueueView)
    await flushPromises()

    expect(notificationsApi.listQueue).toHaveBeenCalledTimes(1)
    expect(notificationsApi.listQueue).toHaveBeenLastCalledWith(
      expect.objectContaining({ page: 1, pageSize: 20 }),
    )

    const pagination = wrapper.findComponent({ name: 'NPagination' })
    expect(pagination.exists()).toBe(true)
    expect(wrapper.text()).toContain('common.paginationRange')

    pagination.vm.$emit('update:page', 2)
    await flushPromises()

    expect(notificationsApi.listQueue).toHaveBeenLastCalledWith(
      expect.objectContaining({ page: 2, pageSize: 20 }),
    )

    pagination.vm.$emit('update:page-size', 50)
    await flushPromises()

    expect(notificationsApi.listQueue).toHaveBeenLastCalledWith(
      expect.objectContaining({ page: 1, pageSize: 50 }),
    )
  })

  it('shows base empty-state messaging when queue has no entries', async () => {
    notificationsApi.listQueue.mockResolvedValue({
      items: [],
      page: 1,
      page_size: 20,
      total: 0,
    })

    const wrapper = mount(NotificationsQueueView)
    await flushPromises()

    expect(wrapper.text()).toContain('settings.notifications.queue.empty.title')
    expect(wrapper.text()).toContain('settings.notifications.queue.empty.description')
  })

  it('shows filtered no-results state and supports clearing filters', async () => {
    routeApi.query = { status: 'failed' }
    notificationsApi.listQueue.mockResolvedValue({
      items: [],
      page: 1,
      page_size: 20,
      total: 0,
    })

    const wrapper = mount(NotificationsQueueView)
    await flushPromises()

    expect(wrapper.text()).toContain('settings.notifications.queue.empty.noResultsTitle')

    const clearBtn = wrapper.findAll('button').find((b) => b.text() === 'common.clear')
    expect(clearBtn).toBeTruthy()
    await clearBtn!.trigger('click')
    await flushPromises()

    expect(notificationsApi.listQueue).toHaveBeenLastCalledWith(
      expect.objectContaining({ status: undefined, channel: undefined, page: 1, pageSize: 20 }),
    )
  })
})
