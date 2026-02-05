<script setup lang="ts">
import { computed } from 'vue'
import { NAlert, NButton, NFormItem, NInput, NInputNumber, NSelect, NSwitch, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobEditorContext } from '../context'

type Option = { label: string; value: string }

defineProps<{
  fsSymlinkPolicyOptions: Array<Option>
  fsHardlinkPolicyOptions: Array<Option>
  fsErrorPolicyOptions: Array<Option>
}>()

const { t } = useI18n()

const {
  form,
  fieldErrors,
  fsPathDraft,
  clearFieldError,
  openFsPicker,
  addFsPathsFromDraft,
  removeFsPath,
  clearFsPaths,
} = useJobEditorContext()

const consistencyPolicyOptions = computed(() => [
  { label: t('jobs.consistency.policy.warn'), value: 'warn' },
  { label: t('jobs.consistency.policy.fail'), value: 'fail' },
  { label: t('jobs.consistency.policy.ignore'), value: 'ignore' },
])

const snapshotModeOptions = computed(() => [
  { label: t('jobs.snapshot.mode.off'), value: 'off' },
  { label: t('jobs.snapshot.mode.auto'), value: 'auto' },
  { label: t('jobs.snapshot.mode.required'), value: 'required' },
])

const snapshotProviderOptions = computed(() => [
  { label: t('jobs.snapshot.provider.default'), value: '' },
  { label: t('jobs.snapshot.provider.btrfs'), value: 'btrfs' },
])
</script>

<template>
  <n-alert type="info" :bordered="false">
    {{ t('jobs.steps.sourceHelp') }}
  </n-alert>

  <template v-if="form.jobType === 'filesystem'">
    <div data-field="fsPaths">
      <n-form-item
        :label="t('jobs.fields.sourcePaths')"
        required
        :validation-status="fieldErrors.fsPaths ? 'error' : undefined"
        :feedback="fieldErrors.fsPaths || undefined"
      >
        <div class="space-y-3 w-full app-border-subtle rounded-lg p-3 app-glass-soft">
          <div class="flex flex-wrap items-center gap-2 justify-between">
            <div v-if="!fieldErrors.fsPaths" class="text-xs app-text-muted">{{ t('jobs.fields.sourcePathsHelp') }}</div>
            <div class="flex items-center gap-2">
              <n-button size="small" type="primary" @click="openFsPicker">
                {{ t('jobs.actions.browseFs') }}
              </n-button>
              <n-button size="small" :disabled="form.fsPaths.length === 0" @click="clearFsPaths">
                {{ t('common.clear') }}
              </n-button>
            </div>
          </div>

          <div v-if="form.fsPaths.length === 0" class="text-sm app-text-muted">
            {{ t('jobs.fields.sourcePathsEmpty') }}
          </div>
          <div v-else class="flex flex-wrap gap-2">
            <n-tag v-for="p in form.fsPaths" :key="p" closable @close="removeFsPath(p)">{{ p }}</n-tag>
          </div>

          <div class="flex gap-2">
            <n-input
              v-model:value="fsPathDraft"
              :placeholder="t('jobs.fields.sourcePathsPlaceholder')"
              @keyup.enter="addFsPathsFromDraft"
            />
            <n-button @click="addFsPathsFromDraft">{{ t('common.add') }}</n-button>
          </div>
        </div>
      </n-form-item>
    </div>

    <n-form-item :label="t('jobs.fields.fsPreScan')">
      <div class="space-y-1">
        <n-switch v-model:value="form.fsPreScan" />
        <div class="text-xs app-text-muted">{{ t('jobs.fields.fsPreScanHelp') }}</div>
      </div>
    </n-form-item>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <n-form-item :label="t('jobs.fields.fsSymlinkPolicy')">
        <n-select v-model:value="form.fsSymlinkPolicy" :options="fsSymlinkPolicyOptions" />
      </n-form-item>
      <n-form-item :label="t('jobs.fields.fsHardlinkPolicy')">
        <n-select v-model:value="form.fsHardlinkPolicy" :options="fsHardlinkPolicyOptions" />
      </n-form-item>
    </div>

    <n-form-item :label="t('jobs.fields.fsErrorPolicy')">
      <div class="space-y-1 w-full">
        <n-select v-model:value="form.fsErrorPolicy" :options="fsErrorPolicyOptions" />
        <div class="text-xs app-text-muted">{{ t('jobs.fields.fsErrorPolicyHelp') }}</div>
      </div>
    </n-form-item>

    <div data-field="fsSnapshotMode">
      <n-form-item
        :label="t('jobs.fields.snapshotMode')"
        :validation-status="fieldErrors.fsSnapshotMode ? 'error' : undefined"
        :feedback="fieldErrors.fsSnapshotMode || undefined"
      >
        <div class="space-y-1 w-full">
          <n-select
            v-model:value="form.fsSnapshotMode"
            :options="snapshotModeOptions"
            @update:value="clearFieldError('fsSnapshotMode')"
          />
          <div class="text-xs app-text-muted">{{ t('jobs.fields.snapshotModeHelp') }}</div>
        </div>
      </n-form-item>
    </div>

    <div v-if="form.fsSnapshotMode !== 'off'" data-field="fsSnapshotProvider">
      <n-form-item :label="t('jobs.fields.snapshotProvider')">
        <div class="space-y-1 w-full">
          <n-select
            v-model:value="form.fsSnapshotProvider"
            :options="snapshotProviderOptions"
            @update:value="clearFieldError('fsSnapshotProvider')"
          />
          <div class="text-xs app-text-muted">{{ t('jobs.fields.snapshotProviderHelp') }}</div>
        </div>
      </n-form-item>
    </div>

    <div data-field="fsConsistencyPolicy">
      <n-form-item :label="t('jobs.fields.consistencyPolicy')">
        <div class="space-y-1 w-full">
          <n-select
            v-model:value="form.fsConsistencyPolicy"
            :options="consistencyPolicyOptions"
            @update:value="clearFieldError('fsConsistencyPolicy')"
          />
          <div class="text-xs app-text-muted">{{ t('jobs.fields.consistencyPolicyHelp') }}</div>
        </div>
      </n-form-item>
    </div>

    <div v-if="form.fsConsistencyPolicy === 'fail'" class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <div data-field="fsConsistencyFailThreshold">
        <n-form-item
          :label="t('jobs.fields.consistencyFailThreshold')"
          :validation-status="fieldErrors.fsConsistencyFailThreshold ? 'error' : undefined"
          :feedback="fieldErrors.fsConsistencyFailThreshold || undefined"
        >
          <div class="space-y-1 w-full">
            <n-input-number
              v-model:value="form.fsConsistencyFailThreshold"
              :min="0"
              :step="1"
              class="w-full"
              @update:value="clearFieldError('fsConsistencyFailThreshold')"
            />
            <div class="text-xs app-text-muted">{{ t('jobs.fields.consistencyFailThresholdHelp') }}</div>
          </div>
        </n-form-item>
      </div>

      <div data-field="fsUploadOnConsistencyFailure">
        <n-form-item :label="t('jobs.fields.uploadOnConsistencyFailure')">
          <div class="space-y-1">
            <n-switch v-model:value="form.fsUploadOnConsistencyFailure" />
            <div class="text-xs app-text-muted">{{ t('jobs.fields.uploadOnConsistencyFailureHelp') }}</div>
          </div>
        </n-form-item>
      </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <n-form-item :label="t('jobs.fields.fsInclude')">
        <div class="space-y-1 w-full">
          <n-input
            v-model:value="form.fsInclude"
            type="textarea"
            :autosize="{ minRows: 2, maxRows: 6 }"
            :placeholder="t('jobs.fields.fsIncludePlaceholder')"
          />
          <div class="text-xs app-text-muted">{{ t('jobs.fields.fsIncludeHelp') }}</div>
        </div>
      </n-form-item>
      <n-form-item :label="t('jobs.fields.fsExclude')">
        <div class="space-y-1 w-full">
          <n-input
            v-model:value="form.fsExclude"
            type="textarea"
            :autosize="{ minRows: 2, maxRows: 6 }"
            :placeholder="t('jobs.fields.fsExcludePlaceholder')"
          />
          <div class="text-xs app-text-muted">{{ t('jobs.fields.fsExcludeHelp') }}</div>
        </div>
      </n-form-item>
    </div>
  </template>

  <template v-else-if="form.jobType === 'sqlite'">
    <div data-field="sqlitePath">
      <n-form-item
        :label="t('jobs.fields.sqlitePath')"
        required
        :validation-status="fieldErrors.sqlitePath ? 'error' : undefined"
        :feedback="fieldErrors.sqlitePath || undefined"
      >
        <div class="space-y-1 w-full">
          <n-input
            v-model:value="form.sqlitePath"
            :placeholder="t('jobs.fields.sqlitePathPlaceholder')"
            @update:value="clearFieldError('sqlitePath')"
          />
          <div v-if="!fieldErrors.sqlitePath" class="text-xs app-text-muted">{{ t('jobs.fields.sqlitePathHelp') }}</div>
        </div>
      </n-form-item>
    </div>

    <n-form-item :label="t('jobs.fields.sqliteIntegrityCheck')">
      <div class="space-y-1">
        <n-switch v-model:value="form.sqliteIntegrityCheck" />
        <div class="text-xs app-text-muted">{{ t('jobs.fields.sqliteIntegrityCheckHelp') }}</div>
      </div>
    </n-form-item>
  </template>

  <template v-else>
    <div data-field="vaultwardenDataDir">
      <n-form-item
        :label="t('jobs.fields.vaultwardenDataDir')"
        required
        :validation-status="fieldErrors.vaultwardenDataDir ? 'error' : undefined"
        :feedback="fieldErrors.vaultwardenDataDir || undefined"
      >
        <div class="space-y-1 w-full">
          <n-input
            v-model:value="form.vaultwardenDataDir"
            :placeholder="t('jobs.fields.vaultwardenDataDirPlaceholder')"
            @update:value="clearFieldError('vaultwardenDataDir')"
          />
          <div v-if="!fieldErrors.vaultwardenDataDir" class="text-xs app-text-muted">
            {{ t('jobs.fields.vaultwardenDataDirHelp') }}
          </div>
        </div>
      </n-form-item>
    </div>

    <div data-field="vaultwardenConsistencyPolicy">
      <n-form-item :label="t('jobs.fields.consistencyPolicy')">
        <div class="space-y-1 w-full">
          <n-select
            v-model:value="form.vaultwardenConsistencyPolicy"
            :options="consistencyPolicyOptions"
            @update:value="clearFieldError('vaultwardenConsistencyPolicy')"
          />
          <div class="text-xs app-text-muted">{{ t('jobs.fields.consistencyPolicyHelp') }}</div>
        </div>
      </n-form-item>
    </div>

    <div v-if="form.vaultwardenConsistencyPolicy === 'fail'" class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <div data-field="vaultwardenConsistencyFailThreshold">
        <n-form-item
          :label="t('jobs.fields.consistencyFailThreshold')"
          :validation-status="fieldErrors.vaultwardenConsistencyFailThreshold ? 'error' : undefined"
          :feedback="fieldErrors.vaultwardenConsistencyFailThreshold || undefined"
        >
          <div class="space-y-1 w-full">
            <n-input-number
              v-model:value="form.vaultwardenConsistencyFailThreshold"
              :min="0"
              :step="1"
              class="w-full"
              @update:value="clearFieldError('vaultwardenConsistencyFailThreshold')"
            />
            <div class="text-xs app-text-muted">{{ t('jobs.fields.consistencyFailThresholdHelp') }}</div>
          </div>
        </n-form-item>
      </div>

      <div data-field="vaultwardenUploadOnConsistencyFailure">
        <n-form-item :label="t('jobs.fields.uploadOnConsistencyFailure')">
          <div class="space-y-1">
            <n-switch v-model:value="form.vaultwardenUploadOnConsistencyFailure" />
            <div class="text-xs app-text-muted">{{ t('jobs.fields.uploadOnConsistencyFailureHelp') }}</div>
          </div>
        </n-form-item>
      </div>
    </div>
  </template>
</template>
