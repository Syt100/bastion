<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NCard, NCode, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import { copyText } from '@/lib/clipboard'
import { useJobDetailContext } from '@/lib/jobDetailContext'

const { t } = useI18n()
const message = useMessage()

const ctx = useJobDetailContext()

const job = computed(() => ctx.job.value)

const jobJson = computed(() => {
  const j = job.value
  if (!j) return ''
  try {
    return JSON.stringify(j, null, 2)
  } catch {
    return String(j)
  }
})

async function copyJobJson(): Promise<void> {
  if (!jobJson.value) return
  const ok = await copyText(jobJson.value)
  if (ok) message.success(t('messages.copied'))
}
</script>

<template>
  <div class="space-y-3">
    <AppEmptyState v-if="!job" :title="t('common.noData')" />

    <template v-else>
      <n-card class="app-card" :title="t('common.json')">
        <div class="flex items-center justify-end gap-2 mb-3">
          <n-button size="small" @click="copyJobJson">{{ t('common.copy') }}</n-button>
        </div>
        <n-code :code="jobJson" language="json" class="text-xs" />
      </n-card>
    </template>
  </div>
</template>
