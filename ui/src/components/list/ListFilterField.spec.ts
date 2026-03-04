// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import ListFilterField from './ListFilterField.vue'

describe('ListFilterField', () => {
  it('renders inline default control width classes', () => {
    const wrapper = mount(ListFilterField, {
      props: { label: 'Status', layout: 'inline' },
      slots: { default: () => 'control' },
    })

    expect(wrapper.classes()).toContain('shrink-0')
    expect(wrapper.find('div:nth-child(2)').classes()).toContain('w-40')
    expect(wrapper.text()).toContain('Status')
  })

  it('uses full width behavior for inline full controls', () => {
    const wrapper = mount(ListFilterField, {
      props: { label: 'Sort', layout: 'inline', controlWidth: 'full' },
      slots: { default: () => 'control' },
    })

    expect(wrapper.classes()).toContain('w-full')
    expect(wrapper.find('div:nth-child(2)').classes()).toContain('w-full')
  })

  it('renders stacked layout classes', () => {
    const wrapper = mount(ListFilterField, {
      props: { label: 'Schedule', layout: 'stack' },
      slots: { default: () => 'control' },
    })

    expect(wrapper.classes()).toContain('space-y-2')
    expect(wrapper.find('div:nth-child(2)').classes()).toContain('w-full')
  })
})
