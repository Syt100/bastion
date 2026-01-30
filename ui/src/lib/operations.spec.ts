import { describe, expect, it } from 'vitest'

import { operationKindLabel, operationStatusLabel } from './operations'

describe('operations labels', () => {
  it('falls back to raw values when translations are missing', () => {
    const t = (key: string) => key
    expect(operationKindLabel(t, 'restore')).toBe('restore')
    expect(operationStatusLabel(t, 'running')).toBe('running')
  })

  it('returns translated labels when available', () => {
    const t = (key: string) => {
      if (key === 'operations.kinds.restore') return 'Restore'
      if (key === 'operations.statuses.running') return 'Running'
      return key
    }
    expect(operationKindLabel(t, 'restore')).toBe('Restore')
    expect(operationStatusLabel(t, 'running')).toBe('Running')
  })
})

