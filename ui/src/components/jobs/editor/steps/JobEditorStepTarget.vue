<script setup lang="ts">
import { NButton, NFormItem, NInput, NInputNumber, NSelect } from 'naive-ui'
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
