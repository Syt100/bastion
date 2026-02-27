// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  return {
    NCard: vue.defineComponent({
      name: 'NCard',
      props: ['bordered'],
      setup(_props, { slots, attrs }) {
        return () => vue.h('div', { 'data-stub': 'NCard', ...attrs }, slots.default?.())
      },
    }),
    NSpin: vue.defineComponent({
      name: 'NSpin',
      setup() {
        return () => vue.h('div', { 'data-stub': 'NSpin' })
      },
    }),
  }
})

import AppEmptyState from './AppEmptyState.vue'

describe('AppEmptyState', () => {
  it('uses card variant by default', () => {
    const wrapper = mount(AppEmptyState, {
      props: {
        title: 'No data',
      },
    })

    expect(wrapper.find('[data-stub="NCard"]').exists()).toBe(true)
  })

  it('renders plain variant without card wrapper', () => {
    const wrapper = mount(AppEmptyState, {
      props: {
        title: 'No data',
        variant: 'plain',
      },
    })

    expect(wrapper.find('[data-stub="NCard"]').exists()).toBe(false)
    expect(wrapper.text()).toContain('No data')
  })

  it('renders inset variant with inset surface class', () => {
    const wrapper = mount(AppEmptyState, {
      props: {
        title: 'No data',
        variant: 'inset',
      },
    })

    expect(wrapper.find('[data-stub="NCard"]').exists()).toBe(false)
    expect(wrapper.classes()).toContain('app-panel-inset')
  })
})
