import { describe, expect, it, vi } from 'vitest'

const { copyTextMock } = vi.hoisted(() => ({
  copyTextMock: vi.fn(),
}))

vi.mock('@/lib/clipboard', () => ({
  copyText: copyTextMock,
}))

import { createClipboardCopyAction } from './clipboardFeedback'

describe('createClipboardCopyAction', () => {
  it('shows success message on copy success', async () => {
    copyTextMock.mockResolvedValueOnce(true)
    const t = vi.fn((key: string) => key)
    const success = vi.fn()
    const error = vi.fn()
    const copy = createClipboardCopyAction(t, { success, error })

    const ok = await copy('value')

    expect(ok).toBe(true)
    expect(success).toHaveBeenCalledWith('messages.copied')
    expect(error).not.toHaveBeenCalled()
  })

  it('shows error message on copy failure', async () => {
    copyTextMock.mockResolvedValueOnce(false)
    const t = vi.fn((key: string) => key)
    const success = vi.fn()
    const error = vi.fn()
    const copy = createClipboardCopyAction(t, { success, error })

    const ok = await copy('value')

    expect(ok).toBe(false)
    expect(error).toHaveBeenCalledWith('errors.copyFailed')
    expect(success).not.toHaveBeenCalled()
  })
})
