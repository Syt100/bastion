<script setup lang="ts">
import { computed, type CSSProperties } from 'vue'
import { NButton, NDrawer, NDrawerContent, NModal, NSpace, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUnixSecondsFormatter } from '@/lib/datetime'
import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'
import { runEventLevelTagType } from '@/lib/run_events'
import { useUiStore } from '@/stores/ui'
import type { RunEvent } from '@/stores/jobs'
import RunEventDetailContent from '@/components/runs/RunEventDetailContent.vue'

type HeaderMetaField = 'timestamp' | 'level' | 'kind' | 'seq' | 'traceId' | 'requestId'
type HeaderMetaToken = {
  key: HeaderMetaField
  type: 'text' | 'level'
  value: string
  className?: string
}

const props = withDefaults(
  defineProps<{
    show: boolean
    event: RunEvent | null
    isDesktop: boolean
    title: string
    closeLabel: string
    headerMetaFields?: readonly HeaderMetaField[]
    maxBodyHeightDesktop?: string
  }>(),
  {
    headerMetaFields: () => ['timestamp', 'level', 'kind'] as HeaderMetaField[],
    maxBodyHeightDesktop: `calc(${MODAL_HEIGHT.desktopLoose} - 120px)`,
  },
)

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
}>()

defineSlots<{
  'header-actions'?: (props: { event: RunEvent }) => unknown
}>()

const ui = useUiStore()
const { t } = useI18n()
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const desktopContentStyle: CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  overflow: 'hidden',
  minHeight: '0',
}

function normalizeRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as Record<string, unknown>
}

function asNonEmptyString(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const normalized = value.trim()
  return normalized.length > 0 ? normalized : null
}

function pickNestedString(root: unknown, path: string[]): string | null {
  let current: unknown = root
  for (const key of path) {
    const record = normalizeRecord(current)
    if (!record) return null
    current = record[key]
  }
  return asNonEmptyString(current)
}

function traceIdFor(event: RunEvent): string | null {
  return (
    pickNestedString(event.fields, ['error_envelope', 'context', 'trace_id']) ??
    pickNestedString(event.fields, ['trace_id']) ??
    pickNestedString(event.fields, ['traceId'])
  )
}

function requestIdFor(event: RunEvent): string | null {
  return (
    pickNestedString(event.fields, ['error_envelope', 'transport', 'provider_request_id']) ??
    pickNestedString(event.fields, ['provider_request_id']) ??
    pickNestedString(event.fields, ['request_id'])
  )
}

function tokenForField(field: HeaderMetaField, event: RunEvent): HeaderMetaToken | null {
  if (field === 'timestamp') {
    return {
      key: field,
      type: 'text',
      value: formatUnixSeconds(event.ts),
      className: 'tabular-nums',
    }
  }
  if (field === 'level') return { key: field, type: 'level', value: event.level }
  if (field === 'kind') {
    return {
      key: field,
      type: 'text',
      value: event.kind,
      className: 'app-text-muted',
    }
  }
  if (field === 'seq') {
    return {
      key: field,
      type: 'text',
      value: `#${event.seq}`,
      className: 'font-mono app-text-muted',
    }
  }
  if (field === 'traceId') {
    const value = traceIdFor(event)
    if (!value) return null
    return {
      key: field,
      type: 'text',
      value: `${t('runEvents.details.labels.traceId')}: ${value}`,
      className: 'font-mono app-text-muted',
    }
  }

  const requestId = requestIdFor(event)
  if (!requestId) return null
  return {
    key: field,
    type: 'text',
    value: `${t('runEvents.details.labels.providerRequestId')}: ${requestId}`,
    className: 'font-mono app-text-muted',
  }
}

const headerMetaTokens = computed(() => {
  if (!props.event) return []
  return props.headerMetaFields
    .map((field) => tokenForField(field, props.event!))
    .filter((token): token is HeaderMetaToken => token != null)
})
</script>

<template>
  <n-modal
    v-if="isDesktop"
    :show="show"
    preset="card"
    :style="{ width: MODAL_WIDTH.md, maxHeight: MODAL_HEIGHT.max }"
    :content-style="desktopContentStyle"
    :title="title"
    @update:show="emit('update:show', $event)"
  >
    <div v-if="event" class="run-event-detail-modal-body run-detail-event-modal-body run-events-detail-modal-body flex h-full min-h-0 flex-col gap-3">
      <div class="text-sm app-text-muted flex shrink-0 flex-wrap items-center gap-2">
        <template v-for="item in headerMetaTokens" :key="item.key">
          <n-tag v-if="item.type === 'level'" size="small" :type="runEventLevelTagType(item.value)">{{ item.value }}</n-tag>
          <span v-else :class="item.className">{{ item.value }}</span>
        </template>
        <slot name="header-actions" :event="event" />
      </div>
      <RunEventDetailContent
        class="run-event-detail-scroll run-detail-event-scroll run-events-detail-scroll min-h-0 flex-1"
        :event="event"
        :max-body-height="maxBodyHeightDesktop"
      />
      <n-space justify="end" class="shrink-0">
        <n-button @click="emit('update:show', false)">{{ closeLabel }}</n-button>
      </n-space>
    </div>
  </n-modal>

  <n-drawer
    v-else
    :show="show"
    placement="bottom"
    height="70vh"
    @update:show="emit('update:show', $event)"
  >
    <n-drawer-content :title="title" closable>
      <div v-if="event" class="space-y-3">
        <div class="text-sm app-text-muted flex flex-wrap items-center gap-2">
          <template v-for="item in headerMetaTokens" :key="`mobile-${item.key}`">
            <n-tag v-if="item.type === 'level'" size="small" :type="runEventLevelTagType(item.value)">{{ item.value }}</n-tag>
            <span v-else :class="item.className">{{ item.value }}</span>
          </template>
          <slot name="header-actions" :event="event" />
        </div>
        <RunEventDetailContent class="run-event-detail-scroll run-detail-event-scroll run-events-detail-scroll" :event="event" />
      </div>
    </n-drawer-content>
  </n-drawer>
</template>
