<script setup lang="ts">
import { computed, type CSSProperties } from 'vue'
import { NButton, NDrawer, NDrawerContent, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppModalShell from '@/components/AppModalShell.vue'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'
import {
  runEventLevelTagType,
  runEventTransportMetadata,
  RUN_EVENT_DETAIL_HEADER_META_FIELDS_DEFAULT,
  type RunEventDetailHeaderMetaField,
} from '@/lib/run_events'
import { useUiStore } from '@/stores/ui'
import type { RunEvent } from '@/stores/jobs'
import RunEventDetailContent from '@/components/runs/RunEventDetailContent.vue'

type HeaderMetaToken = {
  key: RunEventDetailHeaderMetaField
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
    headerMetaFields?: readonly RunEventDetailHeaderMetaField[]
    maxBodyHeightDesktop?: string
  }>(),
  {
    headerMetaFields: () => RUN_EVENT_DETAIL_HEADER_META_FIELDS_DEFAULT,
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
  maxHeight: MODAL_HEIGHT.max,
}

function tokenForField(
  field: RunEventDetailHeaderMetaField,
  event: RunEvent,
  metadata: ReturnType<typeof runEventTransportMetadata>,
): HeaderMetaToken | null {
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
    const value = metadata.traceId
    if (!value) return null
    return {
      key: field,
      type: 'text',
      value: `${t('runEvents.details.labels.traceId')}: ${value}`,
      className: 'font-mono app-text-muted',
    }
  }

  const requestId = metadata.requestId
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
  const metadata = runEventTransportMetadata(props.event)
  return props.headerMetaFields
    .map((field) => tokenForField(field, props.event!, metadata))
    .filter((token): token is HeaderMetaToken => token != null)
})
</script>

<template>
  <AppModalShell
    v-if="isDesktop"
    :show="show"
    :width="MODAL_WIDTH.md"
    :content-style="desktopContentStyle"
    :scroll-body="false"
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
    </div>

    <template #footer>
      <n-button @click="emit('update:show', false)">{{ closeLabel }}</n-button>
    </template>
  </AppModalShell>

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
