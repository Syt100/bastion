// @vitest-environment jsdom
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const NButton = vue.defineComponent({
    name: 'NButton',
    props: ['title', 'disabled'],
    emits: ['click'],
    setup(props, { slots, emit, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            ...attrs,
            title: props.title,
            disabled: props.disabled,
            onClick: (e: MouseEvent) => emit('click', e),
          },
          [slots.icon?.(), slots.default?.()],
        )
    },
  })

  const NIcon = vue.defineComponent({
    name: 'NIcon',
    setup(_, { slots }) {
      return () => vue.h('span', { 'data-stub': 'NIcon' }, slots.default?.())
    },
  })

  const NInput = vue.defineComponent({
    name: 'NInput',
    props: ['value', 'placeholder', 'readonly', 'ariaLabel'],
    emits: ['update:value'],
    setup(props, { slots, emit, attrs }) {
      return () =>
        vue.h(
          'div',
          { ...attrs, class: ['n-input', attrs.class].filter(Boolean).join(' ') },
          [
            vue.h('div', { class: 'n-input-wrapper' }, [
              slots.prefix ? vue.h('div', { class: 'n-input__prefix' }, slots.prefix()) : null,
              vue.h('div', { class: 'n-input__input' }, [
                vue.h('input', {
                  class: 'n-input__input-el',
                  value: props.value,
                  placeholder: props.placeholder,
                  readonly: props.readonly,
                  'aria-label': props.ariaLabel,
                  onInput: (e: Event) => emit('update:value', (e.target as HTMLInputElement).value),
                }),
              ]),
            ]),
          ],
        )
    },
  })

  const NPopover = vue.defineComponent({
    name: 'NPopover',
    props: ['show'],
    emits: ['update:show'],
    setup(props, { slots }) {
      return () => vue.h('div', { 'data-stub': 'NPopover' }, [slots.trigger?.(), props.show ? slots.default?.() : null])
    },
  })

  const NDrawer = vue.defineComponent({
    name: 'NDrawer',
    props: ['show'],
    emits: ['update:show'],
    setup(props, { slots }) {
      return () => (props.show ? vue.h('div', { 'data-stub': 'NDrawer' }, slots.default?.()) : null)
    },
  })

  const NDrawerContent = vue.defineComponent({
    name: 'NDrawerContent',
    setup(_, { slots }) {
      return () => vue.h('div', { 'data-stub': 'NDrawerContent' }, slots.default?.())
    },
  })

  return { NButton, NDrawer, NDrawerContent, NIcon, NInput, NPopover }
})

import PickerPathBarInput from './PickerPathBarInput.vue'

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

async function flushAsync(): Promise<void> {
  await new Promise((r) => setTimeout(r, 0))
  await new Promise((r) => setTimeout(r, 0))
}

describe('PickerPathBarInput', () => {
  const originalGetBoundingClientRect = HTMLElement.prototype.getBoundingClientRect

  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)

    // Make collapse-by-overflow deterministic in jsdom:
    // - elements with data-test-width use that width
    // - others use a simple text-length heuristic
    HTMLElement.prototype.getBoundingClientRect = function () {
      const ds = (this as HTMLElement).dataset?.testWidth
      const width = ds != null ? Number(ds) : Math.ceil(((this as HTMLElement).textContent ?? '').length * 7)
      return {
        x: 0,
        y: 0,
        top: 0,
        left: 0,
        right: width,
        bottom: 0,
        width,
        height: 0,
        toJSON: () => ({}),
      } as DOMRect
    }
  })

  afterEach(() => {
    HTMLElement.prototype.getBoundingClientRect = originalGetBoundingClientRect
  })

  it('does not collapse when the full breadcrumb fits the available width (even if segments are many)', async () => {
    const wrapper = mount(PickerPathBarInput, {
      props: {
        value: '',
        placeholder: 'Path',
        upTitle: 'Up',
        refreshTitle: 'Refresh',
      },
    })
    await flushAsync()

    // Wide bar: no need to collapse.
    wrapper.find('div.relative').element.dataset.testWidth = '800'
    wrapper.find('.flex.items-center.gap-1.shrink-0').element.dataset.testWidth = '60'

    const longPath = '/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t'
    await wrapper.setProps({ value: longPath })
    await flushAsync()

    const segmentButtons = wrapper
      .findAll('button')
      .filter((b) => b.attributes('title') && !['Up', 'Refresh'].includes(b.attributes('title')!))

    expect(segmentButtons.some((b) => b.text().trim() === '…')).toBe(false)
  })

  it('collapses only when overflowing and shows more than the default tail segment count when space permits', async () => {
    const wrapper = mount(PickerPathBarInput, {
      props: {
        value: '',
        placeholder: 'Path',
        upTitle: 'Up',
        refreshTitle: 'Refresh',
      },
    })
    await flushAsync()

    // Narrow bar: will overflow for long paths.
    wrapper.find('div.relative').element.dataset.testWidth = '300'
    wrapper.find('.flex.items-center.gap-1.shrink-0').element.dataset.testWidth = '60'

    const longPath = '/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t'
    await wrapper.setProps({ value: longPath })
    await flushAsync()

    const segmentButtons = wrapper
      .findAll('button')
      .filter((b) => b.attributes('title') && !['Up', 'Refresh'].includes(b.attributes('title')!))

    expect(segmentButtons.some((b) => b.text().trim() === '…')).toBe(true)

    // Default desktop tail is 2, which would render: "/" + "…" + 2 segments = 4 segment buttons.
    // We expect the component to maximize tail segments to fill available width when collapsed.
    expect(segmentButtons.length).toBeGreaterThan(4)
  })

  it('keeps measurement nodes off-layout to avoid introducing horizontal overflow in mobile modals', async () => {
    stubMatchMedia(false)

    const wrapper = mount(PickerPathBarInput, {
      props: {
        value: '/a/b/c/d/e/f/g/h/i/j/k',
        placeholder: 'Path',
        upTitle: 'Up',
        refreshTitle: 'Refresh',
      },
    })
    await flushAsync()

    // Ensure the hidden measurement container exists and cannot affect scroll width.
    const measureContainer = wrapper.find('.w-0.h-0.overflow-hidden.opacity-0.pointer-events-none')
    expect(measureContainer.exists()).toBe(true)
  })
})

