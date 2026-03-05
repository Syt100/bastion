// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  return {
    NModal: vue.defineComponent({
      name: 'NModal',
      props: ['show', 'title', 'style', 'contentStyle', 'maskClosable'],
      emits: ['update:show'],
      setup(props, { slots }) {
        return () => {
          if (!props.show) return vue.h('div', { 'data-stub': 'NModal-hidden' })
          return vue.h('section', { 'data-stub': 'NModal' }, [
            vue.h('header', props.title as string),
            slots.header?.(),
            slots['header-extra']?.(),
            slots.default?.(),
            slots.footer?.(),
          ])
        }
      },
    }),
  }
})

import AppModalShell from './AppModalShell.vue'

describe('AppModalShell', () => {
  it('renders body and footer slots when modal is shown', () => {
    const wrapper = mount(AppModalShell, {
      props: {
        show: true,
        title: 'Dialog title',
      },
      slots: {
        default: () => 'body-content',
        footer: () => 'footer-content',
      },
    })

    expect(wrapper.get('[data-stub="NModal"]').text()).toContain('body-content')
    expect(wrapper.get('[data-stub="NModal"]').text()).toContain('footer-content')
    expect(wrapper.find('.app-modal-shell__body').exists()).toBe(true)
    expect(wrapper.find('.app-modal-shell__footer').exists()).toBe(true)
  })

  it('hides modal contents when show is false', () => {
    const wrapper = mount(AppModalShell, {
      props: {
        show: false,
        title: 'Dialog title',
      },
      slots: {
        default: () => 'body-content',
      },
    })

    expect(wrapper.find('[data-stub="NModal"]').exists()).toBe(false)
    expect(wrapper.find('[data-stub="NModal-hidden"]').exists()).toBe(true)
  })

  it('renders optional header and header-extra slots', () => {
    const wrapper = mount(AppModalShell, {
      props: {
        show: true,
        title: 'Dialog title',
      },
      slots: {
        header: () => 'custom-header',
        'header-extra': () => 'header-actions',
        default: () => 'body-content',
      },
    })

    expect(wrapper.get('[data-stub="NModal"]').text()).toContain('custom-header')
    expect(wrapper.get('[data-stub="NModal"]').text()).toContain('header-actions')
  })

  it('supports disabling default scroll body style', () => {
    const wrapper = mount(AppModalShell, {
      props: {
        show: true,
        title: 'Dialog title',
        scrollBody: false,
      },
      slots: {
        default: () => 'body-content',
      },
    })

    expect(wrapper.find('.app-modal-shell__body').exists()).toBe(false)
    expect(wrapper.find('.app-modal-shell__body-plain').exists()).toBe(true)
  })

  it('applies viewport bounds on modal container by default', () => {
    const wrapper = mount(AppModalShell, {
      props: {
        show: true,
        title: 'Dialog title',
      },
      slots: {
        default: () => 'body-content',
      },
    })

    const modal = wrapper.getComponent({ name: 'NModal' })
    const style = modal.props('style') as Record<string, string>
    const contentStyle = modal.props('contentStyle') as Record<string, string>

    expect(style.maxHeight).toBe('calc(100vh - 64px)')
    expect(contentStyle.maxHeight).toBeUndefined()
  })

  it('merges container style into modal container style while keeping content style optional', () => {
    const wrapper = mount(AppModalShell, {
      props: {
        show: true,
        title: 'Dialog title',
        style: { height: '70vh' },
        containerStyle: { maxHeight: '90vh' },
      },
      slots: {
        default: () => 'body-content',
      },
    })

    const modal = wrapper.getComponent({ name: 'NModal' })
    const style = modal.props('style') as Record<string, string>

    expect(style.height).toBe('70vh')
    expect(style.maxHeight).toBe('90vh')
  })
})
