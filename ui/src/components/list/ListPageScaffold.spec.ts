// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import ListPageScaffold from './ListPageScaffold.vue'

describe('ListPageScaffold', () => {
  it('renders all regions when slots are provided', () => {
    const wrapper = mount(ListPageScaffold, {
      slots: {
        selection: '<div id="selection">selection</div>',
        toolbar: '<div id="toolbar">toolbar</div>',
        content: '<div id="content">content</div>',
        footer: '<div id="footer">footer</div>',
      },
    })

    expect(wrapper.find('[data-testid="list-page-selection"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="list-page-toolbar"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="list-page-content"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="list-page-footer"]').exists()).toBe(true)
  })

  it('only renders required content region when optional slots are missing', () => {
    const wrapper = mount(ListPageScaffold, {
      slots: {
        content: '<div id="content">content</div>',
      },
    })

    expect(wrapper.find('[data-testid="list-page-selection"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="list-page-toolbar"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="list-page-content"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="list-page-footer"]').exists()).toBe(false)
  })
})
