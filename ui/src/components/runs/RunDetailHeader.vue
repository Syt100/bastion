<script setup lang="ts">
import { NButton, NDropdown, NIcon, NTag } from 'naive-ui'
import { EllipsisHorizontal } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import type { RunStatus } from '@/stores/jobs'
import { runStatusLabel } from '@/lib/runs'

const props = defineProps<{
  runId: string | null
  nodeId: string
  status: RunStatus | null
  loading: boolean
  canRestore: boolean
  canVerify: boolean
}>()

const emit = defineEmits<{
  (e: 'back'): void
  (e: 'refresh'): void
  (e: 'restore'): void
  (e: 'verify'): void
  (e: 'copy-run-id'): void
}>()

const { t } = useI18n()

function statusTagType(status: RunStatus): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  if (status === 'running') return 'warning'
  return 'default'
}
</script>

<template>
  <page-header :title="t('runs.title')">
    <template #prefix>
      <NodeContextTag :node-id="nodeId" />
    </template>
    <template #subtitle>
      <div v-if="runId" class="flex items-center gap-2 text-sm opacity-70 min-w-0">
        <span class="font-mono tabular-nums truncate">{{ runId }}</span>
        <n-button size="tiny" quaternary @click="emit('copy-run-id')">{{ t('common.copy') }}</n-button>
      </div>
    </template>

    <n-tag v-if="status" size="small" :bordered="false" :type="statusTagType(status)" class="mr-2">
      {{ runStatusLabel(t, status) }}
    </n-tag>
    <n-button size="small" @click="emit('back')">{{ t('common.back') }}</n-button>
    <n-button size="small" :loading="loading" @click="emit('refresh')">{{ t('common.refresh') }}</n-button>
    <n-button size="small" type="primary" :disabled="!canRestore" @click="emit('restore')">
      {{ t('runs.actions.restore') }}
    </n-button>
    <n-dropdown
      trigger="click"
      :options="[
        {
          label: t('runs.actions.verify'),
          key: 'verify',
          disabled: !canVerify,
        },
      ]"
      @select="(key) => (key === 'verify' ? emit('verify') : null)"
    >
      <n-button size="small" quaternary>
        <template #icon>
          <n-icon :component="EllipsisHorizontal" />
        </template>
        {{ t('common.more') }}
      </n-button>
    </n-dropdown>
  </page-header>
</template>
