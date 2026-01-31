<script setup lang="ts">
import { NButton, NDropdown, NIcon, type DropdownOption } from 'naive-ui'
import { EllipsisHorizontal } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

defineProps<{
  options: DropdownOption[]
  size?: 'tiny' | 'small' | 'medium' | 'large'
  disabled?: boolean
  showLabel?: boolean
}>()

const emit = defineEmits<{
  select: [key: string | number]
}>()

const { t } = useI18n()
</script>

<template>
  <n-dropdown :options="options" trigger="click" @select="(key) => emit('select', key)">
    <n-button
      :size="size ?? 'small'"
      tertiary
      :disabled="disabled"
      :aria-label="t('common.more')"
    >
      <template #icon>
        <n-icon><EllipsisHorizontal /></n-icon>
      </template>
      <span v-if="showLabel">{{ t('common.more') }}</span>
    </n-button>
  </n-dropdown>
</template>

