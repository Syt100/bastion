import { describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

import { useObservedElementHeightPx } from './resizeObserver'

describe('useObservedElementHeightPx', () => {
  it('measures height even when ResizeObserver is unavailable', () => {
    const original = (globalThis as any).ResizeObserver
    ;(globalThis as any).ResizeObserver = undefined

    try {
      const el = {
        clientHeight: 123.8,
      } as unknown as HTMLElement

      const elRef = ref<HTMLElement | null>(el)
      const { heightPx, start } = useObservedElementHeightPx(elRef)

      start()
      expect(heightPx.value).toBe(123)
    } finally {
      ;(globalThis as any).ResizeObserver = original
    }
  })

  it('updates height when the ResizeObserver callback runs', () => {
    const original = (globalThis as any).ResizeObserver

    const instances: Array<{ cb: () => void; disconnected: boolean }> = []

    class MockResizeObserver {
      disconnected = false
      cb: () => void
      constructor(cb: () => void) {
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

    ;(globalThis as any).ResizeObserver = MockResizeObserver

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
      inst.cb()
      expect(heightPx.value).toBe(240)

      stop()
      expect(inst.disconnected).toBe(true)
    } finally {
      ;(globalThis as any).ResizeObserver = original
      vi.restoreAllMocks()
    }
  })

  it('disconnects the previous observer when start() is called again', () => {
    const original = (globalThis as any).ResizeObserver

    const instances: Array<{ disconnected: boolean }> = []

    class MockResizeObserver {
      disconnected = false
      constructor(_cb: () => void) {
        instances.push(this)
      }
      observe(): void {
        // no-op
      }
      disconnect(): void {
        this.disconnected = true
      }
    }

    ;(globalThis as any).ResizeObserver = MockResizeObserver

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
      ;(globalThis as any).ResizeObserver = original
    }
  })
})
