<script setup lang="ts">
import { NButton, NCheckbox, NTag, type DropdownOption } from 'naive-ui'
import { PlayOutline } from '@vicons/ionicons5'

import AppIcon from '@/components/AppIcon.vue'
import OverflowActionsButton from '@/components/list/OverflowActionsButton.vue'
import type { JobListItem } from '@/stores/jobs'

withDefaults(
  defineProps<{
    job: JobListItem
    selected?: boolean
    selectable?: boolean
    checked?: boolean
    mobile?: boolean
    mainTriggerTestId: string
    runNowTestId: string
    openDetailsLabel: string
    archivedLabel: string
    neverRanLabel: string
    runNowLabel: string
    nodeLabel: string
    scheduleLabel: string
    latestRunStatusLabel: string | null
    latestRunStatusType: 'success' | 'error' | 'warning' | 'default' | null
    latestRunStartedAtLabel: string | null
    latestRunStartedAtTitle: string | null
    runNowLoading: boolean
    runNowDisabled: boolean
    overflowOptions: DropdownOption[]
  }>(),
  {
    selected: false,
    selectable: false,
    checked: false,
    mobile: false,
  },
)

const emit = defineEmits<{
  'main-click': []
  'run-now': []
  'overflow-select': [key: string | number]
  'update:checked': [value: boolean]
}>()
</script>

<template>
  <div
    class="app-list-row app-motion-soft"
    :class="selected ? 'bg-[var(--app-primary-soft)]' : ''"
  >
    <div class="min-w-0 flex items-start gap-2 flex-1">
      <div v-if="selectable" class="pt-0.5" @click.stop>
        <n-checkbox
          :checked="checked"
          @update:checked="(value) => emit('update:checked', value)"
        />
      </div>

      <button
        :data-testid="mainTriggerTestId"
        type="button"
        class="min-w-0 flex-1 text-left rounded"
        :aria-label="openDetailsLabel"
        @click="emit('main-click')"
      >
        <div class="min-w-0">
          <div class="flex items-center gap-2 min-w-0">
            <div class="font-semibold truncate">{{ job.name }}</div>
            <n-tag v-if="job.archived_at" size="small" :bordered="false" type="warning">
              {{ archivedLabel }}
            </n-tag>
          </div>
          <div class="mt-1 flex items-center gap-2 min-w-0 app-meta-text">
            <n-tag size="small" :bordered="false" :type="job.agent_id ? 'default' : 'info'">
              {{ nodeLabel }}
            </n-tag>
            <span class="min-w-0 truncate">{{ scheduleLabel }}</span>
          </div>
        </div>
      </button>
    </div>

    <div class="shrink-0 flex items-start gap-2" :class="mobile ? 'text-right' : ''">
      <div class="text-right" :class="mobile ? '' : 'min-w-[6.5rem]'">
        <n-tag
          v-if="latestRunStatusLabel"
          size="small"
          :bordered="false"
          :type="latestRunStatusType ?? 'default'"
        >
          {{ latestRunStatusLabel }}
        </n-tag>
        <n-tag v-else size="small" :bordered="false">
          {{ neverRanLabel }}
        </n-tag>

        <div
          v-if="latestRunStartedAtLabel"
          class="mt-1 app-meta-text font-mono tabular-nums max-w-[10rem] truncate"
          :title="latestRunStartedAtTitle ?? ''"
        >
          {{ latestRunStartedAtLabel }}
        </div>
      </div>

      <div class="flex items-center gap-1" @click.stop>
        <n-button
          :data-testid="runNowTestId"
          size="small"
          quaternary
          :loading="runNowLoading"
          :disabled="runNowDisabled"
          :title="runNowLabel"
          :aria-label="runNowLabel"
          class="app-motion-soft app-pressable"
          @click="emit('run-now')"
        >
          <template #icon>
            <AppIcon :component="PlayOutline" size="sm" tone="primary" />
          </template>
        </n-button>
        <OverflowActionsButton
          size="small"
          :options="overflowOptions"
          @select="(key) => emit('overflow-select', key)"
        />
      </div>
    </div>
  </div>
</template>
