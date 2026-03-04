// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  return {
    NCard: vue.defineComponent({
      name: 'NCard',
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': 'NCard' }, slots.default?.())
      },
    }),
    NSpin: vue.defineComponent({
      name: 'NSpin',
      setup() {
        return () => null
      },
    }),
  }
})

import ListStatePresenter from './ListStatePresenter.vue'

describe('ListStatePresenter', () => {
  const baseProps = {
    loading: false,
    itemCount: 0,
    baseEmpty: true,
    loadingTitle: 'loading-title',
    baseEmptyTitle: 'base-empty-title',
    filteredEmptyTitle: 'filtered-empty-title',
  }

  it('renders loading state when loading and no items', () => {
    const wrapper = mount(ListStatePresenter, {
      props: { ...baseProps, loading: true },
    })

    expect(wrapper.text()).toContain('loading-title')
  })

  it('renders base empty state and base actions', () => {
    const wrapper = mount(ListStatePresenter, {
      props: baseProps,
      slots: {
        baseActions: () => 'base-actions',
      },
    })

    expect(wrapper.text()).toContain('base-empty-title')
    expect(wrapper.text()).toContain('base-actions')
  })

  it('renders filtered empty state and filtered actions', () => {
    const wrapper = mount(ListStatePresenter, {
      props: { ...baseProps, baseEmpty: false },
      slots: {
        filteredActions: () => 'filtered-actions',
      },
    })

    expect(wrapper.text()).toContain('filtered-empty-title')
    expect(wrapper.text()).toContain('filtered-actions')
  })

  it('renders default slot when data exists', () => {
    const wrapper = mount(ListStatePresenter, {
      props: { ...baseProps, itemCount: 3 },
      slots: {
        default: () => 'list-content',
      },
    })

    expect(wrapper.text()).toContain('list-content')
    expect(wrapper.text()).not.toContain('base-empty-title')
  })
})
