<script setup lang="ts">
import { computed } from 'vue'
import { NTag, NSpace } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useAgentsStore } from '@/stores/agents'

const props = defineProps<{
  nodeId: string
}>()

const { t } = useI18n()
const agents = useAgentsStore()

const isHub = computed(() => props.nodeId === 'hub')

const agent = computed(() => {
  if (isHub.value) return null
  return agents.items.find((a) => a.id === props.nodeId) ?? null
})

const primaryLabel = computed(() => {
  if (isHub.value) return t('jobs.nodes.hub')
  return agent.value?.name?.trim() || props.nodeId
})

const statusLabel = computed(() => {
  if (isHub.value) return null
  if (!agent.value) return null
  if (agent.value.revoked) return t('agents.status.revoked')
  return agent.value.online ? t('agents.status.online') : t('agents.status.offline')
})

function statusType(): 'success' | 'warning' | 'error' | 'default' {
  const a = agent.value
  if (!a) return 'default'
  if (a.revoked) return 'error'
  return a.online ? 'success' : 'default'
}
</script>

<template>
  <n-space size="small" align="center" :wrap-item="false">
    <n-tag size="small" :bordered="false" :type="isHub ? 'info' : 'default'">
      {{ primaryLabel }}
    </n-tag>
    <n-tag v-if="statusLabel" size="small" :bordered="false" :type="statusType()">
      {{ statusLabel }}
    </n-tag>
  </n-space>
</template>

