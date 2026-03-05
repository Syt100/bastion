import { nextTick, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const {
  heightPx,
  startMock,
  stopMock,
  measureMock,
  useObservedElementHeightPxMock,
} = vi.hoisted(() => {
  const heightPx = { value: 0 }
  const startMock = vi.fn()
  const stopMock = vi.fn()
  const measureMock = vi.fn()
  const useObservedElementHeightPxMock = vi.fn(() => ({
    heightPx,
    start: startMock,
    stop: stopMock,
    measure: measureMock,
  }))
  return { heightPx, startMock, stopMock, measureMock, useObservedElementHeightPxMock }
})

vi.mock('@/lib/resizeObserver', () => ({
  useObservedElementHeightPx: useObservedElementHeightPxMock,
}))

import { usePickerTableBodyMaxHeightPx } from './usePickerTableBodyMaxHeightPx'

async function flushOpenLifecycle(): Promise<void> {
  await nextTick()
  await Promise.resolve()
}

describe('usePickerTableBodyMaxHeightPx', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    heightPx.value = 0
  })

  it('starts observer on open without chained manual measure calls', async () => {
    const show = ref(false)
    const onOpen = vi.fn()

    usePickerTableBodyMaxHeightPx(show, { onOpen })

    show.value = true
    await flushOpenLifecycle()

    expect(onOpen).toHaveBeenCalledTimes(1)
    expect(startMock).toHaveBeenCalledTimes(1)
    expect(measureMock).not.toHaveBeenCalled()
  })

  it('stops observer and runs close callback when modal closes', async () => {
    const show = ref(false)
    const onClose = vi.fn()

    usePickerTableBodyMaxHeightPx(show, { onClose })

    show.value = true
    await flushOpenLifecycle()

    show.value = false
    await nextTick()

    expect(onClose).toHaveBeenCalledTimes(1)
    expect(stopMock).toHaveBeenCalledTimes(1)
  })
})
