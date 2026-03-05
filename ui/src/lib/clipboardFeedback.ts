import { copyText } from '@/lib/clipboard'

type Translate = (key: string, params?: Record<string, unknown>) => string
type MessageFeedback = {
  success: (message: string) => void
  error: (message: string) => void
}

export type ClipboardFeedbackOptions = {
  successKey?: string
  errorKey?: string
}

export function createClipboardCopyAction(
  t: Translate,
  message: MessageFeedback,
  defaults: ClipboardFeedbackOptions = {},
): (value: string, options?: ClipboardFeedbackOptions) => Promise<boolean> {
  return async (value: string, options: ClipboardFeedbackOptions = {}) => {
    const ok = await copyText(value)
    if (ok) {
      message.success(t(options.successKey ?? defaults.successKey ?? 'messages.copied'))
      return true
    }
    message.error(t(options.errorKey ?? defaults.errorKey ?? 'errors.copyFailed'))
    return false
  }
}
