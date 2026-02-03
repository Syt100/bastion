<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { NAlert, NButton, NCard, NForm, NFormItem, NSwitch, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useNotificationsStore, type NotificationsSettings } from '@/stores/notifications'
import { formatToastError, toApiErrorInfo } from '@/lib/errors'

const { t } = useI18n()
const message = useMessage()

const notifications = useNotificationsStore()

const saving = ref(false)
const error = ref<string | null>(null)

const draft = reactive<NotificationsSettings>({
  enabled: true,
  channels: { wecom_bot: { enabled: true }, email: { enabled: true } },
  templates: { wecom_markdown: '', email_subject: '', email_body: '' },
})

function loadFromStore(): void {
  if (!notifications.settings) return
  draft.enabled = notifications.settings.enabled
  draft.channels.wecom_bot.enabled = notifications.settings.channels.wecom_bot.enabled
  draft.channels.email.enabled = notifications.settings.channels.email.enabled
  draft.templates.wecom_markdown = notifications.settings.templates.wecom_markdown
  draft.templates.email_subject = notifications.settings.templates.email_subject
  draft.templates.email_body = notifications.settings.templates.email_body
}

async function refresh(): Promise<void> {
  error.value = null
  try {
    await notifications.refreshSettings()
    loadFromStore()
  } catch (e) {
    message.error(formatToastError(t('errors.fetchNotificationSettingsFailed'), e, t))
  }
}

async function save(): Promise<void> {
  saving.value = true
  error.value = null
  try {
    await notifications.saveSettings(JSON.parse(JSON.stringify(draft)) as NotificationsSettings)
    message.success(t('messages.notificationSettingsSaved'))
  } catch (e) {
    const info = toApiErrorInfo(e)
    error.value = info?.message ?? String(e)
    message.error(formatToastError(t('errors.saveNotificationSettingsFailed'), e, t))
  } finally {
    saving.value = false
  }
}

onMounted(refresh)
</script>

<template>
  <n-card class="app-card" :bordered="false" :title="t('settings.notifications.channelsTitle')">
    <div class="space-y-4">
      <n-alert v-if="error" type="error" :bordered="false">{{ error }}</n-alert>

      <n-alert v-if="!draft.enabled" type="warning" :bordered="false">
        {{ t('settings.notifications.globalDisabledHelp') }}
      </n-alert>

      <n-form label-placement="left" label-width="220">
        <n-form-item :label="t('settings.notifications.globalEnabled')">
          <n-switch v-model:value="draft.enabled" />
        </n-form-item>

        <n-form-item :label="t('settings.notifications.wecomEnabled')">
          <n-switch v-model:value="draft.channels.wecom_bot.enabled" :disabled="!draft.enabled" />
        </n-form-item>

        <n-form-item :label="t('settings.notifications.emailEnabled')">
          <n-switch v-model:value="draft.channels.email.enabled" :disabled="!draft.enabled" />
        </n-form-item>
      </n-form>

      <div class="flex items-center justify-end gap-2">
        <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
        <n-button type="primary" :loading="saving" @click="save">{{ t('common.save') }}</n-button>
      </div>
    </div>
  </n-card>
</template>
