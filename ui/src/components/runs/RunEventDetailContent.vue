<script setup lang="ts">
import { computed } from 'vue'
import { NCode } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import type { RunEvent } from '@/stores/jobs'

type JsonRecord = Record<string, unknown>

type EnvelopeTextRef = {
  key: string
  params: JsonRecord
}

type ErrorEnvelope = {
  code: string
  kind: string
  retriable: {
    value: boolean
    reason: string | null
    retryAfterSec: number | null
  }
  hint: EnvelopeTextRef | null
  message: EnvelopeTextRef | null
  transport: {
    protocol: string
    statusCode: number | null
    statusText: string | null
    provider: string | null
    providerCode: string | null
    providerRequestId: string | null
    disconnectCode: number | null
    ioKind: string | null
    osErrorCode: number | null
  }
  context: JsonRecord | null
}

type DetailRow = { label: string; value: string }
type PartialFailureRow = {
  resource: string
  code: string | null
  kind: string | null
  protocol: string | null
}

const props = withDefaults(
  defineProps<{
    event: RunEvent
    maxBodyHeight?: string | null
  }>(),
  {
    maxBodyHeight: null,
  },
)

const { t } = useI18n()

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function normalizeFields(fields: unknown | null): JsonRecord | null {
  if (!fields || typeof fields !== 'object') return null
  if (Array.isArray(fields)) return null
  return fields as JsonRecord
}

function toNumber(value: unknown): number | null {
  if (typeof value !== 'number' || !Number.isFinite(value)) return null
  return value
}

function toBoolean(value: unknown): boolean | null {
  if (typeof value !== 'boolean') return null
  return value
}

function toString(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const v = value.trim()
  return v ? v : null
}

function normalizeTextRef(value: unknown): EnvelopeTextRef | null {
  const obj = normalizeFields(value)
  if (!obj) return null
  const key = toString(obj.key)
  if (!key) return null
  const params = normalizeFields(obj.params) ?? {}
  return { key, params }
}

function normalizeErrorEnvelope(fields: JsonRecord | null): ErrorEnvelope | null {
  if (!fields) return null
  const envelope = normalizeFields(fields.error_envelope)
  if (!envelope) return null

  const code = toString(envelope.code)
  const kind = toString(envelope.kind)
  const retriable = normalizeFields(envelope.retriable)
  const transport = normalizeFields(envelope.transport)
  if (!code || !kind || !retriable || !transport) return null

  const retriableValue = toBoolean(retriable.value)
  const protocol = toString(transport.protocol)
  if (retriableValue == null || !protocol) return null

  return {
    code,
    kind,
    retriable: {
      value: retriableValue,
      reason: toString(retriable.reason),
      retryAfterSec: toNumber(retriable.retry_after_sec),
    },
    hint: normalizeTextRef(envelope.hint),
    message: normalizeTextRef(envelope.message),
    transport: {
      protocol,
      statusCode: toNumber(transport.status_code),
      statusText: toString(transport.status_text),
      provider: toString(transport.provider),
      providerCode: toString(transport.provider_code),
      providerRequestId: toString(transport.provider_request_id),
      disconnectCode: toNumber(transport.disconnect_code),
      ioKind: toString(transport.io_kind),
      osErrorCode: toNumber(transport.os_error_code),
    },
    context: normalizeFields(envelope.context),
  }
}

function resolveTextRef(textRef: EnvelopeTextRef | null): string | null {
  if (!textRef) return null
  const translated = t(textRef.key, textRef.params)
  return translated === textRef.key ? null : translated
}

function eventDisplayMessage(e: RunEvent): string {
  const fields = normalizeFields(e.fields)
  const envelope = normalizeErrorEnvelope(fields)
  if (!envelope) return e.message

  const localized = resolveTextRef(envelope.message)
  if (localized) return localized
  if (toString(e.message)) return e.message
  return t('runEvents.details.genericMessage')
}

function eventHint(e: RunEvent): string | null {
  const fields = normalizeFields(e.fields)
  const legacyHint = toString(fields?.hint)
  const envelope = normalizeErrorEnvelope(fields)
  if (!envelope) return legacyHint

  const localized = resolveTextRef(envelope.hint)
  if (localized) return localized
  if (legacyHint) return legacyHint
  return t('runEvents.details.genericHint')
}

function diagnosticsRows(e: RunEvent): DetailRow[] {
  const fields = normalizeFields(e.fields)
  const envelope = normalizeErrorEnvelope(fields)
  if (!envelope) return []

  const rows: DetailRow[] = [
    { label: t('runEvents.details.labels.code'), value: envelope.code },
    { label: t('runEvents.details.labels.kind'), value: envelope.kind },
    { label: t('runEvents.details.labels.protocol'), value: envelope.transport.protocol },
    {
      label: t('runEvents.details.labels.retriable'),
      value: envelope.retriable.value ? t('common.yes') : t('common.no'),
    },
  ]

  if (envelope.retriable.reason) {
    rows.push({ label: t('runEvents.details.labels.retryReason'), value: envelope.retriable.reason })
  }

  const retryAfter = envelope.retriable.retryAfterSec ?? toNumber(fields?.retry_after_secs)
  if (retryAfter != null) {
    rows.push({
      label: t('runEvents.details.labels.retryAfter'),
      value: `${Math.max(0, Math.floor(retryAfter))}${t('common.timeUnits.s')}`,
    })
  }

  if (envelope.transport.statusCode != null) {
    rows.push({ label: t('runEvents.details.labels.statusCode'), value: String(Math.floor(envelope.transport.statusCode)) })
  }
  if (envelope.transport.statusText) {
    rows.push({ label: t('runEvents.details.labels.statusText'), value: envelope.transport.statusText })
  }
  if (envelope.transport.provider) {
    rows.push({ label: t('runEvents.details.labels.provider'), value: envelope.transport.provider })
  }
  if (envelope.transport.providerCode) {
    rows.push({ label: t('runEvents.details.labels.providerCode'), value: envelope.transport.providerCode })
  }
  if (envelope.transport.providerRequestId) {
    rows.push({ label: t('runEvents.details.labels.providerRequestId'), value: envelope.transport.providerRequestId })
  }
  if (envelope.transport.disconnectCode != null) {
    rows.push({
      label: t('runEvents.details.labels.disconnectCode'),
      value: String(Math.floor(envelope.transport.disconnectCode)),
    })
  }
  if (envelope.transport.ioKind) {
    rows.push({ label: t('runEvents.details.labels.ioKind'), value: envelope.transport.ioKind })
  }
  if (envelope.transport.osErrorCode != null) {
    rows.push({ label: t('runEvents.details.labels.osErrorCode'), value: String(Math.floor(envelope.transport.osErrorCode)) })
  }

  return rows
}

function operationRows(e: RunEvent): DetailRow[] {
  const envelope = normalizeErrorEnvelope(normalizeFields(e.fields))
  const operation = normalizeFields(envelope?.context?.operation)
  if (!operation) return []

  const rows: DetailRow[] = []
  const operationId = toString(operation.operation_id)
  if (operationId) rows.push({ label: t('runEvents.details.labels.operationId'), value: operationId })

  const status = toString(operation.status)
  if (status) rows.push({ label: t('runEvents.details.labels.operationStatus'), value: status })

  const pollAfterSec = toNumber(operation.poll_after_sec)
  if (pollAfterSec != null) {
    rows.push({
      label: t('runEvents.details.labels.pollAfter'),
      value: `${Math.max(0, Math.floor(pollAfterSec))}${t('common.timeUnits.s')}`,
    })
  }

  return rows
}

function partialFailureRows(e: RunEvent): PartialFailureRow[] {
  const envelope = normalizeErrorEnvelope(normalizeFields(e.fields))
  const partialFailures = envelope?.context?.partial_failures
  if (!Array.isArray(partialFailures)) return []

  return partialFailures
    .map((item) => {
      const row = normalizeFields(item)
      if (!row) return null
      const transport = normalizeFields(row.transport)
      const resource =
        toString(row.resource_id) ??
        toString(row.path) ??
        toString(row.resource) ??
        t('runEvents.details.unknownResource')
      return {
        resource,
        code: toString(row.code),
        kind: toString(row.kind),
        protocol: toString(transport?.protocol),
      }
    })
    .filter((item): item is PartialFailureRow => item != null)
}

const detailMessageText = computed(() => eventDisplayMessage(props.event))
const detailHintText = computed(() => eventHint(props.event))
const detailEnvelopeRows = computed(() => diagnosticsRows(props.event))
const detailOperationRows = computed(() => operationRows(props.event))
const detailPartialFailures = computed(() => partialFailureRows(props.event))
</script>

<template>
  <div
    class="run-event-detail-scroll space-y-3 min-h-0 overflow-y-auto overflow-x-hidden pr-1"
    :style="maxBodyHeight ? { maxHeight: maxBodyHeight } : undefined"
  >
    <div class="font-mono text-sm whitespace-pre-wrap break-words">{{ detailMessageText }}</div>
    <div v-if="detailHintText" class="text-xs rounded-md px-2 py-1 bg-[var(--app-warning-bg,#fff7e6)] text-[var(--app-warning,#d46b08)]">
      {{ t('runEvents.details.hintLabel') }}: {{ detailHintText }}
    </div>
    <div v-if="detailEnvelopeRows.length > 0" class="space-y-1">
      <div class="text-xs app-text-muted">{{ t('runEvents.details.sections.diagnostics') }}</div>
      <div class="grid grid-cols-[auto,1fr] gap-x-2 gap-y-1 text-xs">
        <template v-for="(row, idx) in detailEnvelopeRows" :key="`diag-${idx}`">
          <div class="app-text-muted">{{ row.label }}</div>
          <div class="font-mono break-all">{{ row.value }}</div>
        </template>
      </div>
    </div>
    <div v-if="detailOperationRows.length > 0" class="space-y-1">
      <div class="text-xs app-text-muted">{{ t('runEvents.details.sections.operation') }}</div>
      <div class="grid grid-cols-[auto,1fr] gap-x-2 gap-y-1 text-xs">
        <template v-for="(row, idx) in detailOperationRows" :key="`op-${idx}`">
          <div class="app-text-muted">{{ row.label }}</div>
          <div class="font-mono break-all">{{ row.value }}</div>
        </template>
      </div>
    </div>
    <div v-if="detailPartialFailures.length > 0" class="space-y-1">
      <div class="text-xs app-text-muted">{{ t('runEvents.details.sections.partialFailures') }}</div>
      <div class="space-y-1">
        <div
          v-for="(item, idx) in detailPartialFailures"
          :key="`partial-${idx}`"
          class="rounded border border-[color:var(--app-border)] px-2 py-1 text-xs"
        >
          <div class="font-mono break-all">{{ item.resource }}</div>
          <div class="app-text-muted flex flex-wrap items-center gap-2 mt-0.5">
            <span v-if="item.code">{{ t('runEvents.details.labels.partialCode') }}: {{ item.code }}</span>
            <span v-if="item.kind">{{ t('runEvents.details.labels.partialKind') }}: {{ item.kind }}</span>
            <span v-if="item.protocol">{{ t('runEvents.details.labels.partialProtocol') }}: {{ item.protocol }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-if="event.fields" class="run-event-detail-json max-h-[45vh] overflow-auto rounded-md app-border-subtle p-2">
      <n-code
        :code="formatJson(event.fields)"
        language="json"
      />
    </div>
  </div>
</template>

<style scoped>
.run-event-detail-json :deep(pre),
.run-event-detail-json :deep(code) {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  word-break: break-word;
}
</style>
