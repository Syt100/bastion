<script setup lang="ts">
import { NButton, NDropdown, NFormItem, NInput, NSelect, type DropdownOption } from 'naive-ui'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { useJobEditorContext } from '../context'

type Option = { label: string; value: string }

defineProps<{
  nodeOptions: Array<Option>
  jobTypeOptions: Array<Option>
  overlapOptions: Array<Option>
}>()

const { t } = useI18n()

const { form, fieldErrors, lockedNodeId, clearFieldError, onJobTypeChanged } = useJobEditorContext()

const cronPresets = computed<DropdownOption[]>(() => [
  { label: t('jobs.cronPresets.manual'), key: '__manual__' },
  { label: `${t('jobs.cronPresets.hourly')} (0 * * * *)`, key: '0 * * * *' },
  { label: `${t('jobs.cronPresets.every15m')} (*/15 * * * *)`, key: '*/15 * * * *' },
  { label: `${t('jobs.cronPresets.daily')} (0 0 * * *)`, key: '0 0 * * *' },
  { label: `${t('jobs.cronPresets.weekly')} (0 0 * * 0)`, key: '0 0 * * 0' },
  { label: `${t('jobs.cronPresets.monthly')} (0 0 1 * *)`, key: '0 0 1 * *' },
])

function applyCronPreset(key: string | number): void {
  const k = String(key)
  form.schedule = k === '__manual__' ? '' : k
  clearFieldError('schedule')
}
</script>

<template>
  <div class="space-y-4 app-border-subtle rounded-lg p-3 app-glass-soft">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <div data-field="name">
        <n-form-item
          :label="t('jobs.fields.name')"
          required
          :validation-status="fieldErrors.name ? 'error' : undefined"
          :feedback="fieldErrors.name || undefined"
        >
          <n-input v-model:value="form.name" @update:value="clearFieldError('name')" />
        </n-form-item>
      </div>
      <n-form-item :label="t('jobs.fields.node')">
        <n-select
          v-model:value="form.node"
          :options="nodeOptions"
          filterable
          :disabled="lockedNodeId !== null"
        />
      </n-form-item>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <n-form-item :label="t('jobs.fields.type')">
        <n-select v-model:value="form.jobType" :options="jobTypeOptions" @update:value="onJobTypeChanged" />
      </n-form-item>
      <n-form-item :label="t('jobs.fields.overlap')">
        <n-select v-model:value="form.overlapPolicy" :options="overlapOptions" />
      </n-form-item>
    </div>

    <div data-field="schedule">
      <n-form-item
        :label="t('jobs.fields.schedule')"
        :validation-status="fieldErrors.schedule ? 'error' : undefined"
        :feedback="fieldErrors.schedule || undefined"
      >
        <div class="space-y-2 w-full">
          <n-input
            v-model:value="form.schedule"
            :placeholder="t('jobs.fields.schedulePlaceholder')"
            @update:value="clearFieldError('schedule')"
          />
          <div class="flex flex-wrap items-center justify-between gap-2">
            <div v-if="!fieldErrors.schedule" class="text-xs opacity-70">{{ t('jobs.fields.scheduleHelp') }}</div>
            <n-dropdown :options="cronPresets" @select="applyCronPreset">
              <n-button size="tiny" secondary>{{ t('jobs.actions.cronPresets') }}</n-button>
            </n-dropdown>
          </div>
        </div>
      </n-form-item>
    </div>
  </div>
</template>
