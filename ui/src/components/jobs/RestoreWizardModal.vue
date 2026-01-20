<script setup lang="ts">
import { computed, ref } from 'vue'
import { NButton, NForm, NFormItem, NInput, NModal, NSelect, NSpace, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useAgentsStore } from '@/stores/agents'
import { useSecretsStore } from '@/stores/secrets'
import { useOperationsStore, type ConflictPolicy, type RestoreDestination } from '@/stores/operations'
import { MODAL_WIDTH } from '@/lib/modal'
import { formatToastError } from '@/lib/errors'
import RunEntriesPickerModal, {
  type RunEntriesPickerModalExpose,
  type RunEntriesSelection,
} from '@/components/jobs/RunEntriesPickerModal.vue'
import FsPathPickerModal, { type FsPathPickerModalExpose } from '@/components/fs/FsPathPickerModal.vue'

export type RestoreWizardModalExpose = {
  open: (runId: string, opts?: { defaultNodeId?: string | null }) => void
}

const emit = defineEmits<{
  (e: 'started', opId: string): void
}>()

const { t } = useI18n()
const message = useMessage()

const agents = useAgentsStore()
const secrets = useSecretsStore()
const operations = useOperationsStore()

const show = ref<boolean>(false)
const starting = ref<boolean>(false)
const runId = ref<string | null>(null)
const destinationType = ref<'local_fs' | 'webdav'>('local_fs')
const localFsNodeId = ref<'hub' | string>('hub')
const localFsDirectory = ref<string>('')
const webdavNodeId = ref<'hub' | string>('hub')
const webdavBaseUrl = ref<string>('')
const webdavSecretName = ref<string>('')
const webdavPrefix = ref<string>('')
const conflictPolicy = ref<ConflictPolicy>('overwrite')
const selection = ref<RunEntriesSelection | null>(null)
const entriesPicker = ref<RunEntriesPickerModalExpose | null>(null)
const fsPicker = ref<FsPathPickerModalExpose | null>(null)

const conflictOptions = computed(() => [
  { label: t('restore.conflict.overwrite'), value: 'overwrite' },
  { label: t('restore.conflict.skip'), value: 'skip' },
  { label: t('restore.conflict.fail'), value: 'fail' },
])

const destinationTypeOptions = computed(() => [
  { label: t('restore.destinations.localFs'), value: 'local_fs' },
  { label: t('restore.destinations.webdav'), value: 'webdav' },
])

const nodeOptions = computed(() => [
  { label: t('jobs.nodes.hub'), value: 'hub' },
  ...agents.items
    .filter((a) => !a.revoked)
    .map((a) => ({
      label: a.name || a.id,
      value: a.id,
      disabled: !a.online,
    })),
])

const webdavSecretOptions = computed(() =>
  secrets.webdav.map((s) => ({
    label: s.name,
    value: s.name,
  })),
)

const selectionSummary = computed(() => {
  const v = selection.value
  if (!v) return null
  return t('restore.selectionSummary', { files: v.files.length, dirs: v.dirs.length })
})

function open(nextRunId: string, opts?: { defaultNodeId?: string | null }): void {
  runId.value = nextRunId
  destinationType.value = 'local_fs'
  const defaultNodeIdOrHub = opts?.defaultNodeId ? opts.defaultNodeId : 'hub'
  localFsNodeId.value = defaultNodeIdOrHub
  webdavNodeId.value = defaultNodeIdOrHub
  localFsDirectory.value = ''
  webdavBaseUrl.value = ''
  webdavSecretName.value = ''
  webdavPrefix.value = ''
  conflictPolicy.value = 'overwrite'
  selection.value = null
  show.value = true

  // Best-effort: refresh WebDAV secrets for the node that will execute this restore by default.
  void secrets.refreshWebdav(webdavNodeId.value).catch((error) => {
    message.error(formatToastError(t('errors.fetchWebdavSecretsFailed'), error, t))
  })
}

function openPicker(): void {
  const id = runId.value
  if (!id) return
  entriesPicker.value?.open(id)
}

function openLocalFsDirectoryPicker(): void {
  fsPicker.value?.open(localFsNodeId.value, {
    mode: 'single_dir',
    path: localFsDirectory.value.trim() || undefined,
  })
}

function onFsPicked(paths: string[]): void {
  localFsDirectory.value = paths[0] || ''
}

function clearSelection(): void {
  selection.value = null
}

function onPicked(next: RunEntriesSelection): void {
  selection.value = next
}

async function start(): Promise<void> {
  const id = runId.value
  if (!id) return

  let destination: RestoreDestination
  if (destinationType.value === 'local_fs') {
    const nodeId = localFsNodeId.value.trim()
    const directory = localFsDirectory.value.trim()
    if (!directory) {
      message.error(t('errors.restoreDestinationRequired'))
      return
    }
    destination = { type: 'local_fs', node_id: nodeId, directory }
  } else {
    const baseUrl = webdavBaseUrl.value.trim()
    const secretName = webdavSecretName.value.trim()
    const prefix = webdavPrefix.value.trim()
    if (!baseUrl) {
      message.error(t('errors.webdavBaseUrlRequired'))
      return
    }
    if (!secretName) {
      message.error(t('errors.webdavSecretRequired'))
      return
    }
    if (!prefix) {
      message.error(t('errors.webdavPrefixRequired'))
      return
    }
    destination = { type: 'webdav', base_url: baseUrl, secret_name: secretName, prefix }
  }

  starting.value = true
  try {
    const opId = await operations.startRestore(id, destination, conflictPolicy.value, selection.value)
    show.value = false
    emit('started', opId)
  } catch (error) {
    message.error(formatToastError(t('errors.restoreStartFailed'), error, t))
  } finally {
    starting.value = false
  }
}

defineExpose<RestoreWizardModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('restore.title')">
    <div class="space-y-4">
      <div class="text-sm opacity-70">{{ runId }}</div>
      <n-form label-placement="top">
        <n-form-item :label="t('restore.fields.destinationType')">
          <n-select v-model:value="destinationType" :options="destinationTypeOptions" />
        </n-form-item>

        <template v-if="destinationType === 'local_fs'">
          <n-form-item :label="t('restore.fields.node')">
            <n-select v-model:value="localFsNodeId" :options="nodeOptions" />
          </n-form-item>
          <n-form-item :label="t('restore.fields.destinationDir')">
            <div class="space-y-1 w-full">
              <div class="flex gap-2">
                <n-input
                  v-model:value="localFsDirectory"
                  class="flex-1"
                  :placeholder="t('restore.fields.destinationDirPlaceholder')"
                />
                <n-button @click="openLocalFsDirectoryPicker">{{ t('restore.actions.browse') }}</n-button>
              </div>
              <div class="text-xs opacity-70">{{ t('restore.fields.destinationDirHelp') }}</div>
            </div>
          </n-form-item>
        </template>

        <template v-else>
          <n-form-item :label="t('restore.fields.webdavBaseUrl')">
            <n-input v-model:value="webdavBaseUrl" :placeholder="t('restore.fields.webdavBaseUrlPlaceholder')" />
          </n-form-item>
          <n-form-item :label="t('restore.fields.webdavSecret')">
            <n-select v-model:value="webdavSecretName" :options="webdavSecretOptions" filterable />
          </n-form-item>
          <n-form-item :label="t('restore.fields.webdavPrefix')">
            <div class="space-y-1 w-full">
              <n-input v-model:value="webdavPrefix" :placeholder="t('restore.fields.webdavPrefixPlaceholder')" />
              <div class="text-xs opacity-70">{{ t('restore.fields.webdavPrefixHelp') }}</div>
            </div>
          </n-form-item>
          <div class="text-xs opacity-70">
            {{ t('restore.fields.webdavMetaNote') }}
          </div>
        </template>

        <n-form-item :label="t('restore.fields.conflictPolicy')">
          <n-select v-model:value="conflictPolicy" :options="conflictOptions" />
        </n-form-item>
        <n-form-item :label="t('restore.fields.selection')">
          <div class="space-y-2 w-full">
            <div class="flex flex-wrap items-center justify-between gap-2">
              <div class="text-xs opacity-70">{{ t('restore.fields.selectionHelp') }}</div>
              <div class="flex items-center gap-2">
                <n-button size="small" @click="openPicker">{{ t('restore.actions.pick') }}</n-button>
                <n-button size="small" :disabled="selection == null" @click="clearSelection">
                  {{ t('restore.actions.clearSelection') }}
                </n-button>
              </div>
            </div>
            <div v-if="selectionSummary" class="text-sm">
              {{ selectionSummary }}
            </div>
          </div>
        </n-form-item>
      </n-form>
      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
        <n-button type="primary" :loading="starting" @click="start">{{ t('restore.actions.start') }}</n-button>
      </n-space>
    </div>
  </n-modal>

  <RunEntriesPickerModal ref="entriesPicker" @picked="onPicked" />
  <FsPathPickerModal ref="fsPicker" @picked="onFsPicked" />
</template>
