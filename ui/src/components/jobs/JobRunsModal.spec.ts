// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

type Deferred<T> = {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (error?: unknown) => void
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (error?: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

const jobsApi = {
  listRuns: vi.fn(),
}

const messageApi = {
  success: vi.fn(),
  warning: vi.fn(),
  error: vi.fn(),
}

vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/components/AppModalShell.vue', async () => {
  const vue = await import('vue')
  return {
    default: vue.defineComponent({
      name: 'AppModalShell',
      props: ['show'],
      emits: ['update:show'],
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': 'AppModalShell' }, [slots.default?.(), slots.footer?.()])
      },
    }),
  }
})

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const button = vue.defineComponent({
    name: 'NButton',
    props: ['disabled'],
    setup(props, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            disabled: Boolean(props.disabled),
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  const dataTable = vue.defineComponent({
    name: 'NDataTable',
    props: ['data', 'loading'],
    setup(props) {
      return () => {
        const data = Array.isArray((props as { data?: unknown[] }).data) ? ((props as { data?: unknown[] }).data ?? []) : []
        const ids = data.map((item) => {
          if (item && typeof item === 'object' && 'id' in item) return String((item as { id: unknown }).id)
          return '?'
        })
        return vue.h('div', { 'data-stub': 'NDataTable' }, ids.join(','))
      }
    },
  })

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })

  return {
    NButton: button,
    NDataTable: dataTable,
    NSpace: stub('NSpace'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

import JobRunsModal from './JobRunsModal.vue'

async function flush(): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
}

describe('JobRunsModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('keeps only latest run list response when open is called repeatedly', async () => {
    const calls: Array<{ request: Deferred<Array<{ id: string }>>; signal: AbortSignal | undefined }> = []
    jobsApi.listRuns.mockImplementation((_jobId: string, options?: { signal?: AbortSignal }) => {
      const request = deferred<Array<{ id: string }>>()
      calls.push({ request, signal: options?.signal })
      return request.promise
    })

    const wrapper = mount(JobRunsModal)
    const vm = wrapper.vm as unknown as { open: (jobId: string) => Promise<void> }

    const openA = vm.open('job-a')
    await flush()
    const openB = vm.open('job-b')
    await flush()

    expect(calls).toHaveLength(2)
    expect(calls[0]?.signal?.aborted).toBe(true)

    calls[0]!.request.resolve([{ id: 'run-from-a' }])
    await openA
    await flush()

    calls[1]!.request.resolve([{ id: 'run-from-b' }])
    await openB
    await flush()

    expect(wrapper.text()).toContain('run-from-b')
    expect(wrapper.text()).not.toContain('run-from-a')
    expect(messageApi.error).not.toHaveBeenCalled()
  })
})
