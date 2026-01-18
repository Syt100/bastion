<script setup lang="ts">
import { NButton, NIcon, NInput } from 'naive-ui'
import { SearchOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

withDefaults(
  defineProps<{
    value: string
    placeholder: string
    searchDisabled?: boolean
  }>(),
  {
    searchDisabled: false,
  },
)

const emit = defineEmits<{
  (e: 'update:value', value: string): void
  (e: 'search'): void
}>()

const { t } = useI18n()
</script>

<template>
  <n-input
    :value="value"
    class="flex-1 min-w-0"
    :placeholder="placeholder"
    @update:value="(v) => emit('update:value', String(v))"
    @keyup.enter="emit('search')"
  >
    <template #suffix>
      <n-button
        size="tiny"
        quaternary
        :disabled="searchDisabled"
        :title="t('common.search')"
        :aria-label="t('common.search')"
        @click="emit('search')"
      >
        <template #icon>
          <n-icon><search-outline /></n-icon>
        </template>
      </n-button>
    </template>
  </n-input>
</template>
