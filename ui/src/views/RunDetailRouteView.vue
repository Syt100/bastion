<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import RunDetailPanel from '@/components/runs/RunDetailPanel.vue'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const runId = computed(() => (typeof route.params.runId === 'string' ? route.params.runId : ''))
const fromJob = computed(() => {
  const value = Array.isArray(route.query.from_job) ? route.query.from_job[0] : route.query.from_job
  return typeof value === 'string' && value.length > 0 ? value : null
})
const fromSection = computed(() => {
  const value = Array.isArray(route.query.from_section) ? route.query.from_section[0] : route.query.from_section
  return typeof value === 'string' && value.length > 0 ? value : 'history'
})
const backTo = computed(() => {
  const scope = Array.isArray(route.query.from_scope) ? route.query.from_scope[0] : route.query.from_scope
  if (fromJob.value) {
    return {
      path: `/jobs/${encodeURIComponent(fromJob.value)}/${encodeURIComponent(fromSection.value)}`,
      query: scope ? { scope } : {},
    }
  }
  return scope ? { path: '/runs', query: { scope } } : { path: '/runs' }
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('runs.detail.pageTitle')" :subtitle="t('runs.detail.pageSubtitle')">
      <n-button size="small" quaternary @click="void router.push(backTo)">
        {{ t('runs.detail.backToRuns') }}
      </n-button>
    </PageHeader>

    <RunDetailPanel :run-id="runId" />
  </div>
</template>
