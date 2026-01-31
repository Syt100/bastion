<script setup lang="ts">
import { NButton, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  count: number
  /**
   * Optional short hint about the selection scope, shown next to the count.
   * Example: "Only affects loaded rows"
   */
  hint?: string
}>()

const emit = defineEmits<{
  clear: []
}>()

const { t } = useI18n()
</script>

<template>
  <div
    v-if="props.count > 0"
    class="app-selection-toolbar rounded-xl px-3 py-2 flex items-center justify-between gap-2 flex-wrap"
  >
    <div class="flex items-center gap-2 min-w-0">
      <n-tag size="small" :bordered="false" type="info">
        {{ t('common.selection') }}: {{ props.count }}
      </n-tag>
      <div v-if="props.hint" class="text-xs opacity-80 truncate">
        {{ props.hint }}
      </div>
    </div>

    <div class="flex items-center gap-2 flex-wrap justify-end">
      <slot name="actions" />
      <n-button size="small" tertiary @click="emit('clear')">
        {{ t('common.clearSelection') }}
      </n-button>
    </div>
  </div>
</template>

