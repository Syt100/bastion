import { describe, expect, it, vi } from 'vitest'

import { createDebouncedTask, isAbortError } from './asyncControl'

describe('createDebouncedTask', () => {
  it('runs once for rapid schedules and can cancel pending task', () => {
    vi.useFakeTimers()
    try {
      const task = vi.fn()
      const debounced = createDebouncedTask(task, 120)

      debounced.schedule()
      debounced.schedule()
      debounced.schedule()
      expect(task).not.toHaveBeenCalled()

      vi.advanceTimersByTime(119)
      expect(task).not.toHaveBeenCalled()

      vi.advanceTimersByTime(1)
      expect(task).toHaveBeenCalledTimes(1)

      debounced.schedule()
      debounced.cancel()
      vi.runAllTimers()
      expect(task).toHaveBeenCalledTimes(1)
    } finally {
      vi.useRealTimers()
    }
  })
})

describe('isAbortError', () => {
  it('detects abort by DOMException or name field', () => {
    const abortLike = { name: 'AbortError' }
    expect(isAbortError(abortLike)).toBe(true)
    expect(isAbortError({ name: 'OtherError' })).toBe(false)

    if (typeof DOMException !== 'undefined') {
      expect(isAbortError(new DOMException('aborted', 'AbortError'))).toBe(true)
    }
  })
})
