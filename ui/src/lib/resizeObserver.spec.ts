import { describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

import { useObservedElementHeightPx } from './resizeObserver'

type ResizeObserverCtor = new (cb: ResizeObserverCallback) => ResizeObserver
type GlobalWithResizeObserver = { ResizeObserver?: ResizeObserverCtor }

// Override the global type so the tests can temporarily unset ResizeObserver.
const g = globalThis as unknown as GlobalWithResizeObserver

describe('useObservedElementHeightPx', () => {
  it('measures height even when ResizeObserver is unavailable', () => {
    const original = g.ResizeObserver
    g.ResizeObserver = undefined

    try {
      const el = {
        clientHeight: 123.8,
      } as unknown as HTMLElement

      const elRef = ref<HTMLElement | null>(el)
      const { heightPx, start } = useObservedElementHeightPx(elRef)

      start()
      expect(heightPx.value).toBe(123)
    } finally {
      g.ResizeObserver = original
    }
  })

  it('updates height when the ResizeObserver callback runs', () => {
    const original = g.ResizeObserver

    const instances: Array<{ cb: ResizeObserverCallback; disconnected: boolean }> = []

    class MockResizeObserver {
      disconnected = false
      cb: ResizeObserverCallback
      constructor(cb: ResizeObserverCallback) {
        this.cb = cb
        instances.push(this)
      }
      observe(): void {
        // no-op
      }
      disconnect(): void {
        this.disconnected = true
      }
    }

    g.ResizeObserver = MockResizeObserver as unknown as ResizeObserverCtor

    try {
      let height = 100
      const el = {
        get clientHeight() {
          return height
        },
      } as unknown as HTMLElement

      const elRef = ref<HTMLElement | null>(el)
      const { heightPx, start, stop } = useObservedElementHeightPx(elRef)

      start()
      expect(heightPx.value).toBe(100)
      expect(instances).toHaveLength(1)

      height = 240
      const inst = instances[0]!
      inst.cb([], inst as unknown as ResizeObserver)
      expect(heightPx.value).toBe(240)

      stop()
      expect(inst.disconnected).toBe(true)
    } finally {
      g.ResizeObserver = original
      vi.restoreAllMocks()
    }
  })

  it('disconnects the previous observer when start() is called again', () => {
    const original = g.ResizeObserver

    const instances: Array<{ disconnected: boolean }> = []

    class MockResizeObserver {
      disconnected = false
      constructor(cb: ResizeObserverCallback) {
        void cb
        instances.push(this)
      }
      observe(): void {
        // no-op
      }
      disconnect(): void {
        this.disconnected = true
      }
    }

    g.ResizeObserver = MockResizeObserver as unknown as ResizeObserverCtor

    try {
      const el = {
        clientHeight: 100,
      } as unknown as HTMLElement

      const elRef = ref<HTMLElement | null>(el)
      const { start } = useObservedElementHeightPx(elRef)

      start()
      start()

      expect(instances).toHaveLength(2)
      expect(instances[0]!.disconnected).toBe(true)
    } finally {
      g.ResizeObserver = original
    }
  })
})
