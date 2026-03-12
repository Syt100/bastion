import {
  runEventDisplayMessage,
  runEventErrorEnvelope,
  runEventHint,
  type RunEventTranslate,
} from '@/lib/run_events'

export type DiagnosticEventLike = {
  run_id: string
  seq: number
  ts: number
  level: string
  kind: string
  message: string
  fields: unknown | null
}

export type PreferredEventDiagnostic = {
  kind: string | null
  message: string
  hint: string | null
  copyText: string
  source: 'envelope' | 'legacy'
}

export function latestEnvelopeDiagnosticEvent<T extends DiagnosticEventLike>(events: readonly T[]): T | null {
  for (let index = events.length - 1; index >= 0; index -= 1) {
    const event = events[index]
    if (event && runEventErrorEnvelope(event)) return event
  }
  return null
}

export function envelopeEventDiagnostic<T extends DiagnosticEventLike>(
  event: T | null | undefined,
  t: RunEventTranslate,
): PreferredEventDiagnostic | null {
  if (!event) return null
  const envelope = runEventErrorEnvelope(event)
  if (!envelope) return null

  const message = runEventDisplayMessage(event, t)
  const hint = runEventHint(event, t, { allowGenericFallback: true })
  const kind = envelope.kind ?? event.kind ?? null
  const copyText = [kind, message, hint].filter((value): value is string => Boolean(value)).join('\n')

  return {
    kind,
    message,
    hint,
    copyText,
    source: 'envelope',
  }
}

export function legacyEventDiagnostic(
  kind: string | null | undefined,
  message: string | null | undefined,
): PreferredEventDiagnostic | null {
  const normalizedKind = kind?.trim() || null
  const normalizedMessage = message?.trim() || null
  if (!normalizedKind && !normalizedMessage) return null

  const copyText = [normalizedKind, normalizedMessage].filter((value): value is string => Boolean(value)).join(': ')

  return {
    kind: normalizedKind,
    message: normalizedMessage ?? normalizedKind ?? '',
    hint: null,
    copyText,
    source: 'legacy',
  }
}

export function preferredEventDiagnostic<T extends DiagnosticEventLike>(
  events: readonly T[],
  t: RunEventTranslate,
  legacyKind: string | null | undefined,
  legacyMessage: string | null | undefined,
): PreferredEventDiagnostic | null {
  return envelopeEventDiagnostic(latestEnvelopeDiagnosticEvent(events), t)
    ?? legacyEventDiagnostic(legacyKind, legacyMessage)
}
