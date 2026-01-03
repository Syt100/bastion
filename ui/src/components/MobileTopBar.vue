<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NIcon } from 'naive-ui'
import { ChevronBackOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  title: string
  backTo?: string | null
}>()

const { t } = useI18n()
const router = useRouter()

const canGoBack = computed(() => typeof props.backTo === 'string' && props.backTo.length > 0)

function back(): void {
  if (!canGoBack.value || !props.backTo) return
  void router.push(props.backTo)
}
</script>

<template>
  <div
    class="h-12 px-2 rounded-lg border border-black/5 dark:border-white/10 bg-white/60 dark:bg-[#0b1220]/30 backdrop-blur"
  >
    <div class="h-full grid grid-cols-[96px_1fr_96px] items-center">
      <div class="flex items-center">
        <n-button v-if="canGoBack" quaternary size="small" @click="back">
          <template #icon>
            <n-icon><ChevronBackOutline /></n-icon>
          </template>
          {{ t('common.return') }}
        </n-button>
      </div>

      <div class="text-center text-[17px] font-semibold truncate px-2">
        {{ title }}
      </div>

      <div class="flex items-center justify-end">
        <!-- reserved actions area (intentionally empty in this version) -->
      </div>
    </div>
  </div>
</template>
