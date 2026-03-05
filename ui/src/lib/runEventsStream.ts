import { onBeforeUnmount, ref, type Ref } from 'vue'

import type { RunEvent } from '@/stores/jobs'

export type RunEventsWsStatus = 'disconnected' | 'connecting' | 'live' | 'reconnecting' | 'error'

export type UseRunEventsStreamOptions = {
  buildUrl: (runId: string, afterSeq: number) => string
  onEvent: (event: RunEvent) => void | Promise<void>
  validateEvent?: (event: RunEvent) => boolean
}

export type RunEventsStreamController = {
  status: Ref<RunEventsWsStatus>
  reconnectAttempts: Ref<number>
  reconnectInSeconds: Ref<number | null>
  lastSeq: Ref<number>
  start: (runId: string, afterSeq?: number) => void
  stop: () => void
  reconnect: (runId?: string) => void
  setLastSeq: (seq: number) => void
}

export function reconnectDelaySeconds(attempt: number): number {
  // 1s, 2s, 4s, 8s, ... capped.
  const cappedAttempt = Math.max(0, Math.min(10, attempt))
  return Math.min(30, Math.max(1, 1 << cappedAttempt))
}

function parseWsRunEvent(payload: unknown): RunEvent | null {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) return null
  const event = payload as RunEvent
  if (typeof event.seq !== 'number' || !Number.isFinite(event.seq)) return null
  return event
}

export function useRunEventsStream(options: UseRunEventsStreamOptions): RunEventsStreamController {
  const status = ref<RunEventsWsStatus>('disconnected')
  const reconnectAttempts = ref<number>(0)
  const reconnectInSeconds = ref<number | null>(null)
  const lastSeq = ref<number>(0)

  let socket: WebSocket | null = null
  let allowReconnect = false
  let reconnectTimer: number | null = null
  let reconnectCountdownTimer: number | null = null
  let currentRunId: string | null = null

  function clearReconnectTimers(): void {
    if (reconnectTimer !== null) {
      window.clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    if (reconnectCountdownTimer !== null) {
      window.clearInterval(reconnectCountdownTimer)
      reconnectCountdownTimer = null
    }
    reconnectInSeconds.value = null
  }

  function closeSocket(nextStatus: RunEventsWsStatus = 'disconnected'): void {
    const current = socket
    socket = null
    if (current) {
      current.onopen = null
      current.onmessage = null
      current.onerror = null
      current.onclose = null
      current.close()
    }
    clearReconnectTimers()
    status.value = nextStatus
  }

  function setLastSeq(seq: number): void {
    if (!Number.isFinite(seq)) return
    lastSeq.value = Math.max(0, Math.floor(seq))
  }

  function connect(runId: string, afterSeq: number, isReconnect: boolean): void {
    closeSocket()
    status.value = isReconnect ? 'reconnecting' : 'connecting'

    const nextSocket = new WebSocket(options.buildUrl(runId, afterSeq))
    socket = nextSocket

    nextSocket.onopen = () => {
      status.value = 'live'
      reconnectAttempts.value = 0
      clearReconnectTimers()
    }

    nextSocket.onmessage = (evt: MessageEvent) => {
      let parsed: unknown
      try {
        parsed = JSON.parse(String(evt.data)) as unknown
      } catch {
        return
      }

      const event = parseWsRunEvent(parsed)
      if (!event) return
      if (options.validateEvent && !options.validateEvent(event)) return
      if (event.seq <= lastSeq.value) return

      lastSeq.value = event.seq
      void Promise.resolve(options.onEvent(event))
    }

    nextSocket.onerror = () => {
      status.value = 'error'
    }

    nextSocket.onclose = () => {
      socket = null
      if (!allowReconnect || currentRunId !== runId) {
        status.value = 'disconnected'
        return
      }

      reconnectAttempts.value += 1
      const delay = reconnectDelaySeconds(reconnectAttempts.value - 1)
      reconnectInSeconds.value = delay
      status.value = 'reconnecting'

      if (reconnectCountdownTimer !== null) window.clearInterval(reconnectCountdownTimer)
      reconnectCountdownTimer = window.setInterval(() => {
        if (reconnectInSeconds.value == null) return
        reconnectInSeconds.value = Math.max(0, reconnectInSeconds.value - 1)
      }, 1000)

      if (reconnectTimer !== null) window.clearTimeout(reconnectTimer)
      reconnectTimer = window.setTimeout(() => {
        reconnectTimer = null
        if (!allowReconnect || currentRunId !== runId) return
        connect(runId, lastSeq.value, true)
      }, delay * 1000)
    }
  }

  function start(runId: string, afterSeq = 0): void {
    currentRunId = runId
    allowReconnect = true
    reconnectAttempts.value = 0
    setLastSeq(afterSeq)
    connect(runId, lastSeq.value, false)
  }

  function stop(): void {
    allowReconnect = false
    currentRunId = null
    reconnectAttempts.value = 0
    closeSocket('disconnected')
  }

  function reconnect(runId?: string): void {
    const targetRunId = runId ?? currentRunId
    if (!targetRunId) return
    currentRunId = targetRunId
    allowReconnect = true
    reconnectAttempts.value = 0
    clearReconnectTimers()
    connect(targetRunId, lastSeq.value, false)
  }

  onBeforeUnmount(stop)

  return {
    status,
    reconnectAttempts,
    reconnectInSeconds,
    lastSeq,
    start,
    stop,
    reconnect,
    setLastSeq,
  }
}
