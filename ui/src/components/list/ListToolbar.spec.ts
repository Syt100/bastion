// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { Comment, h } from 'vue'

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
  }
})

import ListToolbar from './ListToolbar.vue'

describe('ListToolbar', () => {
  it('does not render actions container when actions slot is omitted', () => {
    const wrapper = mount(ListToolbar, {
      slots: {
        search: () => h('div', 'search'),
      },
    })

    expect(wrapper.find('[data-testid=\"list-toolbar-actions\"]').exists()).toBe(false)
  })

  it('does not render actions container when actions slot renders only comments', () => {
    const wrapper = mount(ListToolbar, {
      props: { embedded: true },
      slots: {
        search: () => h('div', 'search'),
        actions: () => h(Comment),
      },
    })

    expect(wrapper.find('[data-testid=\"list-toolbar-actions\"]').exists()).toBe(false)
  })

  it('renders actions container when actions slot has content', () => {
    const wrapper = mount(ListToolbar, {
      props: { embedded: true },
      slots: {
        search: () => h('div', 'search'),
        actions: () => h('button', 'action'),
      },
    })

    expect(wrapper.find('[data-testid=\"list-toolbar-actions\"]').exists()).toBe(true)
  })
})

