import { onBeforeUnmount, watch, type Ref } from 'vue'

type Options = {
  onEscape: () => void
  onFocusPath?: () => void
  onBackspace?: () => void
  onEnter?: () => boolean | void
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false

  const el = target as Element
  if ((el as HTMLElement).isContentEditable) return true

  const tag = el.tagName.toLowerCase()
  if (tag === 'input' || tag === 'textarea' || tag === 'select') return true

  // Some UI components render editable controls with aria roles instead of native inputs.
  const role = el.getAttribute('role')?.toLowerCase()
  if (role === 'textbox' || role === 'combobox') return true

  return Boolean(
    el.closest?.(
      'input, textarea, select, [contenteditable], [role="textbox"], [role="combobox"]',
    ),
  )
}

export function usePickerKeyboardShortcuts(show: Ref<boolean>, options: Options): void {
  const onKeydown = (e: KeyboardEvent) => {
    if (!show.value) return
    if (e.defaultPrevented) return

    // Always allow Esc to close (or delegate to nested overlays).
    if (e.key === 'Escape') {
      e.preventDefault()
      options.onEscape()
      return
    }

    // Ctrl/Cmd+L focuses the path/prefix editor (best-effort; browsers may still capture it).
    if ((e.ctrlKey || e.metaKey) && !e.altKey && e.key.toLowerCase() === 'l') {
      e.preventDefault()
      options.onFocusPath?.()
      return
    }

    // Don't hijack keys while typing/editing.
    if (isEditableTarget(e.target)) return

    // Ignore modified key presses for the navigation shortcuts.
    if (e.ctrlKey || e.metaKey || e.altKey) return

    if (e.key === 'Backspace') {
      e.preventDefault()
      options.onBackspace?.()
      return
    }

    if (e.key === 'Enter') {
      const handled = options.onEnter?.()
      if (handled) e.preventDefault()
    }
  }

  watch(show, (open) => {
    if (open) window.addEventListener('keydown', onKeydown)
    else window.removeEventListener('keydown', onKeydown)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKeydown)
  })
}

