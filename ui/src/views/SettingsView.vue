<script setup lang="ts">
import { computed, h, onMounted, reactive, ref } from 'vue'
import {
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

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const secrets = useSecretsStore()

const editorOpen = ref<boolean>(false)
const editorLoading = ref<boolean>(false)
const editorSaving = ref<boolean>(false)

const wecomEditorOpen = ref<boolean>(false)
const wecomEditorLoading = ref<boolean>(false)
const wecomEditorSaving = ref<boolean>(false)

const smtpEditorOpen = ref<boolean>(false)
const smtpEditorLoading = ref<boolean>(false)
const smtpEditorSaving = ref<boolean>(false)

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

const dateFormatter = computed(
  () =>
    new Intl.DateTimeFormat(ui.locale, {
      dateStyle: 'medium',
      timeStyle: 'medium',
    }),
)

function formatUnixSeconds(ts: number | null): string {
  if (!ts) return '-'
  return dateFormatter.value.format(new Date(ts * 1000))
}

async function refresh(): Promise<void> {
  try {
    await secrets.refreshWebdav()
  } catch {
    message.error(t('errors.fetchWebdavSecretsFailed'))
  }

  try {
    await secrets.refreshWecomBots()
  } catch {
    message.error(t('errors.fetchWecomBotsFailed'))
  }

  try {
    await secrets.refreshSmtp()
  } catch {
    message.error(t('errors.fetchSmtpSecretsFailed'))
  }
}

function openCreate(): void {
  form.name = ''
  form.username = ''
  form.password = ''
  editorOpen.value = true
}

function openWecomCreate(): void {
  wecomForm.name = ''
  wecomForm.webhookUrl = ''
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
  smtpEditorOpen.value = true
}

async function openEdit(name: string): Promise<void> {
  editorOpen.value = true
  editorLoading.value = true
  try {
    const secret = await secrets.getWebdav(name)
    form.name = secret.name
    form.username = secret.username
    form.password = secret.password
  } catch {
    message.error(t('errors.fetchWebdavSecretFailed'))
    editorOpen.value = false
  } finally {
    editorLoading.value = false
  }
}

async function openWecomEdit(name: string): Promise<void> {
  wecomEditorOpen.value = true
  wecomEditorLoading.value = true
  try {
    const secret = await secrets.getWecomBot(name)
    wecomForm.name = secret.name
    wecomForm.webhookUrl = secret.webhook_url
  } catch {
    message.error(t('errors.fetchWecomBotFailed'))
    wecomEditorOpen.value = false
  } finally {
    wecomEditorLoading.value = false
  }
}

async function openSmtpEdit(name: string): Promise<void> {
  smtpEditorOpen.value = true
  smtpEditorLoading.value = true
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
  } catch {
    message.error(t('errors.fetchSmtpSecretFailed'))
    smtpEditorOpen.value = false
  } finally {
    smtpEditorLoading.value = false
  }
}

async function save(): Promise<void> {
  const name = form.name.trim()
  const username = form.username.trim()
  if (!name || !username) {
    message.error(t('errors.secretNameOrUsernameRequired'))
    return
  }

  editorSaving.value = true
  try {
    await secrets.upsertWebdav(name, username, form.password)
    message.success(t('messages.webdavSecretSaved'))
    editorOpen.value = false
    await refresh()
  } catch {
    message.error(t('errors.saveWebdavSecretFailed'))
  } finally {
    editorSaving.value = false
  }
}

async function saveWecom(): Promise<void> {
  const name = wecomForm.name.trim()
  const webhookUrl = wecomForm.webhookUrl.trim()
  if (!name || !webhookUrl) {
    message.error(t('errors.wecomNameOrWebhookRequired'))
    return
  }

  wecomEditorSaving.value = true
  try {
    await secrets.upsertWecomBot(name, webhookUrl)
    message.success(t('messages.wecomBotSaved'))
    wecomEditorOpen.value = false
    await refresh()
  } catch {
    message.error(t('errors.saveWecomBotFailed'))
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
    message.error(t('errors.smtpRequiredFields'))
    return
  }

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
  } catch {
    message.error(t('errors.saveSmtpSecretFailed'))
  } finally {
    smtpEditorSaving.value = false
  }
}

async function remove(name: string): Promise<void> {
  try {
    await secrets.deleteWebdav(name)
    message.success(t('messages.webdavSecretDeleted'))
    await refresh()
  } catch {
    message.error(t('errors.deleteWebdavSecretFailed'))
  }
}

async function removeWecom(name: string): Promise<void> {
  try {
    await secrets.deleteWecomBot(name)
    message.success(t('messages.wecomBotDeleted'))
    await refresh()
  } catch {
    message.error(t('errors.deleteWecomBotFailed'))
  }
}

async function removeSmtp(name: string): Promise<void> {
  try {
    await secrets.deleteSmtp(name)
    message.success(t('messages.smtpSecretDeleted'))
    await refresh()
  } catch {
    message.error(t('errors.deleteSmtpSecretFailed'))
  }
}

async function copyToClipboard(value: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(value)
    message.success(t('messages.copied'))
  } catch {
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
  <div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
      <div>
        <h1 class="text-xl font-semibold">{{ t('settings.title') }}</h1>
        <p class="text-sm opacity-70">{{ t('settings.subtitle') }}</p>
      </div>
      <n-space>
        <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      </n-space>
    </div>

    <n-card :title="t('settings.webdav.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openCreate">{{ t('settings.webdav.new') }}</n-button>
      </template>

      <n-data-table :loading="secrets.loadingWebdav" :columns="columns" :data="secrets.webdav" />
    </n-card>

    <n-card :title="t('settings.wecom.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openWecomCreate">{{ t('settings.wecom.new') }}</n-button>
      </template>

      <n-data-table :loading="secrets.loadingWecomBots" :columns="wecomColumns" :data="secrets.wecomBots" />
    </n-card>

    <n-card :title="t('settings.smtp.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openSmtpCreate">{{ t('settings.smtp.new') }}</n-button>
      </template>

      <n-data-table :loading="secrets.loadingSmtp" :columns="smtpColumns" :data="secrets.smtp" />
    </n-card>

    <n-modal v-model:show="editorOpen" preset="card" :title="t('settings.webdav.editorTitle')">
      <div class="space-y-4">
        <n-form label-placement="top">
          <n-form-item :label="t('settings.webdav.fields.name')">
            <n-input v-model:value="form.name" :disabled="editorLoading" />
          </n-form-item>
          <n-form-item :label="t('settings.webdav.fields.username')">
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

    <n-modal v-model:show="wecomEditorOpen" preset="card" :title="t('settings.wecom.editorTitle')">
      <div class="space-y-4">
        <n-form label-placement="top">
          <n-form-item :label="t('settings.wecom.fields.name')">
            <n-input v-model:value="wecomForm.name" :disabled="wecomEditorLoading" />
          </n-form-item>
          <n-form-item :label="t('settings.wecom.fields.webhookUrl')">
            <n-input v-model:value="wecomForm.webhookUrl" :disabled="wecomEditorLoading" />
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="wecomEditorOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="wecomEditorSaving" @click="saveWecom">{{ t('common.save') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="smtpEditorOpen" preset="card" :title="t('settings.smtp.editorTitle')">
      <div class="space-y-4">
        <n-form label-placement="top">
          <n-form-item :label="t('settings.smtp.fields.name')">
            <n-input v-model:value="smtpForm.name" :disabled="smtpEditorLoading" />
          </n-form-item>
          <n-form-item :label="t('settings.smtp.fields.host')">
            <n-input v-model:value="smtpForm.host" :disabled="smtpEditorLoading" />
          </n-form-item>
          <n-form-item :label="t('settings.smtp.fields.port')">
            <n-input-number v-model:value="smtpForm.port" :disabled="smtpEditorLoading" :min="1" :max="65535" />
          </n-form-item>
          <n-form-item :label="t('settings.smtp.fields.tls')">
            <n-select v-model:value="smtpForm.tls" :options="smtpTlsOptions" :disabled="smtpEditorLoading" />
          </n-form-item>
          <n-form-item :label="t('settings.smtp.fields.username')">
            <n-input v-model:value="smtpForm.username" :disabled="smtpEditorLoading" autocomplete="username" />
          </n-form-item>
          <n-form-item :label="t('settings.smtp.fields.password')">
            <n-input v-model:value="smtpForm.password" :disabled="smtpEditorLoading" autocomplete="current-password" />
          </n-form-item>
          <n-form-item :label="t('settings.smtp.fields.from')">
            <n-input v-model:value="smtpForm.from" :disabled="smtpEditorLoading" />
          </n-form-item>
          <n-form-item :label="t('settings.smtp.fields.to')">
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
