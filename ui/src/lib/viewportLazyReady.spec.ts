// @vitest-environment jsdom
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick, onMounted, ref } from 'vue'
import { describe, expect, it } from 'vitest'

import { useViewportLazyReady } from './viewportLazyReady'

type IntersectionObserverCtor = new (
  cb: IntersectionObserverCallback,
  options?: IntersectionObserverInit,
) => IntersectionObserver

type GlobalWithIntersectionObserver = {
  IntersectionObserver?: IntersectionObserverCtor
}

const g = globalThis as unknown as GlobalWithIntersectionObserver

function mountHost(rootMargin = '0px') {
  return mount(
    defineComponent({
      setup() {
        const enabled = ref(true)
        const domTarget = ref<HTMLElement | null>(null)
        const lazy = useViewportLazyReady(enabled, { rootMargin })

        onMounted(() => {
          lazy.target.value = domTarget.value
        })

        return {
          enabled,
          domTarget,
          ready: lazy.ready,
        }
      },
      template: '<div><div ref=domTarget data-testid=target></div></div>',
    }),
  )
}

describe('useViewportLazyReady', () => {
  it('marks ready immediately when IntersectionObserver is unavailable', async () => {
    const original = g.IntersectionObserver
    g.IntersectionObserver = undefined

    try {
      const wrapper = mountHost()
      await nextTick()
      expect((wrapper.vm as { ready: boolean }).ready).toBe(true)
      wrapper.unmount()
    } finally {
      g.IntersectionObserver = original
    }
  })

  it('marks ready after intersection callback and disconnects observer', async () => {
    const original = g.IntersectionObserver

    const instances: Array<{
      cb: IntersectionObserverCallback
      disconnected: boolean
      options?: IntersectionObserverInit
    }> = []

    class MockIntersectionObserver {
      cb: IntersectionObserverCallback
      disconnected = false
      options?: IntersectionObserverInit

      constructor(cb: IntersectionObserverCallback, options?: IntersectionObserverInit) {
        this.cb = cb
        this.options = options
        instances.push(this)
      }

      observe(): void {
        // no-op
      }

      disconnect(): void {
        this.disconnected = true
      }

      unobserve(): void {
        // no-op
      }

      takeRecords(): IntersectionObserverEntry[] {
        return []
      }

      root: Element | null = null
      rootMargin = '0px'
      thresholds: ReadonlyArray<number> = [0]
    }

    g.IntersectionObserver = MockIntersectionObserver as unknown as IntersectionObserverCtor

    try {
      const wrapper = mountHost('200px 0px')
      await nextTick()

      expect(instances).toHaveLength(1)
      expect((wrapper.vm as { ready: boolean }).ready).toBe(false)
      expect(instances[0]!.options?.rootMargin).toBe('200px 0px')

      const entry = { isIntersecting: true } as IntersectionObserverEntry
      instances[0]!.cb([entry], instances[0] as unknown as IntersectionObserver)
      await nextTick()

      expect((wrapper.vm as { ready: boolean }).ready).toBe(true)
      expect(instances[0]!.disconnected).toBe(true)
      wrapper.unmount()
    } finally {
      g.IntersectionObserver = original
    }
  })

  it('resets ready when the section is disabled', async () => {
    const original = g.IntersectionObserver

    const instances: Array<{ cb: IntersectionObserverCallback; disconnected: boolean }> = []

    class MockIntersectionObserver {
      cb: IntersectionObserverCallback
      disconnected = false

      constructor(cb: IntersectionObserverCallback) {
        this.cb = cb
        instances.push(this)
      }

      observe(): void {
        // no-op
      }

      disconnect(): void {
        this.disconnected = true
      }

      unobserve(): void {
        // no-op
      }

      takeRecords(): IntersectionObserverEntry[] {
        return []
      }

      root: Element | null = null
      rootMargin = '0px'
      thresholds: ReadonlyArray<number> = [0]
    }

    g.IntersectionObserver = MockIntersectionObserver as unknown as IntersectionObserverCtor

    try {
      const wrapper = mountHost()
      await nextTick()

      expect(instances).toHaveLength(1)

      const vm = wrapper.vm as { ready: boolean; enabled: boolean }
      const entry = { isIntersecting: true } as IntersectionObserverEntry
      instances[0]!.cb([entry], instances[0] as unknown as IntersectionObserver)
      await nextTick()
      expect(vm.ready).toBe(true)

      vm.enabled = false
      await nextTick()
      expect(vm.ready).toBe(false)

      wrapper.unmount()
    } finally {
      g.IntersectionObserver = original
    }
  })
})
