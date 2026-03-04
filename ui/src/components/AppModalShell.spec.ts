// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  return {
    NModal: vue.defineComponent({
      name: 'NModal',
      props: ['show', 'title', 'style'],
      emits: ['update:show'],
      setup(props, { slots }) {
        return () => {
          if (!props.show) return vue.h('div', { 'data-stub': 'NModal-hidden' })
          return vue.h('section', { 'data-stub': 'NModal' }, [
            vue.h('header', props.title as string),
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
})
