<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { NAlert, NButton, NCard, NForm, NFormItem, NInput, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useNotificationsStore, type NotificationsSettings } from '@/stores/notifications'
import { formatToastError, resolveApiFieldErrors, toApiErrorInfo } from '@/lib/errors'

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

const fieldErrors = reactive<{ wecomMarkdown?: string; emailSubject?: string; emailBody?: string }>({})

function clearFieldErrors(): void {
  fieldErrors.wecomMarkdown = undefined
  fieldErrors.emailSubject = undefined
  fieldErrors.emailBody = undefined
}

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
  clearFieldErrors()
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
  clearFieldErrors()
  try {
    await notifications.saveSettings(JSON.parse(JSON.stringify(draft)) as NotificationsSettings)
    message.success(t('messages.notificationTemplatesSaved'))
  } catch (e) {
    const info = toApiErrorInfo(e, t)
    const mapped = resolveApiFieldErrors(info, {
      t,
      fieldMap: {
        'templates.wecom_markdown': 'wecomMarkdown',
        'templates.email_subject': 'emailSubject',
        'templates.email_body': 'emailBody',
      },
    })
    fieldErrors.wecomMarkdown = mapped.wecomMarkdown
    fieldErrors.emailSubject = mapped.emailSubject
    fieldErrors.emailBody = mapped.emailBody
    error.value = info.message || String(e)
    message.error(formatToastError(t('errors.saveNotificationTemplatesFailed'), e, t))
  } finally {
    saving.value = false
  }
}

onMounted(refresh)
</script>

<template>
  <n-card class="app-card" :bordered="false" :title="t('settings.notifications.templatesTitle')">
    <div class="space-y-4">
      <n-alert v-if="error" type="error" :bordered="false">{{ error }}</n-alert>

      <n-alert type="info" :bordered="false">
          <div class="space-y-1">
            <div class="font-medium">{{ t('settings.notifications.templateHelpTitle') }}</div>
          <div class="text-xs app-text-muted">
            {{ t('settings.notifications.templateHelpVars') }}
          </div>
        </div>
      </n-alert>

      <n-form label-placement="top">
        <n-form-item
          :label="t('settings.notifications.wecomTemplate')"
          :validation-status="fieldErrors.wecomMarkdown ? 'error' : undefined"
          :feedback="fieldErrors.wecomMarkdown"
        >
          <n-input
            v-model:value="draft.templates.wecom_markdown"
            type="textarea"
            :autosize="{ minRows: 6, maxRows: 14 }"
          />
        </n-form-item>

        <n-form-item
          :label="t('settings.notifications.emailSubjectTemplate')"
          :validation-status="fieldErrors.emailSubject ? 'error' : undefined"
          :feedback="fieldErrors.emailSubject"
        >
          <n-input v-model:value="draft.templates.email_subject" />
        </n-form-item>

        <n-form-item
          :label="t('settings.notifications.emailBodyTemplate')"
          :validation-status="fieldErrors.emailBody ? 'error' : undefined"
          :feedback="fieldErrors.emailBody"
        >
          <n-input
            v-model:value="draft.templates.email_body"
            type="textarea"
            :autosize="{ minRows: 8, maxRows: 16 }"
          />
        </n-form-item>
      </n-form>

      <div class="flex items-center justify-end gap-2">
        <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
        <n-button type="primary" :loading="saving" @click="save">{{ t('common.save') }}</n-button>
      </div>
    </div>
  </n-card>
</template>
