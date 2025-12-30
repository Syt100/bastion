<script setup lang="ts">
import { computed, h, onMounted, reactive, ref } from 'vue'
import {
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

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const secrets = useSecretsStore()

const editorOpen = ref<boolean>(false)
const editorLoading = ref<boolean>(false)
const editorSaving = ref<boolean>(false)

const form = reactive<{ name: string; username: string; password: string }>({
  name: '',
  username: '',
  password: '',
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
}

function openCreate(): void {
  form.name = ''
  form.username = ''
  form.password = ''
  editorOpen.value = true
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

async function remove(name: string): Promise<void> {
  try {
    await secrets.deleteWebdav(name)
    message.success(t('messages.webdavSecretDeleted'))
    await refresh()
  } catch {
    message.error(t('errors.deleteWebdavSecretFailed'))
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
  </div>
</template>
