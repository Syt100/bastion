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
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useSecretsStore, type SecretListItem, type SmtpTlsMode } from '@/stores/secrets'
import { useUiStore } from '@/stores/ui'
import PageHeader from '@/components/PageHeader.vue'
import { MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { copyText } from '@/lib/clipboard'
import { formatToastError, toApiErrorInfo } from '@/lib/errors'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const secrets = useSecretsStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const editorOpen = ref<boolean>(false)
const editorLoading = ref<boolean>(false)
const editorSaving = ref<boolean>(false)
const editorError = ref<string | null>(null)
const editorFieldErrors = reactive<{ name?: string; username?: string }>({})

const wecomEditorOpen = ref<boolean>(false)
const wecomEditorLoading = ref<boolean>(false)
const wecomEditorSaving = ref<boolean>(false)
const wecomEditorError = ref<string | null>(null)
const wecomFieldErrors = reactive<{ name?: string; webhookUrl?: string }>({})

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

const form = reactive<{ name: string; username: string; password: string }>({
  name: '',
  username: '',
  password: '',
})

const wecomForm = reactive<{ name: string; webhookUrl: string }>({
  name: '',
  webhookUrl: '',
})

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

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

async function refresh(): Promise<void> {
  try {
    await secrets.refreshWebdav()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWebdavSecretsFailed'), error, t))
  }

  try {
    await secrets.refreshWecomBots()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWecomBotsFailed'), error, t))
  }

  try {
    await secrets.refreshSmtp()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchSmtpSecretsFailed'), error, t))
  }
}

function openCreate(): void {
  form.name = ''
  form.username = ''
  form.password = ''
  editorError.value = null
  editorFieldErrors.name = undefined
  editorFieldErrors.username = undefined
  editorOpen.value = true
}

function openWecomCreate(): void {
  wecomForm.name = ''
  wecomForm.webhookUrl = ''
  wecomEditorError.value = null
  wecomFieldErrors.name = undefined
  wecomFieldErrors.webhookUrl = undefined
  wecomEditorOpen.value = true
}

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

async function openEdit(name: string): Promise<void> {
  editorOpen.value = true
  editorLoading.value = true
  editorError.value = null
  editorFieldErrors.name = undefined
  editorFieldErrors.username = undefined
  try {
    const secret = await secrets.getWebdav(name)
    form.name = secret.name
    form.username = secret.username
    form.password = secret.password
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWebdavSecretFailed'), error, t))
    editorOpen.value = false
  } finally {
    editorLoading.value = false
  }
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
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWecomBotFailed'), error, t))
    wecomEditorOpen.value = false
  } finally {
    wecomEditorLoading.value = false
  }
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
  } catch (error) {
    message.error(formatToastError(t('errors.fetchSmtpSecretFailed'), error, t))
    smtpEditorOpen.value = false
  } finally {
    smtpEditorLoading.value = false
  }
}

async function save(): Promise<void> {
  const name = form.name.trim()
  const username = form.username.trim()
  if (!name || !username) {
    editorError.value = t('errors.secretNameOrUsernameRequired')
    editorFieldErrors.name = !name ? t('apiErrors.invalid_name') : undefined
    editorFieldErrors.username = !username ? t('apiErrors.invalid_username') : undefined
    return
  }

  editorError.value = null
  editorFieldErrors.name = undefined
  editorFieldErrors.username = undefined
  editorSaving.value = true
  try {
    await secrets.upsertWebdav(name, username, form.password)
    message.success(t('messages.webdavSecretSaved'))
    editorOpen.value = false
    await refresh()
  } catch (error) {
    const info = toApiErrorInfo(error, t)
    editorError.value = info.message || t('errors.saveWebdavSecretFailed')
    if (info.field === 'name') editorFieldErrors.name = info.message
    if (info.field === 'username') editorFieldErrors.username = info.message
  } finally {
    editorSaving.value = false
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

  wecomEditorError.value = null
  wecomFieldErrors.name = undefined
  wecomFieldErrors.webhookUrl = undefined
  wecomEditorSaving.value = true
  try {
    await secrets.upsertWecomBot(name, webhookUrl)
    message.success(t('messages.wecomBotSaved'))
    wecomEditorOpen.value = false
    await refresh()
  } catch (error) {
    const info = toApiErrorInfo(error, t)
    wecomEditorError.value = info.message || t('errors.saveWecomBotFailed')
    if (info.field === 'name') wecomFieldErrors.name = info.message
    if (info.field === 'webhook_url') wecomFieldErrors.webhookUrl = info.message
  } finally {
    wecomEditorSaving.value = false
  }
}

function parseSmtpRecipients(toText: string): string[] {
  const parts = toText
    .split(/\r?\n|,/g)
    .map((s) => s.trim())
    .filter(Boolean)
  return Array.from(new Set(parts))
}

async function saveSmtp(): Promise<void> {
  const name = smtpForm.name.trim()
  const host = smtpForm.host.trim()
  const from = smtpForm.from.trim()
  const to = parseSmtpRecipients(smtpForm.toText)
  const port = Number(smtpForm.port)

  if (!name || !host || !from || to.length === 0 || !Number.isFinite(port) || port <= 0) {
    smtpEditorError.value = t('errors.smtpRequiredFields')
    smtpFieldErrors.name = !name ? t('apiErrors.invalid_name') : undefined
    smtpFieldErrors.host = !host ? t('apiErrors.invalid_host') : undefined
    smtpFieldErrors.port = !Number.isFinite(port) || port <= 0 ? t('apiErrors.invalid_port') : undefined
    smtpFieldErrors.from = !from ? t('apiErrors.invalid_from') : undefined
    smtpFieldErrors.toText = to.length === 0 ? t('apiErrors.invalid_to') : undefined
    return
  }

  smtpEditorError.value = null
  smtpFieldErrors.name = undefined
  smtpFieldErrors.host = undefined
  smtpFieldErrors.port = undefined
  smtpFieldErrors.username = undefined
  smtpFieldErrors.password = undefined
  smtpFieldErrors.from = undefined
  smtpFieldErrors.toText = undefined
  smtpEditorSaving.value = true
  try {
    await secrets.upsertSmtp(name, {
      host,
      port,
      username: smtpForm.username.trim(),
      password: smtpForm.password,
      from,
      to,
      tls: smtpForm.tls,
    })
    message.success(t('messages.smtpSecretSaved'))
    smtpEditorOpen.value = false
    await refresh()
  } catch (error) {
    const info = toApiErrorInfo(error, t)
    smtpEditorError.value = info.message || t('errors.saveSmtpSecretFailed')
    if (info.field === 'name') smtpFieldErrors.name = info.message
    if (info.field === 'host') smtpFieldErrors.host = info.message
    if (info.field === 'port') smtpFieldErrors.port = info.message
    if (info.field === 'username') smtpFieldErrors.username = info.message
    if (info.field === 'password') smtpFieldErrors.password = info.message
    if (info.field === 'from') smtpFieldErrors.from = info.message
    if (info.field === 'to') smtpFieldErrors.toText = info.message
  } finally {
    smtpEditorSaving.value = false
  }
}

async function remove(name: string): Promise<void> {
  try {
    await secrets.deleteWebdav(name)
    message.success(t('messages.webdavSecretDeleted'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.deleteWebdavSecretFailed'), error, t))
  }
}

async function removeWecom(name: string): Promise<void> {
  try {
    await secrets.deleteWecomBot(name)
    message.success(t('messages.wecomBotDeleted'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.deleteWecomBotFailed'), error, t))
  }
}

async function removeSmtp(name: string): Promise<void> {
  try {
    await secrets.deleteSmtp(name)
    message.success(t('messages.smtpSecretDeleted'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.deleteSmtpSecretFailed'), error, t))
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

const columns = computed<DataTableColumns<SecretListItem>>(() => [
  { title: t('settings.webdav.columns.name'), key: 'name' },
  {
    title: t('settings.webdav.columns.updatedAt'),
    key: 'updated_at',
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('settings.webdav.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NButton,
              { size: 'small', onClick: () => copyToClipboard(row.name) },
              { default: () => t('common.copy') },
            ),
            h(NButton, { size: 'small', onClick: () => openEdit(row.name) }, { default: () => t('common.edit') }),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => remove(row.name),
                positiveText: t('common.delete'),
                negativeText: t('common.cancel'),
              },
              {
                trigger: () =>
                  h(NButton, { size: 'small', type: 'error', tertiary: true }, { default: () => t('common.delete') }),
                default: () => t('settings.webdav.deleteConfirm'),
              },
            ),
          ],
        },
      ),
  },
])

const wecomColumns = computed<DataTableColumns<SecretListItem>>(() => [
  { title: t('settings.wecom.columns.name'), key: 'name' },
  {
    title: t('settings.wecom.columns.updatedAt'),
    key: 'updated_at',
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('settings.wecom.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NButton,
              { size: 'small', onClick: () => copyToClipboard(row.name) },
              { default: () => t('common.copy') },
            ),
            h(NButton, { size: 'small', onClick: () => openWecomEdit(row.name) }, { default: () => t('common.edit') }),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => removeWecom(row.name),
                positiveText: t('common.delete'),
                negativeText: t('common.cancel'),
              },
              {
                trigger: () =>
                  h(NButton, { size: 'small', type: 'error', tertiary: true }, { default: () => t('common.delete') }),
                default: () => t('settings.wecom.deleteConfirm'),
              },
            ),
          ],
        },
      ),
  },
])

const smtpColumns = computed<DataTableColumns<SecretListItem>>(() => [
  { title: t('settings.smtp.columns.name'), key: 'name' },
  {
    title: t('settings.smtp.columns.updatedAt'),
    key: 'updated_at',
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('settings.smtp.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NButton,
              { size: 'small', onClick: () => copyToClipboard(row.name) },
              { default: () => t('common.copy') },
            ),
            h(NButton, { size: 'small', onClick: () => openSmtpEdit(row.name) }, { default: () => t('common.edit') }),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => removeSmtp(row.name),
                positiveText: t('common.delete'),
                negativeText: t('common.cancel'),
              },
              {
                trigger: () =>
                  h(NButton, { size: 'small', type: 'error', tertiary: true }, { default: () => t('common.delete') }),
                default: () => t('settings.smtp.deleteConfirm'),
              },
            ),
          ],
        },
      ),
  },
])

const smtpTlsOptions = computed(() => [
  { label: t('settings.smtp.tls.starttls'), value: 'starttls' as const },
  { label: t('settings.smtp.tls.implicit'), value: 'implicit' as const },
  { label: t('settings.smtp.tls.none'), value: 'none' as const },
])

onMounted(refresh)
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('settings.title')" :subtitle="t('settings.subtitle')">
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
    </PageHeader>

    <n-card class="app-card" :title="t('settings.webdav.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openCreate">{{ t('settings.webdav.new') }}</n-button>
      </template>

      <div v-if="!isDesktop" class="space-y-2">
        <div
          v-if="!secrets.loadingWebdav && secrets.webdav.length === 0"
          class="text-sm opacity-70 px-1 py-2"
        >
          {{ t('common.noData') }}
        </div>
        <div
          v-for="row in secrets.webdav"
          :key="row.name"
          class="p-3 rounded-lg border border-black/5 dark:border-white/10 bg-white/60 dark:bg-[#0b1220]/30"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="font-medium">{{ row.name }}</div>
              <div class="text-xs opacity-70 mt-1">{{ formatUnixSeconds(row.updated_at) }}</div>
            </div>
            <n-space size="small">
              <n-button size="small" @click="openEdit(row.name)">{{ t('common.edit') }}</n-button>
              <n-popconfirm
                :positive-text="t('common.delete')"
                :negative-text="t('common.cancel')"
                @positive-click="remove(row.name)"
              >
                <template #trigger>
                  <n-button size="small" type="error" tertiary>{{ t('common.delete') }}</n-button>
                </template>
                {{ t('settings.webdav.deleteConfirm') }}
              </n-popconfirm>
            </n-space>
          </div>
        </div>
      </div>

      <div v-else class="overflow-x-auto">
        <n-data-table :loading="secrets.loadingWebdav" :columns="columns" :data="secrets.webdav" />
      </div>
    </n-card>

    <n-card class="app-card" :title="t('settings.wecom.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openWecomCreate">{{ t('settings.wecom.new') }}</n-button>
      </template>

      <div v-if="!isDesktop" class="space-y-2">
        <div
          v-if="!secrets.loadingWecomBots && secrets.wecomBots.length === 0"
          class="text-sm opacity-70 px-1 py-2"
        >
          {{ t('common.noData') }}
        </div>
        <div
          v-for="row in secrets.wecomBots"
          :key="row.name"
          class="p-3 rounded-lg border border-black/5 dark:border-white/10 bg-white/60 dark:bg-[#0b1220]/30"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="font-medium">{{ row.name }}</div>
              <div class="text-xs opacity-70 mt-1">{{ formatUnixSeconds(row.updated_at) }}</div>
            </div>
            <n-space size="small">
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
        <n-data-table :loading="secrets.loadingWecomBots" :columns="wecomColumns" :data="secrets.wecomBots" />
      </div>
    </n-card>

    <n-card class="app-card" :title="t('settings.smtp.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openSmtpCreate">{{ t('settings.smtp.new') }}</n-button>
      </template>

      <div v-if="!isDesktop" class="space-y-2">
        <div
          v-if="!secrets.loadingSmtp && secrets.smtp.length === 0"
          class="text-sm opacity-70 px-1 py-2"
        >
          {{ t('common.noData') }}
        </div>
        <div
          v-for="row in secrets.smtp"
          :key="row.name"
          class="p-3 rounded-lg border border-black/5 dark:border-white/10 bg-white/60 dark:bg-[#0b1220]/30"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="font-medium">{{ row.name }}</div>
              <div class="text-xs opacity-70 mt-1">{{ formatUnixSeconds(row.updated_at) }}</div>
            </div>
            <n-space size="small">
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
        <n-data-table :loading="secrets.loadingSmtp" :columns="smtpColumns" :data="secrets.smtp" />
      </div>
    </n-card>

    <n-modal v-model:show="editorOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('settings.webdav.editorTitle')">
      <div class="space-y-4">
        <n-alert v-if="editorError" type="error" :bordered="false">
          {{ editorError }}
        </n-alert>

        <n-form label-placement="top">
          <n-form-item
            :label="t('settings.webdav.fields.name')"
            :validation-status="editorFieldErrors.name ? 'error' : undefined"
            :feedback="editorFieldErrors.name"
          >
            <n-input v-model:value="form.name" :disabled="editorLoading" />
          </n-form-item>
          <n-form-item
            :label="t('settings.webdav.fields.username')"
            :validation-status="editorFieldErrors.username ? 'error' : undefined"
            :feedback="editorFieldErrors.username"
          >
            <n-input v-model:value="form.username" :disabled="editorLoading" autocomplete="username" />
          </n-form-item>
          <n-form-item :label="t('settings.webdav.fields.password')">
            <n-input v-model:value="form.password" :disabled="editorLoading" autocomplete="current-password" />
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="editorOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="editorSaving" @click="save">{{ t('common.save') }}</n-button>
        </n-space>
      </div>
    </n-modal>

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
          <n-form-item
            :label="t('settings.smtp.fields.port')"
            :validation-status="smtpFieldErrors.port ? 'error' : undefined"
            :feedback="smtpFieldErrors.port"
          >
            <n-input-number v-model:value="smtpForm.port" :disabled="smtpEditorLoading" :min="1" :max="65535" />
          </n-form-item>
          <n-form-item :label="t('settings.smtp.fields.tls')">
            <n-select v-model:value="smtpForm.tls" :options="smtpTlsOptions" :disabled="smtpEditorLoading" />
          </n-form-item>
          <n-form-item
            :label="t('settings.smtp.fields.username')"
            :validation-status="smtpFieldErrors.username ? 'error' : undefined"
            :feedback="smtpFieldErrors.username"
          >
            <n-input v-model:value="smtpForm.username" :disabled="smtpEditorLoading" autocomplete="username" />
          </n-form-item>
          <n-form-item
            :label="t('settings.smtp.fields.password')"
            :validation-status="smtpFieldErrors.password ? 'error' : undefined"
            :feedback="smtpFieldErrors.password"
          >
            <n-input v-model:value="smtpForm.password" :disabled="smtpEditorLoading" autocomplete="current-password" />
          </n-form-item>
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
            <n-input v-model:value="smtpForm.toText" type="textarea" :disabled="smtpEditorLoading" />
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
