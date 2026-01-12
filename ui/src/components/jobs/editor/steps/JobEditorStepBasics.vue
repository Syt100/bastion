<script setup lang="ts">
import { NFormItem, NInput, NSelect } from 'naive-ui'
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
        <div class="space-y-1 w-full">
          <n-input
            v-model:value="form.schedule"
            :placeholder="t('jobs.fields.schedulePlaceholder')"
            @update:value="clearFieldError('schedule')"
          />
          <div v-if="!fieldErrors.schedule" class="text-xs opacity-70">{{ t('jobs.fields.scheduleHelp') }}</div>
        </div>
      </n-form-item>
    </div>
  </div>
</template>
