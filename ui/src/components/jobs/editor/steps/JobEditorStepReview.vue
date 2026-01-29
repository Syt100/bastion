<script setup lang="ts">
import { NAlert, NButton, NCode, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { copyText } from '@/lib/clipboard'

import { useJobEditorContext } from '../context'

defineProps<{
  nodeLabel: string
  overlapLabel: string
  jobTypeLabel: string
  targetTypeLabel: string
  notifyModeLabel: string
  fsSymlinkPolicyLabel: string
  fsHardlinkPolicyLabel: string
  fsErrorPolicyLabel: string
  disabledWecomSelected: string[]
  disabledEmailSelected: string[]
}>()

const { t } = useI18n()
const message = useMessage()

const { form, showJsonPreview, previewPayload } = useJobEditorContext()

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

async function copyPreviewJson(): Promise<void> {
  const ok = await copyText(formatJson(previewPayload.value))
  if (ok) {
    message.success(t('messages.copied'))
  } else {
    message.error(t('errors.copyFailed'))
  }
}
</script>

<template>
  <n-alert type="info" :bordered="false">
    {{ t('jobs.steps.reviewHelp') }}
  </n-alert>

  <div class="mt-3 space-y-3">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <div class="app-border-subtle rounded-lg p-3 app-glass-soft">
        <div class="text-sm font-medium">{{ t('jobs.steps.basics') }}</div>
        <div class="mt-2 space-y-2 text-sm">
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.name') }}</div>
            <div class="font-medium text-right break-all">{{ form.name.trim() }}</div>
          </div>
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.node') }}</div>
            <div class="font-medium text-right break-all">{{ nodeLabel }}</div>
          </div>
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.type') }}</div>
            <div class="font-medium text-right break-all">{{ jobTypeLabel }}</div>
          </div>
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.overlap') }}</div>
            <div class="font-medium text-right break-all">{{ overlapLabel }}</div>
          </div>
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.scheduleTimezone') }}</div>
            <div class="font-medium text-right break-all">{{ form.scheduleTimezone.trim() || '-' }}</div>
          </div>
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.schedule') }}</div>
            <div class="font-medium text-right break-all">{{ form.schedule.trim() ? form.schedule.trim() : '-' }}</div>
          </div>
        </div>
      </div>

      <div class="app-border-subtle rounded-lg p-3 app-glass-soft">
        <div class="text-sm font-medium">{{ t('jobs.steps.target') }}</div>
        <div class="mt-2 space-y-2 text-sm">
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.targetType') }}</div>
            <div class="font-medium text-right break-all">{{ targetTypeLabel }}</div>
          </div>
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.partSizeMiB') }}</div>
            <div class="font-medium text-right break-all">{{ Math.max(1, Math.floor(form.partSizeMiB || 1)) }}</div>
          </div>
          <div v-if="form.targetType === 'webdav'" class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.webdavBaseUrl') }}</div>
            <div class="font-medium text-right break-all">{{ form.webdavBaseUrl.trim() }}</div>
          </div>
          <div v-if="form.targetType === 'webdav'" class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.webdavSecret') }}</div>
            <div class="font-medium text-right break-all">{{ form.webdavSecretName.trim() }}</div>
          </div>
          <div v-if="form.targetType === 'local_dir'" class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.localBaseDir') }}</div>
            <div class="font-medium text-right break-all">{{ form.localBaseDir.trim() }}</div>
          </div>
        </div>

        <div class="mt-4 text-sm font-medium">{{ t('jobs.steps.source') }}</div>
        <div class="mt-2 space-y-2 text-sm">
          <template v-if="form.jobType === 'filesystem'">
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.sourcePaths') }}</div>
              <div class="font-medium text-right">{{ form.fsPaths.length }}</div>
            </div>
            <div v-if="form.fsPaths.length > 0" class="flex flex-wrap gap-2">
              <n-tag v-for="p in form.fsPaths.slice(0, 6)" :key="p">{{ p }}</n-tag>
              <n-tag v-if="form.fsPaths.length > 6" type="info">+{{ form.fsPaths.length - 6 }}</n-tag>
            </div>
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.fsSymlinkPolicy') }}</div>
              <div class="font-medium text-right">{{ fsSymlinkPolicyLabel }}</div>
            </div>
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.fsHardlinkPolicy') }}</div>
              <div class="font-medium text-right">{{ fsHardlinkPolicyLabel }}</div>
            </div>
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.fsErrorPolicy') }}</div>
              <div class="font-medium text-right">{{ fsErrorPolicyLabel }}</div>
            </div>
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.fsInclude') }}</div>
              <div class="font-medium text-right">
                {{ form.fsInclude.trim() ? form.fsInclude.split(/\r?\n/g).filter((l) => l.trim()).length : 0 }}
              </div>
            </div>
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.fsExclude') }}</div>
              <div class="font-medium text-right">
                {{ form.fsExclude.trim() ? form.fsExclude.split(/\r?\n/g).filter((l) => l.trim()).length : 0 }}
              </div>
            </div>
          </template>

          <template v-else-if="form.jobType === 'sqlite'">
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.sqlitePath') }}</div>
              <div class="font-medium text-right break-all">{{ form.sqlitePath.trim() }}</div>
            </div>
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.sqliteIntegrityCheck') }}</div>
              <div class="font-medium text-right break-all">{{ form.sqliteIntegrityCheck ? t('common.yes') : t('common.no') }}</div>
            </div>
          </template>

          <template v-else>
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.vaultwardenDataDir') }}</div>
              <div class="font-medium text-right break-all">{{ form.vaultwardenDataDir.trim() }}</div>
            </div>
          </template>
        </div>
      </div>

      <div class="app-border-subtle rounded-lg p-3 app-glass-soft">
        <div class="text-sm font-medium">{{ t('jobs.steps.security') }}</div>
        <div class="mt-2 space-y-2 text-sm">
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.artifactFormat') }}</div>
            <div class="font-medium text-right break-all">
              {{
                form.artifactFormat === 'raw_tree_v1'
                  ? t('jobs.fields.artifactFormatRawTreeV1')
                  : t('jobs.fields.artifactFormatArchiveV1')
              }}
            </div>
          </div>
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.encryptionEnabled') }}</div>
            <div class="font-medium text-right break-all">
              {{ form.encryptionEnabled ? t('common.yes') : t('common.no') }}
            </div>
          </div>
          <div v-if="form.encryptionEnabled" class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.encryptionKeyName') }}</div>
            <div class="font-medium text-right break-all">{{ form.encryptionKeyName.trim() }}</div>
          </div>
        </div>

        <div class="mt-4 text-sm font-medium">{{ t('jobs.steps.notifications') }}</div>
        <div class="mt-2 space-y-2 text-sm">
          <div class="flex items-start justify-between gap-3">
            <div class="opacity-70">{{ t('jobs.fields.notificationsMode') }}</div>
            <div class="font-medium text-right break-all">{{ notifyModeLabel }}</div>
          </div>

          <template v-if="form.notifyMode === 'custom'">
            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.notifyWecomBots') }}</div>
              <div class="font-medium text-right">{{ form.notifyWecomBots.length }}</div>
            </div>
            <div v-if="form.notifyWecomBots.length > 0" class="flex flex-wrap gap-2">
              <n-tag v-for="name in form.notifyWecomBots.slice(0, 6)" :key="name">{{ name }}</n-tag>
              <n-tag v-if="form.notifyWecomBots.length > 6" type="info">+{{ form.notifyWecomBots.length - 6 }}</n-tag>
            </div>

            <div class="flex items-start justify-between gap-3">
              <div class="opacity-70">{{ t('jobs.fields.notifyEmails') }}</div>
              <div class="font-medium text-right">{{ form.notifyEmails.length }}</div>
            </div>
            <div v-if="form.notifyEmails.length > 0" class="flex flex-wrap gap-2">
              <n-tag v-for="name in form.notifyEmails.slice(0, 6)" :key="name">{{ name }}</n-tag>
              <n-tag v-if="form.notifyEmails.length > 6" type="info">+{{ form.notifyEmails.length - 6 }}</n-tag>
            </div>

            <n-alert
              v-if="disabledWecomSelected.length > 0 || disabledEmailSelected.length > 0"
              class="mt-2"
              type="warning"
              :bordered="false"
            >
              <div v-if="disabledWecomSelected.length > 0">
                {{ t('jobs.fields.notifyDisabledSelected', { names: disabledWecomSelected.join(', ') }) }}
              </div>
              <div v-if="disabledEmailSelected.length > 0">
                {{ t('jobs.fields.notifyDisabledSelected', { names: disabledEmailSelected.join(', ') }) }}
              </div>
            </n-alert>
          </template>
        </div>
      </div>
    </div>

    <div class="app-border-subtle rounded-lg p-3 app-glass-soft">
      <div class="flex items-center justify-between gap-3">
        <div class="text-sm font-medium">{{ t('common.json') }}</div>
        <div class="flex items-center gap-2">
          <n-button size="small" secondary @click="showJsonPreview = !showJsonPreview">
            {{ showJsonPreview ? t('jobs.actions.hideJson') : t('jobs.actions.showJson') }}
          </n-button>
          <n-button v-if="showJsonPreview" size="small" secondary @click="copyPreviewJson">
            {{ t('jobs.actions.copyJson') }}
          </n-button>
        </div>
      </div>
      <n-code v-if="showJsonPreview" class="mt-2" :code="formatJson(previewPayload)" language="json" />
    </div>
  </div>
</template>
