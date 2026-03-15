import { describe, expect, it } from 'vitest'

import { runErrorLabel } from './runs'

describe('runErrorLabel', () => {
  const t = (key: string) => {
    if (key === 'runs.errorHints.run_failed') return 'The latest run did not complete successfully.'
    if (key === 'runs.errorHints.generic') return 'The latest run reported an execution error.'
    return key
  }

  it('maps known error codes to operator-facing copy', () => {
    expect(runErrorLabel(t, 'run_failed')).toBe('The latest run did not complete successfully.')
  })

  it('falls back to a generic operator-facing label for opaque error codes', () => {
    expect(runErrorLabel(t, 'worker_crashed')).toBe('The latest run reported an execution error.')
  })

  it('preserves descriptive free-form errors', () => {
    expect(runErrorLabel(t, 'socket timeout while uploading archive')).toBe('socket timeout while uploading archive')
  })
})
