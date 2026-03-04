import { describe, expect, it } from 'vitest'

import { useIdBusyState } from './idBusyState'

describe('useIdBusyState', () => {
  it('starts/stops busy flags by id', () => {
    const state = useIdBusyState<string>()

    expect(state.start('row-1')).toBe(true)
    expect(state.isBusy('row-1')).toBe(true)
    expect(state.start('row-1')).toBe(false)

    state.stop('row-1')
    expect(state.isBusy('row-1')).toBe(false)
  })

  it('clears all busy ids', () => {
    const state = useIdBusyState<number>()
    state.start(1)
    state.start(2)

    expect(state.isBusy(1)).toBe(true)
    expect(state.isBusy(2)).toBe(true)

    state.clear()
    expect(state.isBusy(1)).toBe(false)
    expect(state.isBusy(2)).toBe(false)
  })
})
