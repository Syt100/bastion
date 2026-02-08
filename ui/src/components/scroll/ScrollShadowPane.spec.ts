// @vitest-environment jsdom
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'
import { describe, expect, it, vi } from 'vitest'

import ScrollShadowPane from './ScrollShadowPane.vue'

type ResizeObserverCtor = new (cb: ResizeObserverCallback) => ResizeObserver

type GlobalWithResizeObserver = {
  ResizeObserver?: ResizeObserverCtor
}

const g = globalThis as unknown as GlobalWithResizeObserver

describe('ScrollShadowPane', () => {
  it('uses overscroll-auto when the pane has no vertical overflow', async () => {
    const original = g.ResizeObserver
    g.ResizeObserver = undefined

    try {
      const wrapper = mount(
        defineComponent({
          components: { ScrollShadowPane },
          template: '<ScrollShadowPane data-testid="pane"><div>content</div></ScrollShadowPane>',
        }),
      )

      await nextTick()

      const scroller = wrapper.get('[data-testid="pane"]')
      expect(scroller.classes()).toContain('overscroll-auto')
      expect(scroller.classes()).not.toContain('overscroll-contain')

      wrapper.unmount()
    } finally {
      g.ResizeObserver = original
    }
  })

  it('switches to overscroll-contain when the pane has vertical overflow', async () => {
    const originalResizeObserver = g.ResizeObserver
    g.ResizeObserver = undefined

    vi.stubGlobal('requestAnimationFrame', ((cb: FrameRequestCallback) => {
      cb(0)
      return 1
    }) as typeof requestAnimationFrame)
    vi.stubGlobal('cancelAnimationFrame', ((id: number) => {
      void id
    }) as typeof cancelAnimationFrame)

    try {
      const wrapper = mount(
        defineComponent({
          components: { ScrollShadowPane },
          template: '<ScrollShadowPane data-testid="pane"><div>content</div></ScrollShadowPane>',
        }),
      )

      await nextTick()

      const scrollerEl = wrapper.get('[data-testid="pane"]').element as HTMLElement

      Object.defineProperty(scrollerEl, 'clientHeight', {
        configurable: true,
        value: 120,
      })
      Object.defineProperty(scrollerEl, 'scrollHeight', {
        configurable: true,
        value: 320,
      })

      scrollerEl.dispatchEvent(new Event('scroll'))
      await nextTick()

      const scroller = wrapper.get('[data-testid="pane"]')
      expect(scroller.classes()).toContain('overscroll-contain')
      expect(scroller.classes()).not.toContain('overscroll-auto')

      wrapper.unmount()
    } finally {
      g.ResizeObserver = originalResizeObserver
      vi.unstubAllGlobals()
    }
  })
})
