<script setup lang="ts">
import { computed, type CSSProperties } from 'vue'
import { NButton, NDrawer, NDrawerContent, NModal, NSpace, NTag } from 'naive-ui'

import { useUnixSecondsFormatter } from '@/lib/datetime'
import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'
import { runEventLevelTagType } from '@/lib/run_events'
import { useUiStore } from '@/stores/ui'
import type { RunEvent } from '@/stores/jobs'
import RunEventDetailContent from '@/components/runs/RunEventDetailContent.vue'

const props = withDefaults(
  defineProps<{
    show: boolean
    event: RunEvent | null
    isDesktop: boolean
    title: string
    closeLabel: string
    maxBodyHeightDesktop?: string
  }>(),
  {
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
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const desktopContentStyle: CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  overflow: 'hidden',
  minHeight: '0',
}
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
        <span class="tabular-nums">{{ formatUnixSeconds(event.ts) }}</span>
        <n-tag size="small" :type="runEventLevelTagType(event.level)">{{ event.level }}</n-tag>
        <span class="app-text-muted">{{ event.kind }}</span>
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
          <span class="tabular-nums">{{ formatUnixSeconds(event.ts) }}</span>
          <n-tag size="small" :type="runEventLevelTagType(event.level)">{{ event.level }}</n-tag>
          <span class="app-text-muted">{{ event.kind }}</span>
          <slot name="header-actions" :event="event" />
        </div>
        <RunEventDetailContent class="run-event-detail-scroll run-detail-event-scroll run-events-detail-scroll" :event="event" />
      </div>
    </n-drawer-content>
  </n-drawer>
</template>
