<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NButton, NCode, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { copyText } from '@/lib/clipboard'
import {
  runEventContextEntries,
  runEventDisplayMessage,
  runEventErrorChain,
  runEventErrorEnvelope,
  runEventFieldsRecord,
  runEventHint,
  runEventOperationMetadata,
  runEventPartialFailures,
} from '@/lib/run_events'
import type { RunEvent } from '@/stores/jobs'

type DetailRow = { key: string; label: string; value: string }
type PartialFailureRow = {
  resource: string
  code: string | null
  kind: string | null
  protocol: string | null
}
const ERROR_CHAIN_PREVIEW_MAX = 2
const LONG_VALUE_PREVIEW_MAX = 120
const KEY_FACT_KEYS = ['code', 'kind', 'protocol', 'retriable', 'statusCode', 'providerCode', 'providerRequestId']
const COPYABLE_DETAIL_KEYS = new Set(['code', 'providerRequestId', 'target_url'])
const MONO_VALUE_KEYS = new Set([
  'code',
  'statusCode',
  'providerCode',
  'providerRequestId',
  'disconnectCode',
  'osErrorCode',
  'operationId',
  'part_name',
  'target_url',
  'next_attempt_at',
])

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
const message = useMessage()
const rawExpanded = ref<boolean>(false)
const errorChainExpanded = ref<boolean>(false)
const expandedContextRows = ref<Record<string, boolean>>({})

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function toNumber(value: unknown): number | null {
  if (typeof value !== 'number' || !Number.isFinite(value)) return null
  return value
}

function diagnosticsRows(e: RunEvent): DetailRow[] {
  const fields = runEventFieldsRecord(e)
  const envelope = runEventErrorEnvelope(e)
  if (!envelope) return []

  const rows: DetailRow[] = [
    { key: 'code', label: t('runEvents.details.labels.code'), value: envelope.code },
    { key: 'kind', label: t('runEvents.details.labels.kind'), value: envelope.kind },
    { key: 'protocol', label: t('runEvents.details.labels.protocol'), value: envelope.transport.protocol },
    {
      key: 'retriable',
      label: t('runEvents.details.labels.retriable'),
      value: envelope.retriable.value ? t('common.yes') : t('common.no'),
    },
  ]

  if (envelope.retriable.reason) {
    rows.push({ key: 'retryReason', label: t('runEvents.details.labels.retryReason'), value: envelope.retriable.reason })
  }

  const retryAfter = envelope.retriable.retryAfterSec ?? toNumber(fields?.retry_after_secs)
  if (retryAfter != null) {
    rows.push({
      key: 'retryAfter',
      label: t('runEvents.details.labels.retryAfter'),
      value: `${Math.max(0, Math.floor(retryAfter))}${t('common.timeUnits.s')}`,
    })
  }

  if (envelope.transport.statusCode != null) {
    rows.push({ key: 'statusCode', label: t('runEvents.details.labels.statusCode'), value: String(Math.floor(envelope.transport.statusCode)) })
  }
  if (envelope.transport.statusText) {
    rows.push({ key: 'statusText', label: t('runEvents.details.labels.statusText'), value: envelope.transport.statusText })
  }
  if (envelope.transport.provider) {
    rows.push({ key: 'provider', label: t('runEvents.details.labels.provider'), value: envelope.transport.provider })
  }
  if (envelope.transport.providerCode) {
    rows.push({ key: 'providerCode', label: t('runEvents.details.labels.providerCode'), value: envelope.transport.providerCode })
  }
  if (envelope.transport.providerRequestId) {
    rows.push({ key: 'providerRequestId', label: t('runEvents.details.labels.providerRequestId'), value: envelope.transport.providerRequestId })
  }
  if (envelope.transport.disconnectCode != null) {
    rows.push({
      key: 'disconnectCode',
      label: t('runEvents.details.labels.disconnectCode'),
      value: String(Math.floor(envelope.transport.disconnectCode)),
    })
  }
  if (envelope.transport.ioKind) {
    rows.push({ key: 'ioKind', label: t('runEvents.details.labels.ioKind'), value: envelope.transport.ioKind })
  }
  if (envelope.transport.osErrorCode != null) {
    rows.push({ key: 'osErrorCode', label: t('runEvents.details.labels.osErrorCode'), value: String(Math.floor(envelope.transport.osErrorCode)) })
  }

  return rows
}

function operationRows(e: RunEvent): DetailRow[] {
  const operation = runEventOperationMetadata(e)
  if (!operation) return []

  const rows: DetailRow[] = []
  if (operation.operationId) rows.push({ key: 'operationId', label: t('runEvents.details.labels.operationId'), value: operation.operationId })

  if (operation.status) rows.push({ key: 'operationStatus', label: t('runEvents.details.labels.operationStatus'), value: operation.status })

  if (operation.pollAfterSec != null) {
    rows.push({
      key: 'pollAfter',
      label: t('runEvents.details.labels.pollAfter'),
      value: `${Math.max(0, Math.floor(operation.pollAfterSec))}${t('common.timeUnits.s')}`,
    })
  }

  return rows
}

function partialFailureRows(e: RunEvent): PartialFailureRow[] {
  return runEventPartialFailures(e)
    .map((item) => {
      return {
        resource: item.resource ?? t('runEvents.details.unknownResource'),
        code: item.code,
        kind: item.kind,
        protocol: item.protocol,
      }
    })
}

function contextLabel(key: string): string {
  const keyMap: Record<string, string> = {
    stage: t('runEvents.details.labels.stage'),
    source: t('runEvents.details.labels.source'),
    channel: t('runEvents.details.labels.channel'),
    attempt: t('runEvents.details.labels.attempt'),
    max_attempts: t('runEvents.details.labels.maxAttempts'),
    next_attempt_at: t('runEvents.details.labels.nextAttemptAt'),
    part_name: t('runEvents.details.labels.partName'),
    part_size_bytes: t('runEvents.details.labels.partSizeBytes'),
    target_url: t('runEvents.details.labels.targetUrl'),
  }
  return keyMap[key] ?? key
}

function contextValue(value: unknown): string | null {
  if (typeof value === 'string') {
    const v = value.trim()
    return v ? v : null
  }
  if (typeof value === 'number' && Number.isFinite(value)) return String(value)
  if (typeof value === 'boolean') return value ? t('common.yes') : t('common.no')
  return null
}

function contextRows(e: RunEvent): DetailRow[] {
  const priorityKeys = [
    'stage',
    'source',
    'channel',
    'attempt',
    'max_attempts',
    'next_attempt_at',
    'part_name',
    'part_size_bytes',
    'target_url',
  ]

  return runEventContextEntries(e, { priorityKeys })
    .map((item) => {
      const rowValue = contextValue(item.value)
      if (!rowValue) return null
      return { key: item.key, label: contextLabel(item.key), value: rowValue }
    })
    .filter((row): row is DetailRow => row != null)
}

function errorChainRows(e: RunEvent): string[] {
  return runEventErrorChain(e)
}

function isMonoValueRow(row: DetailRow): boolean {
  if (MONO_VALUE_KEYS.has(row.key)) return true
  if (row.value.includes('://')) return true
  return /[_\-](id|code)$/i.test(row.key)
}

function contextRowToken(row: DetailRow, index: number): string {
  return `${row.key}:${index}`
}

function contextRowIsExpandable(row: DetailRow): boolean {
  return row.value.length > LONG_VALUE_PREVIEW_MAX
}

function contextRowExpanded(row: DetailRow, index: number): boolean {
  return expandedContextRows.value[contextRowToken(row, index)] === true
}

function contextRowDisplayValue(row: DetailRow, index: number): string {
  if (!contextRowIsExpandable(row) || contextRowExpanded(row, index)) return row.value
  return `${row.value.slice(0, LONG_VALUE_PREVIEW_MAX)}…`
}

function toggleContextRow(row: DetailRow, index: number): void {
  const token = contextRowToken(row, index)
  expandedContextRows.value = {
    ...expandedContextRows.value,
    [token]: !expandedContextRows.value[token],
  }
}

function isCopyableRow(row: DetailRow): boolean {
  return COPYABLE_DETAIL_KEYS.has(row.key)
}

async function copyRowValue(value: string): Promise<void> {
  const ok = await copyText(value)
  if (ok) message.success(t('messages.copied'))
  else message.error(t('errors.copyFailed'))
}

const detailMessageText = computed(() => runEventDisplayMessage(props.event, t))
const detailHintText = computed(() => runEventHint(props.event, t, { allowGenericFallback: true }))
const detailEnvelopeRows = computed(() => diagnosticsRows(props.event))
const detailKeyFactRows = computed(() => {
  const rowsByKey = new Map(detailEnvelopeRows.value.map((row) => [row.key, row]))
  return KEY_FACT_KEYS.map((key) => rowsByKey.get(key)).filter((row): row is DetailRow => row != null)
})
const detailDiagnosticsRows = computed(() => {
  const keyFactSet = new Set(detailKeyFactRows.value.map((row) => row.key))
  return detailEnvelopeRows.value.filter((row) => !keyFactSet.has(row.key))
})
const detailContextRows = computed(() => contextRows(props.event))
const detailOperationRows = computed(() => operationRows(props.event))
const detailPartialFailures = computed(() => partialFailureRows(props.event))
const detailErrorChainRows = computed(() => errorChainRows(props.event))
const detailErrorChainHiddenCount = computed(() =>
  Math.max(0, detailErrorChainRows.value.length - ERROR_CHAIN_PREVIEW_MAX),
)
const detailVisibleErrorChainRows = computed(() =>
  errorChainExpanded.value
    ? detailErrorChainRows.value
    : detailErrorChainRows.value.slice(0, ERROR_CHAIN_PREVIEW_MAX),
)
const hasRawFields = computed(() => runEventFieldsRecord(props.event) != null)

watch(
  () => props.event.seq,
  () => {
    rawExpanded.value = false
    errorChainExpanded.value = false
    expandedContextRows.value = {}
  },
)
</script>

<template>
  <div
    class="run-event-detail-scroll text-sm space-y-2.5 min-h-0 overflow-y-auto overflow-x-hidden pr-1 pb-1"
    :style="maxBodyHeight ? { maxHeight: maxBodyHeight } : undefined"
  >
    <div class="run-event-section">
      <div class="run-event-section-title run-event-section-title--spaced">{{ t('runEvents.details.sections.summary') }}</div>
      <div class="run-event-message">{{ detailMessageText }}</div>
      <div v-if="detailHintText" class="run-event-hint">
        {{ t('runEvents.details.hintLabel') }}: {{ detailHintText }}
      </div>
    </div>
    <div v-if="detailKeyFactRows.length > 0" class="run-event-section">
      <div class="run-event-section-title run-event-section-title--spaced">{{ t('runEvents.details.sections.keyFacts') }}</div>
      <div class="run-event-key-facts-grid">
        <div
          v-for="(row, idx) in detailKeyFactRows"
          :key="`fact-${idx}`"
          class="run-event-key-fact"
        >
          <div class="run-event-key-fact-header">
            <div class="run-event-key-fact-label">{{ row.label }}</div>
            <n-button
              v-if="isCopyableRow(row)"
              size="tiny"
              text
              :data-testid="`run-event-copy-${row.key}`"
              @click="copyRowValue(row.value)"
            >
              {{ t('common.copy') }}
            </n-button>
          </div>
          <div class="run-event-key-fact-value" :class="{ 'run-event-value-mono': isMonoValueRow(row) }">{{ row.value }}</div>
        </div>
      </div>
    </div>
    <div v-if="detailDiagnosticsRows.length > 0" class="run-event-section">
      <div class="run-event-section-title run-event-section-title--spaced">{{ t('runEvents.details.sections.diagnostics') }}</div>
      <div class="run-event-kv-list">
        <template v-for="(row, idx) in detailDiagnosticsRows" :key="`diag-${idx}`">
          <div class="run-event-kv-row">
            <div class="run-event-kv-label">{{ row.label }}</div>
            <div class="run-event-kv-value" :class="{ 'run-event-value-mono': isMonoValueRow(row) }">{{ row.value }}</div>
          </div>
        </template>
      </div>
    </div>
    <div v-if="detailContextRows.length > 0" class="run-event-section">
      <div class="run-event-section-title run-event-section-title--spaced">{{ t('runEvents.details.sections.context') }}</div>
      <div class="run-event-kv-list">
        <template v-for="(row, idx) in detailContextRows" :key="`ctx-${idx}`">
          <div class="run-event-kv-row">
            <div class="run-event-kv-label">{{ row.label }}</div>
            <div class="run-event-kv-value-wrap">
              <div class="run-event-kv-value" :class="{ 'run-event-value-mono': isMonoValueRow(row) }">
                {{ contextRowDisplayValue(row, idx) }}
              </div>
              <div v-if="contextRowIsExpandable(row) || isCopyableRow(row)" class="run-event-inline-actions">
                <n-button
                  v-if="contextRowIsExpandable(row)"
                  size="tiny"
                  text
                  @click="toggleContextRow(row, idx)"
                >
                  {{ contextRowExpanded(row, idx) ? t('runEvents.details.actions.collapseValue') : t('runEvents.details.actions.expandValue') }}
                </n-button>
                <n-button
                  v-if="isCopyableRow(row)"
                  size="tiny"
                  text
                  :data-testid="`run-event-copy-${row.key}`"
                  @click="copyRowValue(row.value)"
                >
                  {{ t('common.copy') }}
                </n-button>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
    <div v-if="detailOperationRows.length > 0" class="run-event-section">
      <div class="run-event-section-title run-event-section-title--spaced">{{ t('runEvents.details.sections.operation') }}</div>
      <div class="run-event-kv-list">
        <template v-for="(row, idx) in detailOperationRows" :key="`op-${idx}`">
          <div class="run-event-kv-row">
            <div class="run-event-kv-label">{{ row.label }}</div>
            <div class="run-event-kv-value" :class="{ 'run-event-value-mono': isMonoValueRow(row) }">{{ row.value }}</div>
          </div>
        </template>
      </div>
    </div>
    <div v-if="detailPartialFailures.length > 0" class="run-event-section">
      <div class="run-event-section-title run-event-section-title--spaced">{{ t('runEvents.details.sections.partialFailures') }}</div>
      <div class="space-y-2">
        <div
          v-for="(item, idx) in detailPartialFailures"
          :key="`partial-${idx}`"
          class="run-event-item-card"
        >
          <div class="font-mono break-words">{{ item.resource }}</div>
          <div class="app-text-muted flex flex-wrap items-center gap-2 mt-1">
            <span v-if="item.code">{{ t('runEvents.details.labels.partialCode') }}: {{ item.code }}</span>
            <span v-if="item.kind">{{ t('runEvents.details.labels.partialKind') }}: {{ item.kind }}</span>
            <span v-if="item.protocol">{{ t('runEvents.details.labels.partialProtocol') }}: {{ item.protocol }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-if="detailErrorChainRows.length > 0" class="run-event-section">
      <div class="run-event-section-heading">
        <div class="run-event-section-title">{{ t('runEvents.details.sections.errorChain') }}</div>
        <n-button
          v-if="detailErrorChainRows.length > ERROR_CHAIN_PREVIEW_MAX || errorChainExpanded"
          size="tiny"
          quaternary
          data-testid="run-event-error-chain-toggle"
          @click="errorChainExpanded = !errorChainExpanded"
        >
          {{
            errorChainExpanded
              ? t('runEvents.details.actions.collapseErrorChain')
              : t('runEvents.details.actions.expandErrorChain', { count: detailErrorChainHiddenCount })
          }}
        </n-button>
      </div>
      <div class="space-y-1">
        <div
          v-for="(entry, idx) in detailVisibleErrorChainRows"
          :key="`chain-${idx}`"
          data-testid="run-event-error-chain-entry"
          class="run-event-item-card font-mono whitespace-pre-wrap break-words"
        >
          <div class="run-event-error-chain-index">#{{ idx + 1 }}</div>
          <div>{{ entry }}</div>
        </div>
      </div>
    </div>
    <div v-if="hasRawFields" class="run-event-section">
      <div class="run-event-section-heading">
        <div class="run-event-section-title">{{ t('runEvents.details.sections.rawEvent') }}</div>
        <n-button size="tiny" quaternary data-testid="run-event-raw-toggle" @click="rawExpanded = !rawExpanded">
          {{ rawExpanded ? t('runEvents.details.actions.hideRaw') : t('runEvents.details.actions.showRaw') }}
        </n-button>
      </div>
      <div
        v-show="rawExpanded"
        data-testid="run-event-raw-json"
        class="run-event-detail-json max-h-[45vh] overflow-auto rounded-md app-border-subtle p-2"
      >
        <n-code
          :code="formatJson(event.fields)"
          language="json"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.run-event-section {
  border: 1px solid var(--app-border);
  border-radius: var(--app-radius-sm);
  padding: 0.625rem;
}

.run-event-section-title {
  font-weight: 600;
  color: var(--app-text-muted);
}

.run-event-section-title--spaced {
  margin-bottom: 0.5rem;
}

.run-event-section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.run-event-message {
  font-size: 0.9375rem;
  line-height: 1.5;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  margin-bottom: 0.25rem;
}

.run-event-hint {
  border-radius: 0.375rem;
  border-left: 2px solid var(--app-warning, #d46b08);
  background: var(--app-warning-bg, #fff7e6);
  color: var(--app-warning, #d46b08);
  padding: 0.375rem 0.5rem;
}

.run-event-kv-list {
  display: flex;
  flex-direction: column;
}

.run-event-key-facts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
  gap: 0.5rem;
}

.run-event-key-fact {
  border: 1px dashed var(--app-border);
  border-radius: 0.375rem;
  padding: 0.4rem 0.5rem;
}

.run-event-key-fact-label {
  font-size: 0.75rem;
  color: var(--app-text-muted);
}

.run-event-key-fact-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.35rem;
  margin-bottom: 0.2rem;
}

.run-event-key-fact-value {
  font-weight: 600;
  line-height: 1.35;
  overflow-wrap: anywhere;
}

.run-event-kv-row {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 0.2rem 0;
}

.run-event-kv-row + .run-event-kv-row {
  border-top: 1px dashed var(--app-border);
}

.run-event-kv-label {
  width: 7rem;
  flex-shrink: 0;
  color: var(--app-text-muted);
  text-align: right;
}

.run-event-kv-value {
  min-width: 0;
  flex: 1;
  overflow-wrap: anywhere;
}

.run-event-kv-value-wrap {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.1rem;
}

.run-event-value-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
}

.run-event-inline-actions {
  display: flex;
  align-items: center;
  gap: 0.3rem;
}

.run-event-item-card {
  border-radius: 0.375rem;
  border: 1px solid var(--app-border);
  padding: 0.45rem 0.55rem;
}

.run-event-error-chain-index {
  color: var(--app-text-muted);
  font-size: 0.75rem;
  margin-bottom: 0.15rem;
}

@media (max-width: 640px) {
  .run-event-kv-label {
    width: 5.5rem;
    text-align: left;
  }
}

.run-event-detail-json :deep(pre),
.run-event-detail-json :deep(code) {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  word-break: break-word;
}
</style>
