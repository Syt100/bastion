<script setup lang="ts">
import { ref } from 'vue'
import { NButton, NIcon, NInput } from 'naive-ui'
import { ArrowUpOutline, RefreshOutline } from '@vicons/ionicons5'

export type PickerPathBarInputExpose = {
  focus: () => void
}

const props = withDefaults(
  defineProps<{
    value: string
    placeholder?: string
    ariaLabel?: string
    upTitle: string
    refreshTitle: string
    disabledUp?: boolean
    disabledRefresh?: boolean
  }>(),
  {
    placeholder: '',
    ariaLabel: undefined,
    disabledUp: false,
    disabledRefresh: false,
  },
)

const emit = defineEmits<{
  (e: 'update:value', value: string): void
  (e: 'up'): void
  (e: 'refresh'): void
  (e: 'enter'): void
}>()

const input = ref<InstanceType<typeof NInput> | null>(null)

function focus(): void {
  try {
    input.value?.focus?.()
  } catch {
    // ignore
  }
}

defineExpose<PickerPathBarInputExpose>({ focus })
</script>

<template>
  <n-input
    ref="input"
    :value="value"
    :placeholder="placeholder"
    :aria-label="ariaLabel || placeholder"
    @update:value="(v) => emit('update:value', v)"
    @keyup.enter="emit('enter')"
  >
    <template #prefix>
      <div class="flex items-center gap-1 -ml-1">
        <n-button
          circle
          quaternary
          size="small"
          :disabled="disabledUp"
          :title="upTitle"
          @click="emit('up')"
        >
          <template #icon>
            <n-icon class="app-picker-path-action-icon opacity-80" :size="18">
              <arrow-up-outline />
            </n-icon>
          </template>
        </n-button>
        <n-button
          circle
          quaternary
          size="small"
          :disabled="disabledRefresh"
          :title="refreshTitle"
          @click="emit('refresh')"
        >
          <template #icon>
            <n-icon class="app-picker-path-action-icon opacity-80" :size="18">
              <refresh-outline />
            </n-icon>
          </template>
        </n-button>
      </div>
    </template>
  </n-input>
</template>

<style scoped>
/* Ionicons outline icons default to a fairly heavy stroke; soften for path bar actions. */
.app-picker-path-action-icon :deep(svg) {
  stroke-width: 24;
}
</style>

