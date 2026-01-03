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
  NModal,
  NPopconfirm,
  NSpace,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useSecretsStore, type SecretListItem } from '@/stores/secrets'
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
const secrets = useSecretsStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const editorOpen = ref<boolean>(false)
const editorLoading = ref<boolean>(false)
const editorSaving = ref<boolean>(false)
const editorError = ref<string | null>(null)
const editorFieldErrors = reactive<{ name?: string; username?: string }>({})

const form = reactive<{ name: string; username: string; password: string }>({
  name: '',
  username: '',
  password: '',
})

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

async function refresh(): Promise<void> {
  try {
    await secrets.refreshWebdav()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWebdavSecretsFailed'), error, t))
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

function openCreate(): void {
  form.name = ''
  form.username = ''
  form.password = ''
  editorError.value = null
  editorFieldErrors.name = undefined
  editorFieldErrors.username = undefined
  editorOpen.value = true
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
    const info = toApiErrorInfo(error)
    if (info?.code === 'invalid_name') editorFieldErrors.name = t('apiErrors.invalid_name')
    if (info?.code === 'invalid_username') editorFieldErrors.username = t('apiErrors.invalid_username')
    editorError.value = info?.message ?? String(error)
  } finally {
    editorSaving.value = false
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
              { size: 'small', onClick: () => void copyToClipboard(row.name) },
              { default: () => t('common.copy') },
            ),
            h(NButton, { size: 'small', onClick: () => void openEdit(row.name) }, { default: () => t('common.edit') }),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => void remove(row.name),
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

onMounted(refresh)
</script>

<template>
  <div class="space-y-6">
    <n-card class="app-card" :title="t('settings.webdav.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openCreate">{{ t('settings.webdav.new') }}</n-button>
        <n-button size="small" @click="refresh">{{ t('common.refresh') }}</n-button>
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
              <n-button size="small" @click="copyToClipboard(row.name)">{{ t('common.copy') }}</n-button>
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
  </div>
</template>
