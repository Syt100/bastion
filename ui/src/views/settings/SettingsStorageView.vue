<script setup lang="ts">
import { computed, h, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NAlert,
  NButton,
  NCard,
  NCheckbox,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NModal,
  NPopconfirm,
  NRadioButton,
  NRadioGroup,
  NSelect,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useAgentsStore, type AgentsLabelsMode } from '@/stores/agents'
import { useBulkOperationsStore, type WebdavDistributePreviewItem, type WebdavDistributePreviewResponse } from '@/stores/bulkOperations'
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
const route = useRoute()
const router = useRouter()

const ui = useUiStore()
const agents = useAgentsStore()
const bulkOps = useBulkOperationsStore()
const secrets = useSecretsStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : 'hub'))

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

const distributeOpen = ref<boolean>(false)
const distributeLoading = ref<boolean>(false)
const distributeSaving = ref<boolean>(false)
const distributeError = ref<string | null>(null)
const distributeSecretName = ref<string>('')
const distributeOverwrite = ref<boolean>(false)

const distributeTarget = ref<'labels' | 'node_ids'>('labels')
const distributeLabelsLoading = ref<boolean>(false)
const distributeLabelIndex = ref<{ label: string; count: number }[]>([])
const distributeLabels = ref<string[]>([])
const distributeLabelsMode = ref<AgentsLabelsMode>('and')
const distributeNodeIdsRaw = ref<string>('')

const distributePreview = ref<WebdavDistributePreviewResponse | null>(null)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const distributeLabelOptions = computed(() =>
  distributeLabelIndex.value.map((it) => ({
    label: `${it.label} (${it.count})`,
    value: it.label,
  })),
)

function parseNodeIds(raw: string): string[] {
  const parts = raw.split(/[\s,]+/g).map((v) => v.trim()).filter(Boolean)
  return Array.from(new Set(parts)).sort()
}

async function refresh(): Promise<void> {
  try {
    await secrets.refreshWebdav(nodeId.value)
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
    const secret = await secrets.getWebdav(nodeId.value, name)
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
    await secrets.upsertWebdav(nodeId.value, name, username, form.password)
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
    await secrets.deleteWebdav(nodeId.value, name)
    message.success(t('messages.webdavSecretDeleted'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.deleteWebdavSecretFailed'), error, t))
  }
}

async function refreshLabelIndex(): Promise<void> {
  distributeLabelsLoading.value = true
  try {
    distributeLabelIndex.value = await agents.listLabelIndex()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentLabelsFailed'), error, t))
  } finally {
    distributeLabelsLoading.value = false
  }
}

function openDistribute(name: string): void {
  distributeSecretName.value = name
  distributeOverwrite.value = false
  distributeTarget.value = 'labels'
  distributeLabels.value = []
  distributeLabelsMode.value = 'and'
  distributeNodeIdsRaw.value = ''
  distributePreview.value = null
  distributeError.value = null
  distributeOpen.value = true
  void refreshLabelIndex()
}

function buildSelector(): { ok: true; selector: { node_ids: string[] } | { labels: string[]; labels_mode: AgentsLabelsMode } } | { ok: false } {
  if (distributeTarget.value === 'node_ids') {
    const ids = parseNodeIds(distributeNodeIdsRaw.value)
    if (ids.length === 0) return { ok: false }
    return { ok: true, selector: { node_ids: ids } }
  }

  if (distributeLabels.value.length === 0) return { ok: false }
  return { ok: true, selector: { labels: distributeLabels.value, labels_mode: distributeLabelsMode.value } }
}

async function previewDistribute(): Promise<void> {
  const sel = buildSelector()
  if (!sel.ok) {
    distributeError.value = t('errors.formInvalid')
    return
  }

  distributeLoading.value = true
  distributeError.value = null
  try {
    distributePreview.value = await bulkOps.previewWebdavSecretDistribute({
      selector: sel.selector,
      payload: { name: distributeSecretName.value, overwrite: distributeOverwrite.value },
    })
  } catch (error) {
    distributeError.value = formatToastError(t('errors.previewBulkOperationFailed'), error, t)
  } finally {
    distributeLoading.value = false
  }
}

async function createDistributeOperation(): Promise<void> {
  const sel = buildSelector()
  if (!sel.ok) {
    distributeError.value = t('errors.formInvalid')
    return
  }
  if (!distributePreview.value) {
    distributeError.value = t('settings.webdav.distribute.previewRequired')
    return
  }

  distributeSaving.value = true
  distributeError.value = null
  try {
    const opId = await bulkOps.create({
      kind: 'webdav_secret_distribute',
      selector: sel.selector,
      payload: { name: distributeSecretName.value, overwrite: distributeOverwrite.value },
    })
    message.success(t('messages.bulkOperationCreated'))
    distributeOpen.value = false
    distributePreview.value = null
    await router.push({ path: '/settings/bulk-operations', query: { open: opId } })
  } catch (error) {
    distributeError.value = formatToastError(t('errors.createBulkOperationFailed'), error, t)
  } finally {
    distributeSaving.value = false
  }
}

const previewColumns = computed<DataTableColumns<WebdavDistributePreviewItem>>(() => [
  {
    title: t('settings.webdav.distribute.previewColumns.node'),
    key: 'agent',
    render: (row) => row.agent_name ?? row.agent_id,
  },
  {
    title: t('settings.webdav.distribute.previewColumns.action'),
    key: 'action',
    render: (row) =>
      h(
        NTag,
        { size: 'small', type: row.action === 'skip' ? 'default' : 'warning' },
        {
          default: () =>
            row.action === 'skip'
              ? t('settings.webdav.distribute.previewActions.skip')
              : t('settings.webdav.distribute.previewActions.update'),
        },
      ),
  },
  {
    title: t('settings.webdav.distribute.previewColumns.note'),
    key: 'note',
    render: (row) => {
      if (!row.note) return '-'
      if (row.note === 'already exists') return t('settings.webdav.distribute.previewNotes.alreadyExists')
      return row.note
    },
  },
])

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
            nodeId.value === 'hub'
              ? h(
                  NButton,
                  { size: 'small', onClick: () => openDistribute(row.name) },
                  { default: () => t('settings.webdav.distribute.action') },
                )
              : null,
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

watch(nodeId, async () => {
  editorOpen.value = false
  await refresh()
})

watch(
  [distributeOverwrite, distributeTarget, distributeLabels, distributeLabelsMode, distributeNodeIdsRaw],
  () => {
    distributePreview.value = null
  },
  { deep: true },
)

watch(distributeOpen, (open) => {
  if (open) return
  distributeError.value = null
  distributePreview.value = null
})
</script>

<template>
  <div class="space-y-6">
    <n-card class="app-card" :bordered="false" :title="t('settings.webdav.title')">
      <template #header-extra>
        <n-button type="primary" size="small" @click="openCreate">{{ t('settings.webdav.new') }}</n-button>
        <n-button size="small" @click="refresh">{{ t('common.refresh') }}</n-button>
      </template>

      <div v-if="!isDesktop" class="space-y-2">
        <div
          v-if="!secrets.loadingWebdav && secrets.webdav.length === 0"
          class="app-help-text px-1 py-2"
        >
          {{ t('common.noData') }}
        </div>
        <div
          v-for="row in secrets.webdav"
          :key="row.name"
          class="p-3 rounded-lg app-border-subtle app-glass-soft"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="font-medium">{{ row.name }}</div>
              <div class="text-xs app-text-muted mt-1">{{ formatUnixSeconds(row.updated_at) }}</div>
            </div>
            <n-space size="small">
              <n-button size="small" @click="copyToClipboard(row.name)">{{ t('common.copy') }}</n-button>
              <n-button size="small" @click="openEdit(row.name)">{{ t('common.edit') }}</n-button>
              <n-button v-if="nodeId === 'hub'" size="small" @click="openDistribute(row.name)">{{ t('settings.webdav.distribute.action') }}</n-button>
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

    <n-modal
      v-model:show="distributeOpen"
      preset="card"
      :style="{ width: MODAL_WIDTH.lg }"
      :title="t('settings.webdav.distribute.title', { name: distributeSecretName })"
    >
      <div class="space-y-4">
        <n-alert v-if="distributeError" type="error" :bordered="false">
          {{ distributeError }}
        </n-alert>

        <div class="text-sm app-text-muted">{{ t('settings.webdav.distribute.help') }}</div>

        <n-form label-placement="top">
          <n-form-item :label="t('settings.webdav.distribute.target')">
            <n-radio-group v-model:value="distributeTarget" size="small">
              <n-radio-button value="labels">{{ t('settings.webdav.distribute.targetLabels') }}</n-radio-button>
              <n-radio-button value="node_ids">{{ t('settings.webdav.distribute.targetNodeIds') }}</n-radio-button>
            </n-radio-group>
          </n-form-item>

          <n-form-item v-if="distributeTarget === 'labels'" :label="t('settings.webdav.distribute.labels')">
            <n-select
              v-model:value="distributeLabels"
              multiple
              filterable
              clearable
              :loading="distributeLabelsLoading"
              :options="distributeLabelOptions"
              :placeholder="t('settings.webdav.distribute.labelsPlaceholder')"
            />
          </n-form-item>

          <n-form-item v-if="distributeTarget === 'labels'" :label="t('settings.webdav.distribute.labelsMode')">
            <n-radio-group v-model:value="distributeLabelsMode" size="small">
              <n-radio-button value="and">{{ t('common.and') }}</n-radio-button>
              <n-radio-button value="or">{{ t('common.or') }}</n-radio-button>
            </n-radio-group>
          </n-form-item>

          <n-form-item v-if="distributeTarget === 'node_ids'" :label="t('settings.webdav.distribute.nodeIds')">
            <n-input
              v-model:value="distributeNodeIdsRaw"
              type="textarea"
              :autosize="{ minRows: 3, maxRows: 8 }"
              :placeholder="t('settings.webdav.distribute.nodeIdsPlaceholder')"
            />
          </n-form-item>

          <n-form-item :label="t('settings.webdav.distribute.overwrite')">
            <n-checkbox v-model:checked="distributeOverwrite">
              {{ t('settings.webdav.distribute.overwriteHelp') }}
            </n-checkbox>
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button :disabled="distributeLoading || distributeSaving" @click="previewDistribute">
            {{ t('settings.webdav.distribute.preview') }}
          </n-button>
          <n-button @click="distributeOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button
            type="primary"
            :loading="distributeSaving"
            :disabled="distributeLoading"
            @click="createDistributeOperation"
          >
            {{ t('settings.webdav.distribute.execute') }}
          </n-button>
        </n-space>

        <n-card v-if="distributePreview" size="small" class="app-card" :bordered="false" :title="t('settings.webdav.distribute.previewTitle')">
          <div class="overflow-x-auto">
            <n-data-table :columns="previewColumns" :data="distributePreview.items" size="small" />
          </div>
        </n-card>
      </div>
    </n-modal>
  </div>
</template>
