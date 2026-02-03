<script setup lang="ts">
import { NAlert, NButton, NFormItem, NSelect } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobEditorContext } from '../context'

type Option = { label: string; value: string }

defineProps<{
  notifyModeOptions: Array<Option>
  wecomDestinationOptions: Array<Option>
  emailDestinationOptions: Array<Option>
  disabledWecomSelected: string[]
  disabledEmailSelected: string[]
}>()

const { t } = useI18n()

const { form } = useJobEditorContext()

const manageDestinationsHref = '/settings/notifications/destinations'
</script>

<template>
  <n-alert type="info" :bordered="false">
    {{ t('jobs.steps.notificationsHelp') }}
  </n-alert>

  <div class="space-y-4 app-border-subtle rounded-lg p-3 app-glass-soft">
    <n-form-item :label="t('jobs.fields.notificationsMode')">
      <div class="space-y-1 w-full">
        <n-select v-model:value="form.notifyMode" :options="notifyModeOptions" />
        <div class="text-xs app-text-muted">{{ t('jobs.fields.notificationsModeHelp') }}</div>
      </div>
    </n-form-item>

    <div class="flex justify-end">
      <n-button
        size="tiny"
        quaternary
        tag="a"
        :href="manageDestinationsHref"
        target="_blank"
        rel="noopener noreferrer"
      >
        {{ t('jobs.actions.manageNotificationDestinations') }}
      </n-button>
    </div>

    <template v-if="form.notifyMode === 'custom'">
      <n-form-item :label="t('jobs.fields.notifyWecomBots')">
        <div class="space-y-2 w-full">
          <n-select
            v-model:value="form.notifyWecomBots"
            multiple
            filterable
            :options="wecomDestinationOptions"
            :placeholder="t('jobs.fields.notifySelectPlaceholder')"
          />
          <div class="text-xs app-text-muted">{{ t('jobs.fields.notifyEmptyMeansDisable') }}</div>
          <n-alert v-if="disabledWecomSelected.length > 0" type="warning" :bordered="false">
            {{ t('jobs.fields.notifyDisabledSelected', { names: disabledWecomSelected.join(', ') }) }}
          </n-alert>
        </div>
      </n-form-item>

      <n-form-item :label="t('jobs.fields.notifyEmails')">
        <div class="space-y-2 w-full">
          <n-select
            v-model:value="form.notifyEmails"
            multiple
            filterable
            :options="emailDestinationOptions"
            :placeholder="t('jobs.fields.notifySelectPlaceholder')"
          />
          <div class="text-xs app-text-muted">{{ t('jobs.fields.notifyEmptyMeansDisable') }}</div>
          <n-alert v-if="disabledEmailSelected.length > 0" type="warning" :bordered="false">
            {{ t('jobs.fields.notifyDisabledSelected', { names: disabledEmailSelected.join(', ') }) }}
          </n-alert>
        </div>
      </n-form-item>
    </template>

    <template v-else>
      <div class="text-xs app-text-muted">{{ t('jobs.fields.notificationsInheritHelp') }}</div>
    </template>
  </div>
</template>
