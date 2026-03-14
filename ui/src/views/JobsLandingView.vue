<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NAlert, NButton, NCard, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import { useUiStore } from '@/stores/ui'
import { resolveRouteScope } from '@/lib/commandCenter'
import { scopeToNodeId } from '@/lib/scope'
import { nodeJobsPath } from '@/lib/nodeRoute'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const ui = useUiStore()

const effectiveScope = computed(() => resolveRouteScope(route, ui.preferredScope))
const targetNodeId = computed(() => scopeToNodeId(effectiveScope.value))
const canOpenWorkspace = computed(() => !!targetNodeId.value)
const scopeLabel = computed(() => {
  if (effectiveScope.value === 'all') return t('nav.scopePicker.all')
  if (effectiveScope.value === 'hub') return t('nav.scopePicker.hub')
  return targetNodeId.value ?? effectiveScope.value
})

function openWorkspace(): void {
  if (!targetNodeId.value) return
  void router.push(nodeJobsPath(targetNodeId.value))
}
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('jobs.title')" :subtitle="t('jobs.landing.subtitle')" />

    <n-card class="app-card console-panel" :bordered="false">
      <div class="console-kicker">{{ t('jobs.landing.kicker') }}</div>
      <div class="mt-3 text-2xl font-semibold tracking-tight">
        {{ t('jobs.landing.title') }}
      </div>
      <p class="mt-2 app-text-muted max-w-3xl">
        {{ t('jobs.landing.body') }}
      </p>

      <div class="mt-4 flex items-center gap-2 flex-wrap">
        <n-tag :bordered="false">{{ t('jobs.landing.scope', { scope: scopeLabel }) }}</n-tag>
        <n-tag v-if="targetNodeId" type="info" :bordered="false">
          {{ t('jobs.landing.node', { node: targetNodeId }) }}
        </n-tag>
      </div>

      <n-alert v-if="!canOpenWorkspace" class="mt-4" type="info" :bordered="false">
        {{ t('jobs.landing.allScopeHint') }}
      </n-alert>

      <div class="mt-5 flex items-center gap-2 flex-wrap">
        <n-button type="primary" :disabled="!canOpenWorkspace" @click="openWorkspace">
          {{ t('jobs.landing.openWorkspace') }}
        </n-button>
        <n-button quaternary @click="void router.push('/')">
          {{ t('jobs.landing.backToCommandCenter') }}
        </n-button>
      </div>
    </n-card>
  </div>
</template>
