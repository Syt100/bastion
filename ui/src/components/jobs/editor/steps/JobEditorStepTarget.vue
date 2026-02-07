<script setup lang="ts">
import { NButton, NFormItem, NInput, NInputNumber, NSelect, NSwitch } from 'naive-ui'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { useJobEditorContext } from '../context'

type Option = { label: string; value: string }

defineProps<{
  targetTypeOptions: Array<Option>
  webdavSecretOptions: Array<Option>
}>()

const { t } = useI18n()

const { form, fieldErrors, clearFieldError, onTargetTypeChanged, openLocalBaseDirPicker } = useJobEditorContext()

const manageWebdavSecretsHref = computed(() => `/n/${encodeURIComponent(form.node)}/settings/storage`)

const showWebdavRawTreeDirect = computed(
  () => form.jobType === 'filesystem' && form.targetType === 'webdav' && form.artifactFormat === 'raw_tree_v1',
)

const webdavRawTreeDirectModeOptions = computed(() => [
  { label: t('jobs.webdav.rawTreeDirect.mode.off'), value: 'off' },
  { label: t('jobs.webdav.rawTreeDirect.mode.auto'), value: 'auto' },
  { label: t('jobs.webdav.rawTreeDirect.mode.on'), value: 'on' },
])
</script>

<template>
  <div class="space-y-4 app-border-subtle rounded-lg p-3 app-glass-soft">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <n-form-item :label="t('jobs.fields.targetType')">
        <n-select v-model:value="form.targetType" :options="targetTypeOptions" @update:value="onTargetTypeChanged" />
      </n-form-item>

      <div data-field="partSizeMiB">
        <n-form-item
          :label="t('jobs.fields.partSizeMiB')"
          required
          :validation-status="fieldErrors.partSizeMiB ? 'error' : undefined"
          :feedback="fieldErrors.partSizeMiB || undefined"
        >
          <div class="space-y-1 w-full">
            <n-input-number
              v-model:value="form.partSizeMiB"
              :min="1"
              class="w-full"
              @update:value="clearFieldError('partSizeMiB')"
            />
            <div v-if="!fieldErrors.partSizeMiB" class="text-xs app-text-muted">
              {{ t('jobs.fields.partSizeMiBHelp') }}
            </div>
          </div>
        </n-form-item>
      </div>
    </div>

    <template v-if="form.targetType === 'webdav'">
      <div data-field="webdavBaseUrl">
        <n-form-item
          :label="t('jobs.fields.webdavBaseUrl')"
          required
          :validation-status="fieldErrors.webdavBaseUrl ? 'error' : undefined"
          :feedback="fieldErrors.webdavBaseUrl || undefined"
        >
          <n-input
            v-model:value="form.webdavBaseUrl"
            :placeholder="t('jobs.fields.webdavBaseUrlPlaceholder')"
            @update:value="clearFieldError('webdavBaseUrl')"
          />
        </n-form-item>
      </div>
      <div data-field="webdavSecretName">
        <n-form-item
          :label="t('jobs.fields.webdavSecret')"
          required
          :validation-status="fieldErrors.webdavSecretName ? 'error' : undefined"
          :feedback="fieldErrors.webdavSecretName || undefined"
        >
          <div class="space-y-1 w-full">
            <n-select
              v-model:value="form.webdavSecretName"
              :options="webdavSecretOptions"
              filterable
              @update:value="clearFieldError('webdavSecretName')"
            />
            <div class="flex justify-end">
              <n-button
                size="tiny"
                quaternary
                tag="a"
                :href="manageWebdavSecretsHref"
                target="_blank"
                rel="noopener noreferrer"
              >
                {{ t('jobs.actions.manageWebdavSecrets') }}
              </n-button>
            </div>
          </div>
        </n-form-item>
      </div>

      <div v-if="showWebdavRawTreeDirect" class="app-border-subtle rounded-lg p-3 app-glass-soft">
        <div class="text-sm font-medium">{{ t('jobs.webdav.rawTreeDirect.title') }}</div>
        <div class="mt-2 grid grid-cols-1 md:grid-cols-2 gap-x-4 gap-y-2">
          <div data-field="webdavRawTreeDirectMode">
            <n-form-item
              :label="t('jobs.webdav.rawTreeDirect.modeLabel')"
              :validation-status="fieldErrors.webdavRawTreeDirectMode ? 'error' : undefined"
              :feedback="fieldErrors.webdavRawTreeDirectMode || undefined"
            >
              <div class="space-y-1 w-full">
                <n-select
                  v-model:value="form.webdavRawTreeDirectMode"
                  :options="webdavRawTreeDirectModeOptions"
                  @update:value="clearFieldError('webdavRawTreeDirectMode')"
                />
                <div v-if="!fieldErrors.webdavRawTreeDirectMode" class="text-xs app-text-muted">
                  {{ t('jobs.webdav.rawTreeDirect.modeHelp') }}
                </div>
              </div>
            </n-form-item>
          </div>

          <div v-if="form.webdavRawTreeDirectMode !== 'off'">
            <n-form-item :label="t('jobs.webdav.rawTreeDirect.resumeBySize')">
              <div class="space-y-1">
                <n-switch v-model:value="form.webdavRawTreeDirectResumeBySize" />
                <div class="text-xs app-text-muted">
                  {{ t('jobs.webdav.rawTreeDirect.resumeBySizeHelp') }}
                </div>
              </div>
            </n-form-item>
          </div>
        </div>

        <div v-if="form.webdavRawTreeDirectMode !== 'off'" class="mt-2 space-y-2">
          <div class="text-xs app-text-muted">{{ t('jobs.webdav.rawTreeDirect.limitsHelp') }}</div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-x-4 gap-y-2">
            <div data-field="webdavRawTreeDirectConcurrency">
              <n-form-item
                :label="t('jobs.webdav.rawTreeDirect.concurrency')"
                required
                :validation-status="fieldErrors.webdavRawTreeDirectConcurrency ? 'error' : undefined"
                :feedback="fieldErrors.webdavRawTreeDirectConcurrency || undefined"
              >
                <n-input-number
                  v-model:value="form.webdavRawTreeDirectConcurrency"
                  class="w-full"
                  :min="1"
                  :max="128"
                  @update:value="clearFieldError('webdavRawTreeDirectConcurrency')"
                />
              </n-form-item>
            </div>

            <div data-field="webdavRawTreeDirectPutQps">
              <n-form-item
                :label="t('jobs.webdav.rawTreeDirect.putQps')"
                :validation-status="fieldErrors.webdavRawTreeDirectPutQps ? 'error' : undefined"
                :feedback="fieldErrors.webdavRawTreeDirectPutQps || undefined"
              >
                <n-input-number
                  v-model:value="form.webdavRawTreeDirectPutQps"
                  class="w-full"
                  clearable
                  :min="1"
                  :max="10000"
                  @update:value="clearFieldError('webdavRawTreeDirectPutQps')"
                />
              </n-form-item>
            </div>

            <div data-field="webdavRawTreeDirectHeadQps">
              <n-form-item
                :label="t('jobs.webdav.rawTreeDirect.headQps')"
                :validation-status="fieldErrors.webdavRawTreeDirectHeadQps ? 'error' : undefined"
                :feedback="fieldErrors.webdavRawTreeDirectHeadQps || undefined"
              >
                <n-input-number
                  v-model:value="form.webdavRawTreeDirectHeadQps"
                  class="w-full"
                  clearable
                  :min="1"
                  :max="10000"
                  @update:value="clearFieldError('webdavRawTreeDirectHeadQps')"
                />
              </n-form-item>
            </div>

            <div data-field="webdavRawTreeDirectMkcolQps">
              <n-form-item
                :label="t('jobs.webdav.rawTreeDirect.mkcolQps')"
                :validation-status="fieldErrors.webdavRawTreeDirectMkcolQps ? 'error' : undefined"
                :feedback="fieldErrors.webdavRawTreeDirectMkcolQps || undefined"
              >
                <n-input-number
                  v-model:value="form.webdavRawTreeDirectMkcolQps"
                  class="w-full"
                  clearable
                  :min="1"
                  :max="10000"
                  @update:value="clearFieldError('webdavRawTreeDirectMkcolQps')"
                />
              </n-form-item>
            </div>

            <div data-field="webdavRawTreeDirectBurst">
              <n-form-item
                :label="t('jobs.webdav.rawTreeDirect.burst')"
                :validation-status="fieldErrors.webdavRawTreeDirectBurst ? 'error' : undefined"
                :feedback="fieldErrors.webdavRawTreeDirectBurst || undefined"
              >
                <n-input-number
                  v-model:value="form.webdavRawTreeDirectBurst"
                  class="w-full"
                  clearable
                  :min="1"
                  :max="100000"
                  @update:value="clearFieldError('webdavRawTreeDirectBurst')"
                />
              </n-form-item>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template v-else>
      <div data-field="localBaseDir">
        <n-form-item
          :label="t('jobs.fields.localBaseDir')"
          required
          :validation-status="fieldErrors.localBaseDir ? 'error' : undefined"
          :feedback="fieldErrors.localBaseDir || undefined"
        >
          <div class="space-y-1 w-full">
            <div class="flex gap-2">
              <n-input
                v-model:value="form.localBaseDir"
                class="flex-1"
                :placeholder="t('jobs.fields.localBaseDirPlaceholder')"
                @update:value="clearFieldError('localBaseDir')"
              />
              <n-button secondary @click="openLocalBaseDirPicker">{{ t('common.browse') }}</n-button>
            </div>
            <div v-if="!fieldErrors.localBaseDir" class="text-xs app-text-muted">
              {{ t('jobs.fields.localBaseDirHelp') }}
            </div>
          </div>
        </n-form-item>
      </div>
    </template>
  </div>
</template>
