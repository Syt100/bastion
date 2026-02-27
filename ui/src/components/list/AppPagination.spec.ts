// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  return {
    NPagination: vue.defineComponent({
      name: 'NPagination',
      props: ['page', 'pageSize', 'itemCount', 'pageSizes'],
      emits: ['update:page', 'update:page-size'],
      setup(_props, { slots, attrs }) {
        return () => vue.h('div', { 'data-stub': 'NPagination', ...attrs }, slots.default?.())
      },
    }),
  }
})

import AppPagination from './AppPagination.vue'

describe('AppPagination', () => {
  it('renders total label when provided', () => {
    const wrapper = mount(AppPagination, {
      props: {
        page: 1,
        pageSize: 20,
        itemCount: 100,
        totalLabel: 'Total: 100',
      },
    })

    expect(wrapper.text()).toContain('Total: 100')
    expect(wrapper.find('[data-stub="NPagination"]').exists()).toBe(true)
  })

  it('forwards page update events', async () => {
    const wrapper = mount(AppPagination, {
      props: {
        page: 1,
        pageSize: 20,
        itemCount: 100,
      },
    })

    wrapper.findComponent({ name: 'NPagination' }).vm.$emit('update:page', 2)
    wrapper.findComponent({ name: 'NPagination' }).vm.$emit('update:page-size', 50)
    await wrapper.vm.$nextTick()

    expect(wrapper.emitted('update:page')?.[0]).toEqual([2])
    expect(wrapper.emitted('update:pageSize')?.[0]).toEqual([50])
  })
})
