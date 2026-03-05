// @vitest-environment jsdom
import { defineComponent } from 'vue'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { reconnectDelaySeconds, useRunEventsStream, type RunEventsStreamController } from './runEventsStream'

class MockWebSocket {
  static instances: MockWebSocket[] = []
  url: string
  onopen: (() => void) | null = null
  onmessage: ((evt: { data: unknown }) => void) | null = null
  onerror: (() => void) | null = null
  onclose: (() => void) | null = null

  constructor(url: string) {
    this.url = url
    MockWebSocket.instances.push(this)
  }

  close(): void {
    this.onclose?.()
  }

  triggerOpen(): void {
    this.onopen?.()
  }

  triggerMessage(value: unknown): void {
    this.onmessage?.({ data: value })
  }

  triggerClose(): void {
    this.onclose?.()
  }
}

describe('runEventsStream', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    MockWebSocket.instances = []
    vi.stubGlobal('WebSocket', MockWebSocket as unknown as typeof WebSocket)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  it('dedupes seq and keeps latest sequence for next reconnect', async () => {
    const onEvent = vi.fn()
    const state: { stream: RunEventsStreamController | null } = { stream: null }
    const wrapper = mount(
      defineComponent({
        setup() {
          state.stream = useRunEventsStream({
            buildUrl: (runId, afterSeq) => `ws://test/${runId}?after_seq=${afterSeq}`,
            onEvent,
          })
          return () => null
        },
      }),
    )

    try {
      const stream = state.stream
      expect(stream).not.toBeNull()
      stream!.start('run-1', 3)
      expect(MockWebSocket.instances[0]?.url).toContain('after_seq=3')

      MockWebSocket.instances[0]!.triggerMessage(JSON.stringify({ seq: 2, ts: 1 }))
      MockWebSocket.instances[0]!.triggerMessage(JSON.stringify({ seq: 4, ts: 2 }))
      MockWebSocket.instances[0]!.triggerMessage(JSON.stringify({ seq: 4, ts: 3 }))
      await Promise.resolve()

      expect(onEvent).toHaveBeenCalledTimes(1)
      expect(stream!.lastSeq.value).toBe(4)
    } finally {
      wrapper.unmount()
    }
  })

  it('uses exponential reconnect delay and resumes from updated seq', async () => {
    const state: { stream: RunEventsStreamController | null } = { stream: null }
    const wrapper = mount(
      defineComponent({
        setup() {
          state.stream = useRunEventsStream({
            buildUrl: (runId, afterSeq) => `ws://test/${runId}?after_seq=${afterSeq}`,
            onEvent: () => undefined,
          })
          return () => null
        },
      }),
    )

    try {
      const stream = state.stream
      expect(stream).not.toBeNull()
      stream!.start('run-2', 1)

      const first = MockWebSocket.instances[0]!
      first.triggerClose()
      expect(stream!.status.value).toBe('reconnecting')
      expect(stream!.reconnectInSeconds.value).toBe(1)

      vi.advanceTimersByTime(1000)
      const second = MockWebSocket.instances[1]!
      expect(second.url).toContain('after_seq=1')

      second.triggerMessage(JSON.stringify({ seq: 7, ts: 3 }))
      await Promise.resolve()
      second.triggerClose()
      expect(stream!.reconnectInSeconds.value).toBe(2)

      vi.advanceTimersByTime(2000)
      const third = MockWebSocket.instances[2]!
      expect(third.url).toContain('after_seq=7')
    } finally {
      wrapper.unmount()
    }
  })
})

describe('reconnectDelaySeconds', () => {
  it('backs off exponentially with cap', () => {
    expect(reconnectDelaySeconds(0)).toBe(1)
    expect(reconnectDelaySeconds(1)).toBe(2)
    expect(reconnectDelaySeconds(2)).toBe(4)
    expect(reconnectDelaySeconds(10)).toBe(30)
    expect(reconnectDelaySeconds(50)).toBe(30)
  })
})
