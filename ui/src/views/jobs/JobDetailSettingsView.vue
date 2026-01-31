<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NCheckbox, NModal, NSpace, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import JobEditorModal, { type JobEditorModalExpose } from '@/components/jobs/JobEditorModal.vue'
import JobDeployModal, { type JobDeployModalExpose } from '@/components/jobs/JobDeployModal.vue'
import { useJobsStore, type JobDetail } from '@/stores/jobs'
import { formatToastError } from '@/lib/errors'
import { MODAL_WIDTH } from '@/lib/modal'
import { useJobDetailContext } from '@/lib/jobDetailContext'

const { t } = useI18n()
const message = useMessage()
const router = useRouter()

const ctx = useJobDetailContext()
const jobs = useJobsStore()

const job = computed<JobDetail | null>(() => ctx.job.value)

const editorModal = ref<JobEditorModalExpose | null>(null)
const deployModal = ref<JobDeployModalExpose | null>(null)

async function openEdit(): Promise<void> {
  if (!job.value) return
  await editorModal.value?.openEdit(job.value.id, { nodeId: ctx.nodeId.value })
}

async function openDeploy(): Promise<void> {
  if (!job.value) return
  await deployModal.value?.open(job.value.id)
}

const deleteOpen = ref<boolean>(false)
const deleteBusy = ref<'archive' | 'delete' | null>(null)
const archiveCascadeSnapshots = ref<boolean>(false)

function openDelete(): void {
  archiveCascadeSnapshots.value = false
  deleteOpen.value = true
}

async function archiveJob(): Promise<boolean> {
  if (!job.value) return false
  try {
    await jobs.archiveJob(job.value.id, { cascadeSnapshots: archiveCascadeSnapshots.value })
    message.success(t('messages.jobArchived'))
    await ctx.refresh()
    return true
  } catch (error) {
    message.error(formatToastError(t('errors.archiveJobFailed'), error, t))
    return false
  }
}

async function unarchiveJob(): Promise<void> {
  if (!job.value) return
  try {
    await jobs.unarchiveJob(job.value.id)
    message.success(t('messages.jobUnarchived'))
    await ctx.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.unarchiveJobFailed'), error, t))
  }
}

async function deleteJobPermanently(): Promise<boolean> {
  if (!job.value) return false
  try {
    await jobs.deleteJob(job.value.id)
    message.success(t('messages.jobDeleted'))
    void router.push(`/n/${encodeURIComponent(ctx.nodeId.value)}/jobs`)
    return true
  } catch (error) {
    message.error(formatToastError(t('errors.deleteJobFailed'), error, t))
    return false
  }
}

async function confirmArchive(): Promise<void> {
  deleteBusy.value = 'archive'
  try {
    const ok = await archiveJob()
    if (ok) deleteOpen.value = false
  } finally {
    deleteBusy.value = null
  }
}

async function confirmDeletePermanently(): Promise<void> {
  deleteBusy.value = 'delete'
  try {
    const ok = await deleteJobPermanently()
    if (ok) deleteOpen.value = false
  } finally {
    deleteBusy.value = null
  }
}
</script>

<template>
  <div class="space-y-3">
    <AppEmptyState v-if="!job" :title="t('common.noData')" />

    <template v-else>
      <n-card class="app-card" :title="t('jobs.detail.sections.actions')">
        <div class="flex flex-wrap gap-2 justify-end">
          <n-button :disabled="!!job.archived_at" @click="openEdit">{{ t('common.edit') }}</n-button>
          <n-button :disabled="!!job.archived_at" @click="openDeploy">{{ t('jobs.actions.deploy') }}</n-button>
          <n-button v-if="job.archived_at" @click="unarchiveJob">{{ t('jobs.actions.unarchive') }}</n-button>
          <n-button v-else type="warning" @click="openDelete">{{ t('jobs.actions.archive') }}</n-button>
          <n-button type="error" tertiary @click="openDelete">{{ t('common.delete') }}</n-button>
        </div>
      </n-card>

      <n-modal v-model:show="deleteOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('jobs.deleteTitle')">
        <div class="space-y-3">
          <div class="text-sm opacity-80">
            {{
              job.archived_at
                ? t('jobs.deletePermanentlyHelp')
                : t('jobs.deleteHelp')
            }}
          </div>

          <div v-if="!job.archived_at" class="rounded border border-slate-200/60 dark:border-slate-700/60 p-3 space-y-1">
            <n-checkbox :checked="archiveCascadeSnapshots" @update:checked="(v) => (archiveCascadeSnapshots = v)">
              {{ t('jobs.archiveCascadeLabel') }}
            </n-checkbox>
            <div class="text-xs opacity-70">{{ t('jobs.archiveCascadeHelp') }}</div>
          </div>

          <n-space justify="end">
            <n-button :disabled="deleteBusy !== null" @click="deleteOpen = false">{{ t('common.cancel') }}</n-button>
            <n-button
              v-if="!job.archived_at"
              type="warning"
              :loading="deleteBusy === 'archive'"
              @click="confirmArchive"
            >
              {{ t('jobs.actions.archive') }}
            </n-button>
            <n-button type="error" :loading="deleteBusy === 'delete'" @click="confirmDeletePermanently">
              {{ t('jobs.actions.deletePermanently') }}
            </n-button>
          </n-space>
        </div>
      </n-modal>

      <JobEditorModal ref="editorModal" @saved="ctx.refresh" />
      <JobDeployModal ref="deployModal" />
    </template>
  </div>
</template>

