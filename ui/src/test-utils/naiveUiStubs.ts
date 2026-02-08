import { defineComponent, h } from 'vue'

export function createNaiveStub(name: string, options?: { props?: string[]; emits?: string[] }) {
  return defineComponent({
    name,
    props: options?.props,
    emits: options?.emits,
    setup(_props, { slots, attrs }) {
      return () => h('div', { 'data-stub': name, ...attrs }, slots.default?.())
    },
  })
}

export function createNaiveButtonStub() {
  return defineComponent({
    name: 'NButton',
    setup(_props, { slots, attrs }) {
      return () =>
        h(
          'button',
          {
            'data-stub': 'NButton',
            disabled: Boolean((attrs as { disabled?: unknown }).disabled),
            onClick: (attrs as { onClick?: ((evt: MouseEvent) => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })
}

export function createNaiveInputStub() {
  return defineComponent({
    name: 'NInput',
    inheritAttrs: false,
    props: ['value', 'type', 'readonly'],
    setup(props) {
      return () => {
        const type = (props as { type?: string }).type
        const value = (props as { value?: string }).value
        const tag = type === 'textarea' ? 'textarea' : 'input'
        return h(tag, { 'data-stub': 'NInput', value: value ?? '' })
      }
    },
  })
}
