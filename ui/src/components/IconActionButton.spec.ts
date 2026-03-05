// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  return {
    NButton: vue.defineComponent({
      name: 'NButton',
      props: ['ariaLabel', 'circle', 'quaternary', 'size'],
      setup(props, { slots, attrs }) {
        return () =>
          vue.h(
            'button',
            {
              'data-stub': 'NButton',
              'aria-label': props.ariaLabel,
              'data-circle': String(Boolean(props.circle)),
              'data-quaternary': String(Boolean(props.quaternary)),
              'data-size': props.size,
              ...attrs,
            },
            [slots.icon?.(), slots.default?.()],
          )
      },
    }),
  }
})

import IconActionButton from './IconActionButton.vue'

describe('IconActionButton', () => {
  it('passes accessible label and default visual props', () => {
    const wrapper = mount(IconActionButton, {
      props: { ariaLabel: 'Help' },
      slots: {
        default: () => '?',
      },
    })

    const button = wrapper.get('[data-stub="NButton"]')
    expect(button.attributes('aria-label')).toBe('Help')
    expect(button.attributes('data-circle')).toBe('true')
    expect(button.attributes('data-quaternary')).toBe('true')
    expect(button.attributes('data-size')).toBe('tiny')
  })
})
