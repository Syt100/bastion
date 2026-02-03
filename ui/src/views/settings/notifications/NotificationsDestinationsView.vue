<script setup lang="ts">
import { computed, h, onMounted, reactive, ref } from 'vue'
import {
  NAlert,
  NButton,
  NCard,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NPopconfirm,
  NSelect,
  NSpace,
  NSwitch,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useNotificationsStore, type NotificationDestinationListItem, type NotificationChannel } from '@/stores/notifications'
import { useSecretsStore, type SmtpTlsMode } from '@/stores/secrets'
import { useUiStore } from '@/stores/ui'
import { MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { copyText } from '@/lib/clipboard'
import { formatToastError, toApiErrorInfo } from '@/lib/errors'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const notifications = useNotificationsStore()
const secrets = useSecretsStore()

const isDesktop = useMediaQuery(MQ.mdUp)
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const rowBusy = reactive<Record<string, boolean>>({})

function key(channel: NotificationChannel, name: string): string {
  return `${channel}:${name}`
}

async function refresh(): Promise<void> {
  try {
    await notifications.refreshDestinations()
  } catch (e) {
    message.error(formatToastError(t('errors.fetchNotificationDestinationsFailed'), e, t))
  }
}

async function copyToClipboard(value: string): Promise<void> {
  const ok = await copyText(value)
  if (ok) {
    message.success(t('messages.copied'))
  } else {
    message.error(t('errors.copyFailed'))
  }
}

async function toggleDestination(channel: NotificationChannel, name: string, enabled: boolean): Promise<void> {
  const k = key(channel, name)
  rowBusy[k] = true
  try {
    await notifications.setDestinationEnabled(channel, name, enabled)
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.updateNotificationDestinationFailed'), e, t))
    await refresh()
  } finally {
    rowBusy[k] = false
  }
}

async function testDestination(channel: NotificationChannel, name: string): Promise<void> {
  const k = `${key(channel, name)}:test`
  rowBusy[k] = true
  try {
    await notifications.testDestination(channel, name)
    message.success(t('messages.testNotificationSent'))
  } catch (e) {
    message.error(formatToastError(t('errors.testNotificationFailed'), e, t))
  } finally {
    rowBusy[k] = false
  }
}

const wecom = computed(() => notifications.destinations.filter((d) => d.channel === 'wecom_bot'))
const smtp = computed(() => notifications.destinations.filter((d) => d.channel === 'email'))

// ---- WeCom editor
const wecomEditorOpen = ref<boolean>(false)
const wecomEditorLoading = ref<boolean>(false)
const wecomEditorSaving = ref<boolean>(false)
const wecomEditorError = ref<string | null>(null)
const wecomFieldErrors = reactive<{ name?: string; webhookUrl?: string }>({})

const wecomForm = reactive<{ name: string; webhookUrl: string }>({
  name: '',
  webhookUrl: '',
})

function openWecomCreate(): void {
  wecomForm.name = ''
  wecomForm.webhookUrl = ''
  wecomEditorError.value = null
  wecomFieldErrors.name = undefined
  wecomFieldErrors.webhookUrl = undefined
  wecomEditorOpen.value = true
}

async function openWecomEdit(name: string): Promise<void> {
  wecomEditorOpen.value = true
  wecomEditorLoading.value = true
  wecomEditorError.value = null
  wecomFieldErrors.name = undefined
  wecomFieldErrors.webhookUrl = undefined
  try {
    const secret = await secrets.getWecomBot(name)
    wecomForm.name = secret.name
    wecomForm.webhookUrl = secret.webhook_url
  } catch (e) {
    message.error(formatToastError(t('errors.fetchWecomBotFailed'), e, t))
    wecomEditorOpen.value = false
  } finally {
    wecomEditorLoading.value = false
  }
}

async function saveWecom(): Promise<void> {
  const name = wecomForm.name.trim()
  const webhookUrl = wecomForm.webhookUrl.trim()
  if (!name || !webhookUrl) {
    wecomEditorError.value = t('errors.wecomNameOrWebhookRequired')
    wecomFieldErrors.name = !name ? t('apiErrors.invalid_name') : undefined
    wecomFieldErrors.webhookUrl = !webhookUrl ? t('apiErrors.invalid_webhook_url') : undefined
    return
  }

  wecomEditorSaving.value = true
  wecomEditorError.value = null
  wecomFieldErrors.name = undefined
  wecomFieldErrors.webhookUrl = undefined
  try {
    await secrets.upsertWecomBot(name, webhookUrl)
    message.success(t('messages.wecomBotSaved'))
    wecomEditorOpen.value = false
    await refresh()
  } catch (e) {
    const info = toApiErrorInfo(e)
    if (info?.code === 'invalid_name') wecomFieldErrors.name = t('apiErrors.invalid_name')
    if (info?.code === 'invalid_webhook_url') wecomFieldErrors.webhookUrl = t('apiErrors.invalid_webhook_url')
    wecomEditorError.value = info?.message ?? String(e)
  } finally {
    wecomEditorSaving.value = false
  }
}

async function removeWecom(name: string): Promise<void> {
  try {
    await secrets.deleteWecomBot(name)
    message.success(t('messages.wecomBotDeleted'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.deleteWecomBotFailed'), e, t))
  }
}

// ---- SMTP editor
const smtpEditorOpen = ref<boolean>(false)
const smtpEditorLoading = ref<boolean>(false)
const smtpEditorSaving = ref<boolean>(false)
const smtpEditorError = ref<string | null>(null)
const smtpFieldErrors = reactive<{
  name?: string
  host?: string
  port?: string
  username?: string
  password?: string
  from?: string
  toText?: string
}>({})

const smtpForm = reactive<{
  name: string
  host: string
  port: number
  tls: SmtpTlsMode
  username: string
  password: string
  from: string
  toText: string
}>({
  name: '',
  host: '',
  port: 587,
  tls: 'starttls',
  username: '',
  password: '',
  from: '',
  toText: '',
})

const smtpTlsOptions = computed(() => [
  { label: t('settings.smtp.tls.starttls'), value: 'starttls' as const },
  { label: t('settings.smtp.tls.implicit'), value: 'implicit' as const },
  { label: t('settings.smtp.tls.none'), value: 'none' as const },
])

function openSmtpCreate(): void {
  smtpForm.name = ''
  smtpForm.host = ''
  smtpForm.port = 587
  smtpForm.tls = 'starttls'
  smtpForm.username = ''
  smtpForm.password = ''
  smtpForm.from = ''
  smtpForm.toText = ''
  smtpEditorError.value = null
  smtpFieldErrors.name = undefined
  smtpFieldErrors.host = undefined
  smtpFieldErrors.port = undefined
  smtpFieldErrors.username = undefined
  smtpFieldErrors.password = undefined
  smtpFieldErrors.from = undefined
  smtpFieldErrors.toText = undefined
  smtpEditorOpen.value = true
}

async function openSmtpEdit(name: string): Promise<void> {
  smtpEditorOpen.value = true
  smtpEditorLoading.value = true
  smtpEditorError.value = null
  smtpFieldErrors.name = undefined
  smtpFieldErrors.host = undefined
  smtpFieldErrors.port = undefined
  smtpFieldErrors.username = undefined
  smtpFieldErrors.password = undefined
  smtpFieldErrors.from = undefined
  smtpFieldErrors.toText = undefined
  try {
    const secret = await secrets.getSmtp(name)
    smtpForm.name = secret.name
    smtpForm.host = secret.host
    smtpForm.port = secret.port
    smtpForm.tls = secret.tls
    smtpForm.username = secret.username
    smtpForm.password = secret.password
    smtpForm.from = secret.from
    smtpForm.toText = (secret.to || []).join('\n')
  } catch (e) {
    message.error(formatToastError(t('errors.fetchSmtpSecretFailed'), e, t))
    smtpEditorOpen.value = false
  } finally {
    smtpEditorLoading.value = false
  }
}

async function saveSmtp(): Promise<void> {
  const name = smtpForm.name.trim()
  const host = smtpForm.host.trim()
  const from = smtpForm.from.trim()
  const to = smtpForm.toText
    .split(/\r?\n|,/g)
    .map((x) => x.trim())
    .filter((x) => x.length > 0)

  if (!name) {
    smtpEditorError.value = t('errors.smtpNameRequired')
    smtpFieldErrors.name = t('apiErrors.invalid_name')
    return
  }
  if (!host) {
    smtpEditorError.value = t('errors.smtpHostRequired')
    smtpFieldErrors.host = t('apiErrors.invalid_host')
    return
  }
  if (!smtpForm.port || smtpForm.port <= 0) {
    smtpEditorError.value = t('errors.smtpPortRequired')
    smtpFieldErrors.port = t('apiErrors.invalid_port')
    return
  }
  if (!from) {
    smtpEditorError.value = t('errors.smtpFromRequired')
    smtpFieldErrors.from = t('apiErrors.invalid_from')
    return
  }
  if (to.length === 0) {
    smtpEditorError.value = t('errors.smtpToRequired')
    smtpFieldErrors.toText = t('apiErrors.invalid_to')
    return
  }

  smtpEditorSaving.value = true
  smtpEditorError.value = null
  smtpFieldErrors.name = undefined
  smtpFieldErrors.host = undefined
  smtpFieldErrors.port = undefined
  smtpFieldErrors.username = undefined
  smtpFieldErrors.password = undefined
  smtpFieldErrors.from = undefined
  smtpFieldErrors.toText = undefined

  try {
    await secrets.upsertSmtp(name, {
      host,
      port: smtpForm.port,
      tls: smtpForm.tls,
      username: smtpForm.username.trim(),
      password: smtpForm.password,
      from,
      to,
    })
    message.success(t('messages.smtpSecretSaved'))
    smtpEditorOpen.value = false
    await refresh()
  } catch (e) {
    const info = toApiErrorInfo(e)
    const field = info?.field
    if (field === 'name') smtpFieldErrors.name = t('apiErrors.invalid_name')
    if (field === 'host') smtpFieldErrors.host = t('apiErrors.invalid_host')
    if (field === 'port') smtpFieldErrors.port = t('apiErrors.invalid_port')
    if (field === 'from') smtpFieldErrors.from = t('apiErrors.invalid_from')
    if (field === 'to') smtpFieldErrors.toText = t('apiErrors.invalid_to')
    if (field === 'password') smtpFieldErrors.password = t('apiErrors.invalid_password')
    smtpEditorError.value = info?.message ?? String(e)
  } finally {
    smtpEditorSaving.value = false
  }
}

async function removeSmtp(name: string): Promise<void> {
  try {
    await secrets.deleteSmtp(name)
    message.success(t('messages.smtpSecretDeleted'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.deleteSmtpSecretFailed'), e, t))
  }
}

function renderEnabledSwitch(row: NotificationDestinationListItem) {
  const busy = rowBusy[key(row.channel, row.name)] === true
  return h(NSwitch, {
    value: row.enabled,
    loading: busy,
    onUpdateValue: (v: boolean) => void toggleDestination(row.channel, row.name, v),
  })
}

function renderActions(row: NotificationDestinationListItem, kind: 'wecom' | 'smtp') {
  const testBusy = rowBusy[`${key(row.channel, row.name)}:test`] === true
  return h(
    NSpace,
    { size: 8 },
    {
      default: () => [
        h(
          NButton,
          { size: 'small', onClick: () => void copyToClipboard(row.name) },
          { default: () => t('common.copy') },
        ),
        h(
          NButton,
          { size: 'small', loading: testBusy, onClick: () => void testDestination(row.channel, row.name) },
          { default: () => t('settings.notifications.test') },
        ),
        h(
          NButton,
          {
            size: 'small',
            onClick: () => {
              if (kind === 'wecom') void openWecomEdit(row.name)
              else void openSmtpEdit(row.name)
            },
          },
          { default: () => t('common.edit') },
        ),
        h(
          NPopconfirm,
          {
            onPositiveClick: () => {
              if (kind === 'wecom') void removeWecom(row.name)
              else void removeSmtp(row.name)
            },
            positiveText: t('common.delete'),
            negativeText: t('common.cancel'),
          },
          {
            trigger: () =>
              h(NButton, { size: 'small', type: 'error', tertiary: true }, { default: () => t('common.delete') }),
            default: () => (kind === 'wecom' ? t('settings.wecom.deleteConfirm') : t('settings.smtp.deleteConfirm')),
          },
        ),
      ],
    },
  )
}

const wecomColumns = computed<DataTableColumns<NotificationDestinationListItem>>(() => [
  { title: t('settings.wecom.columns.name'), key: 'name' },
  { title: t('settings.notifications.enabled'), key: 'enabled', render: (row) => renderEnabledSwitch(row) },
  {
    title: t('settings.wecom.columns.updatedAt'),
    key: 'updated_at',
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('settings.wecom.columns.actions'),
    key: 'actions',
    render: (row) => renderActions(row, 'wecom'),
  },
])

const smtpColumns = computed<DataTableColumns<NotificationDestinationListItem>>(() => [
  { title: t('settings.smtp.columns.name'), key: 'name' },
  { title: t('settings.notifications.enabled'), key: 'enabled', render: (row) => renderEnabledSwitch(row) },
  {
    title: t('settings.smtp.columns.updatedAt'),
    key: 'updated_at',
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('settings.smtp.columns.actions'),
    key: 'actions',
    render: (row) => renderActions(row, 'smtp'),
  },
])

onMounted(refresh)
</script>

<template>
  <div class="space-y-6">
    <n-card class="app-card" :bordered="false" :title="t('settings.wecom.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openWecomCreate">{{ t('settings.wecom.new') }}</n-button>
        <n-button size="small" @click="refresh">{{ t('common.refresh') }}</n-button>
      </template>

      <div v-if="!isDesktop" class="space-y-2">
        <div
          v-if="!notifications.loadingDestinations && wecom.length === 0"
          class="app-help-text px-1 py-2"
        >
          {{ t('common.noData') }}
        </div>
        <div
          v-for="row in wecom"
          :key="row.name"
          class="p-3 rounded-lg app-border-subtle app-glass-soft"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="font-medium">{{ row.name }}</div>
              <div class="text-xs app-text-muted mt-1">{{ formatUnixSeconds(row.updated_at) }}</div>
              <div class="text-xs app-text-muted mt-1">
                {{ t('settings.notifications.enabled') }}:
                <span class="font-medium">{{ row.enabled ? t('common.yes') : t('common.no') }}</span>
              </div>
            </div>
            <n-space size="small" align="center">
              <n-switch
                :value="row.enabled"
                :loading="rowBusy[key(row.channel, row.name)] === true"
                @update:value="(v) => toggleDestination(row.channel, row.name, v)"
              />
              <n-button
                size="small"
                :loading="rowBusy[`${key(row.channel, row.name)}:test`] === true"
                @click="testDestination(row.channel, row.name)"
              >
                {{ t('settings.notifications.test') }}
              </n-button>
              <n-button size="small" @click="openWecomEdit(row.name)">{{ t('common.edit') }}</n-button>
              <n-popconfirm
                :positive-text="t('common.delete')"
                :negative-text="t('common.cancel')"
                @positive-click="removeWecom(row.name)"
              >
                <template #trigger>
                  <n-button size="small" type="error" tertiary>{{ t('common.delete') }}</n-button>
                </template>
                {{ t('settings.wecom.deleteConfirm') }}
              </n-popconfirm>
            </n-space>
          </div>
        </div>
      </div>
      <div v-else class="overflow-x-auto">
        <n-data-table :loading="notifications.loadingDestinations" :columns="wecomColumns" :data="wecom" />
      </div>
    </n-card>

    <n-card class="app-card" :bordered="false" :title="t('settings.smtp.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openSmtpCreate">{{ t('settings.smtp.new') }}</n-button>
        <n-button size="small" @click="refresh">{{ t('common.refresh') }}</n-button>
      </template>

      <div v-if="!isDesktop" class="space-y-2">
        <div
          v-if="!notifications.loadingDestinations && smtp.length === 0"
          class="app-help-text px-1 py-2"
        >
          {{ t('common.noData') }}
        </div>
        <div
          v-for="row in smtp"
          :key="row.name"
          class="p-3 rounded-lg app-border-subtle app-glass-soft"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="font-medium">{{ row.name }}</div>
              <div class="text-xs app-text-muted mt-1">{{ formatUnixSeconds(row.updated_at) }}</div>
              <div class="text-xs app-text-muted mt-1">
                {{ t('settings.notifications.enabled') }}:
                <span class="font-medium">{{ row.enabled ? t('common.yes') : t('common.no') }}</span>
              </div>
            </div>
            <n-space size="small" align="center">
              <n-switch
                :value="row.enabled"
                :loading="rowBusy[key(row.channel, row.name)] === true"
                @update:value="(v) => toggleDestination(row.channel, row.name, v)"
              />
              <n-button
                size="small"
                :loading="rowBusy[`${key(row.channel, row.name)}:test`] === true"
                @click="testDestination(row.channel, row.name)"
              >
                {{ t('settings.notifications.test') }}
              </n-button>
              <n-button size="small" @click="openSmtpEdit(row.name)">{{ t('common.edit') }}</n-button>
              <n-popconfirm
                :positive-text="t('common.delete')"
                :negative-text="t('common.cancel')"
                @positive-click="removeSmtp(row.name)"
              >
                <template #trigger>
                  <n-button size="small" type="error" tertiary>{{ t('common.delete') }}</n-button>
                </template>
                {{ t('settings.smtp.deleteConfirm') }}
              </n-popconfirm>
            </n-space>
          </div>
        </div>
      </div>
      <div v-else class="overflow-x-auto">
        <n-data-table :loading="notifications.loadingDestinations" :columns="smtpColumns" :data="smtp" />
      </div>
    </n-card>

    <n-modal v-model:show="wecomEditorOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('settings.wecom.editorTitle')">
      <div class="space-y-4">
        <n-alert v-if="wecomEditorError" type="error" :bordered="false">
          {{ wecomEditorError }}
        </n-alert>

        <n-form label-placement="top">
          <n-form-item
            :label="t('settings.wecom.fields.name')"
            :validation-status="wecomFieldErrors.name ? 'error' : undefined"
            :feedback="wecomFieldErrors.name"
          >
            <n-input v-model:value="wecomForm.name" :disabled="wecomEditorLoading" />
          </n-form-item>
          <n-form-item
            :label="t('settings.wecom.fields.webhookUrl')"
            :validation-status="wecomFieldErrors.webhookUrl ? 'error' : undefined"
            :feedback="wecomFieldErrors.webhookUrl"
          >
            <n-input v-model:value="wecomForm.webhookUrl" :disabled="wecomEditorLoading" />
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="wecomEditorOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="wecomEditorSaving" @click="saveWecom">{{ t('common.save') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="smtpEditorOpen" preset="card" :style="{ width: MODAL_WIDTH.md }" :title="t('settings.smtp.editorTitle')">
      <div class="space-y-4">
        <n-alert v-if="smtpEditorError" type="error" :bordered="false">
          {{ smtpEditorError }}
        </n-alert>

        <n-form label-placement="top">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
            <n-form-item
              :label="t('settings.smtp.fields.name')"
              :validation-status="smtpFieldErrors.name ? 'error' : undefined"
              :feedback="smtpFieldErrors.name"
            >
              <n-input v-model:value="smtpForm.name" :disabled="smtpEditorLoading" />
            </n-form-item>
            <n-form-item
              :label="t('settings.smtp.fields.host')"
              :validation-status="smtpFieldErrors.host ? 'error' : undefined"
              :feedback="smtpFieldErrors.host"
            >
              <n-input v-model:value="smtpForm.host" :disabled="smtpEditorLoading" />
            </n-form-item>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
            <n-form-item
              :label="t('settings.smtp.fields.port')"
              :validation-status="smtpFieldErrors.port ? 'error' : undefined"
              :feedback="smtpFieldErrors.port"
            >
              <n-input-number v-model:value="smtpForm.port" :disabled="smtpEditorLoading" :min="1" :max="65535" class="w-full" />
            </n-form-item>
            <n-form-item :label="t('settings.smtp.fields.tls')">
              <n-select v-model:value="smtpForm.tls" :options="smtpTlsOptions" />
            </n-form-item>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
            <n-form-item :label="t('settings.smtp.fields.username')">
              <n-input v-model:value="smtpForm.username" :disabled="smtpEditorLoading" autocomplete="username" />
            </n-form-item>
            <n-form-item
              :label="t('settings.smtp.fields.password')"
              :validation-status="smtpFieldErrors.password ? 'error' : undefined"
              :feedback="smtpFieldErrors.password"
            >
              <n-input v-model:value="smtpForm.password" :disabled="smtpEditorLoading" type="password" autocomplete="current-password" />
            </n-form-item>
          </div>

          <n-form-item
            :label="t('settings.smtp.fields.from')"
            :validation-status="smtpFieldErrors.from ? 'error' : undefined"
            :feedback="smtpFieldErrors.from"
          >
            <n-input v-model:value="smtpForm.from" :disabled="smtpEditorLoading" />
          </n-form-item>
          <n-form-item
            :label="t('settings.smtp.fields.to')"
            :validation-status="smtpFieldErrors.toText ? 'error' : undefined"
            :feedback="smtpFieldErrors.toText"
          >
            <n-input v-model:value="smtpForm.toText" :disabled="smtpEditorLoading" type="textarea" :autosize="{ minRows: 3, maxRows: 8 }" />
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="smtpEditorOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="smtpEditorSaving" @click="saveSmtp">{{ t('common.save') }}</n-button>
        </n-space>
      </div>
    </n-modal>
  </div>
</template>
