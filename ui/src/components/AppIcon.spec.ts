// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  return {
    NIcon: vue.defineComponent({
      name: 'NIcon',
      props: ['size', 'component'],
      setup(props, { slots, attrs }) {
        return () =>
          vue.h(
            'i',
            {
              'data-stub': 'NIcon',
              'data-size': String(props.size ?? ''),
              ...attrs,
            },
            slots.default?.(),
          )
      },
    }),
  }
})

import AppIcon from './AppIcon.vue'

describe('AppIcon', () => {
  it('maps semantic size props to numeric icon sizes', () => {
    const wrapper = mount(AppIcon, {
      props: { size: 'lg' },
    })

    expect(wrapper.get('[data-stub="NIcon"]').attributes('data-size')).toBe('20')
  })

  it('applies tone class for semantic icon color', () => {
    const wrapper = mount(AppIcon, {
      props: { tone: 'warning' },
    })

    expect(wrapper.classes()).toContain('app-icon-tone-warning')
  })
})
