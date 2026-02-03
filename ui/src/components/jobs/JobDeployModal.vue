<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import {
  NButton,
  NCard,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NModal,
  NRadioButton,
  NRadioGroup,
  NSelect,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { useAgentsStore, type AgentsLabelsMode } from '@/stores/agents'
import { useBulkOperationsStore, type BulkSelectorRequest, type JobDeployPreviewItem, type JobDeployPreviewResponse } from '@/stores/bulkOperations'
import { useJobsStore } from '@/stores/jobs'
import { MODAL_WIDTH } from '@/lib/modal'
import { formatToastError } from '@/lib/errors'

export type JobDeployModalExpose = {
  open: (sourceJobId: string) => Promise<void>
}

const { t } = useI18n()
const message = useMessage()
const router = useRouter()

const agents = useAgentsStore()
const jobs = useJobsStore()
const bulkOps = useBulkOperationsStore()

const show = ref<boolean>(false)
const sourceJobId = ref<string | null>(null)
const sourceJobName = ref<string>('')

const targetMode = ref<'labels' | 'nodes'>('labels')
const labelIndexLoading = ref<boolean>(false)
const labelIndex = ref<{ label: string; count: number }[]>([])
const selectedLabels = ref<string[]>([])
const labelsMode = ref<AgentsLabelsMode>('and')
const selectedNodeIds = ref<string[]>([])

const nameTemplate = ref<string>('{name} ({node})')

const previewLoading = ref<boolean>(false)
const preview = ref<JobDeployPreviewResponse | null>(null)

const saving = ref<boolean>(false)
const error = ref<string | null>(null)

const labelOptions = computed(() =>
  labelIndex.value.map((it) => ({ label: `${it.label} (${it.count})`, value: it.label })),
)

const nodeOptions = computed(() =>
  agents.items
    .filter((a) => !a.revoked)
    .map((a) => ({
      label: a.name ? `${a.name} (${a.id})` : a.id,
      value: a.id,
    })),
)

const previewCounts = computed(() => {
  const items = preview.value?.items ?? []
  const valid = items.filter((it) => it.valid).length
  const invalid = items.length - valid
  return { total: items.length, valid, invalid }
})

function resetState(): void {
  sourceJobId.value = null
  sourceJobName.value = ''
  targetMode.value = 'labels'
  selectedLabels.value = []
  labelsMode.value = 'and'
  selectedNodeIds.value = []
  nameTemplate.value = '{name} ({node})'
  preview.value = null
  error.value = null
}

async function refreshLabelIndex(): Promise<void> {
  labelIndexLoading.value = true
  try {
    labelIndex.value = await agents.listLabelIndex()
  } catch (e) {
    message.error(formatToastError(t('errors.fetchAgentLabelsFailed'), e, t))
    labelIndex.value = []
  } finally {
    labelIndexLoading.value = false
  }
}

function buildSelector(): { ok: true; selector: BulkSelectorRequest } | { ok: false } {
  if (targetMode.value === 'nodes') {
    const ids = Array.from(new Set(selectedNodeIds.value.map((v) => v.trim()).filter((v) => v.length > 0)))
    if (ids.length === 0) return { ok: false }
    return { ok: true, selector: { node_ids: ids } }
  }

  if (selectedLabels.value.length === 0) return { ok: false }
  return { ok: true, selector: { labels: selectedLabels.value, labels_mode: labelsMode.value } }
}

async function previewDeploy(): Promise<void> {
  const id = sourceJobId.value
  if (!id) return

  const sel = buildSelector()
  if (!sel.ok) {
    error.value = t('errors.formInvalid')
    return
  }

  const template = nameTemplate.value.trim() || '{name} ({node})'

  previewLoading.value = true
  error.value = null
  try {
    preview.value = await bulkOps.previewJobDeploy({
      selector: sel.selector,
      payload: { source_job_id: id, name_template: template },
    })
  } catch (e) {
    error.value = formatToastError(t('errors.previewBulkOperationFailed'), e, t)
    preview.value = null
  } finally {
    previewLoading.value = false
  }
}

async function deploy(): Promise<void> {
  const id = sourceJobId.value
  if (!id) return

  const sel = buildSelector()
  if (!sel.ok) {
    error.value = t('errors.formInvalid')
    return
  }
  if (!preview.value) {
    error.value = t('jobs.deploy.previewRequired')
    return
  }

  const template = nameTemplate.value.trim() || '{name} ({node})'

  saving.value = true
  error.value = null
  try {
    const opId = await bulkOps.create({
      kind: 'job_deploy',
      selector: sel.selector,
      payload: { source_job_id: id, name_template: template },
    })

    message.success(t('messages.bulkOperationCreated'))
    show.value = false
    preview.value = null
    await router.push({ path: '/settings/bulk-operations', query: { open: opId } })
  } catch (e) {
    error.value = formatToastError(t('errors.createBulkOperationFailed'), e, t)
  } finally {
    saving.value = false
  }
}

const previewColumns = computed<DataTableColumns<JobDeployPreviewItem>>(() => [
  {
    title: t('jobs.deploy.previewColumns.node'),
    key: 'agent',
    render: (row) => row.agent_name ?? row.agent_id,
  },
  {
    title: t('jobs.deploy.previewColumns.plannedName'),
    key: 'planned_name',
    render: (row) => row.planned_name,
  },
  {
    title: t('jobs.deploy.previewColumns.status'),
    key: 'valid',
    render: (row) =>
      h(
        NTag,
        { size: 'small', type: row.valid ? 'success' : 'error' },
        { default: () => (row.valid ? t('common.yes') : t('common.no')) },
      ),
  },
  {
    title: t('jobs.deploy.previewColumns.error'),
    key: 'error',
    render: (row) => row.error ?? '-',
  },
])

watch(
  [targetMode, selectedLabels, labelsMode, selectedNodeIds, nameTemplate],
  () => {
    preview.value = null
    error.value = null
  },
  { deep: true },
)

async function open(nextSourceJobId: string): Promise<void> {
  resetState()
  show.value = true
  sourceJobId.value = nextSourceJobId
  sourceJobName.value = jobs.items.find((j) => j.id === nextSourceJobId)?.name ?? nextSourceJobId

  try {
    if (agents.items.length === 0) {
      await agents.refresh()
    }
  } catch (e) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), e, t))
  }

  await refreshLabelIndex()
}

defineExpose<JobDeployModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('jobs.deploy.title')">
    <div class="space-y-4">
      <div class="text-sm app-text-muted">{{ sourceJobName }}</div>

      <n-card size="small" class="app-card" :bordered="false">
        <n-form label-placement="top">
          <n-form-item :label="t('jobs.deploy.target')" :show-feedback="false">
            <n-radio-group v-model:value="targetMode" size="small">
              <n-radio-button value="labels">{{ t('jobs.deploy.targetLabels') }}</n-radio-button>
              <n-radio-button value="nodes">{{ t('jobs.deploy.targetNodes') }}</n-radio-button>
            </n-radio-group>
          </n-form-item>

          <template v-if="targetMode === 'labels'">
            <n-form-item :label="t('jobs.deploy.labels')" :show-feedback="false">
              <n-select
                v-model:value="selectedLabels"
                multiple
                filterable
                clearable
                :loading="labelIndexLoading"
                :options="labelOptions"
                :placeholder="t('jobs.deploy.labelsPlaceholder')"
              />
            </n-form-item>

            <n-form-item :label="t('jobs.deploy.labelsMode')" :show-feedback="false">
              <n-radio-group v-model:value="labelsMode" size="small">
                <n-radio-button value="and">{{ t('common.and') }}</n-radio-button>
                <n-radio-button value="or">{{ t('common.or') }}</n-radio-button>
              </n-radio-group>
            </n-form-item>
          </template>

          <template v-else>
            <n-form-item :label="t('jobs.deploy.nodes')" :show-feedback="false">
              <n-select
                v-model:value="selectedNodeIds"
                multiple
                filterable
                clearable
                :options="nodeOptions"
                :placeholder="t('jobs.deploy.nodesPlaceholder')"
              />
            </n-form-item>
          </template>

          <n-form-item :label="t('jobs.deploy.nameTemplate')" :show-feedback="false">
            <n-input v-model:value="nameTemplate" :placeholder="t('jobs.deploy.nameTemplatePlaceholder')" />
            <div class="text-xs app-text-muted mt-1">{{ t('jobs.deploy.nameTemplateHelp') }}</div>
          </n-form-item>
        </n-form>
      </n-card>

      <n-card v-if="preview" size="small" class="app-card" :bordered="false">
        <div class="flex flex-wrap items-center justify-between gap-2 mb-2">
          <div class="text-sm app-text-muted">
            {{ t('jobs.deploy.previewSummary', previewCounts) }}
          </div>
        </div>
        <div class="overflow-x-auto">
          <n-data-table :columns="previewColumns" :data="preview.items" :bordered="false" />
        </div>
      </n-card>

      <div v-if="error" class="text-sm text-[var(--app-danger)]">
        {{ error }}
      </div>

      <n-space justify="end">
        <n-button :loading="previewLoading" @click="previewDeploy">{{ t('jobs.deploy.preview') }}</n-button>
        <n-button type="primary" :loading="saving" @click="deploy">{{ t('jobs.deploy.deploy') }}</n-button>
        <n-button :disabled="previewLoading || saving" @click="show = false">{{ t('common.cancel') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>
