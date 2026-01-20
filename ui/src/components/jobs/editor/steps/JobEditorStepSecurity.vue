<script setup lang="ts">
import { NFormItem, NInput, NSelect, NSwitch, type SelectOption } from 'naive-ui'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { useJobEditorContext } from '../context'

const { t } = useI18n()

const { form, fieldErrors, clearFieldError, onArtifactFormatChanged, onEncryptionEnabledChanged } = useJobEditorContext()

const artifactFormatOptions = computed<SelectOption[]>(() => [
  { label: t('jobs.fields.artifactFormatArchiveV1'), value: 'archive_v1' },
  { label: t('jobs.fields.artifactFormatRawTreeV1'), value: 'raw_tree_v1' },
])
</script>

<template>
  <div class="space-y-4 app-border-subtle rounded-lg p-3 app-glass-soft">
    <n-form-item :label="t('jobs.fields.artifactFormat')">
      <div class="space-y-1 w-full">
        <n-select
          v-model:value="form.artifactFormat"
          :options="artifactFormatOptions"
          @update:value="onArtifactFormatChanged"
        />
        <div class="text-xs opacity-70">
          {{ form.artifactFormat === 'raw_tree_v1' ? t('jobs.fields.artifactFormatRawTreeHelp') : t('jobs.fields.artifactFormatArchiveHelp') }}
        </div>
      </div>
    </n-form-item>

    <n-form-item :label="t('jobs.fields.encryptionEnabled')">
      <div class="space-y-1">
        <n-switch
          v-model:value="form.encryptionEnabled"
          :disabled="form.artifactFormat === 'raw_tree_v1'"
          @update:value="onEncryptionEnabledChanged"
        />
        <div class="text-xs opacity-70">
          {{
            form.artifactFormat === 'raw_tree_v1'
              ? t('jobs.fields.encryptionDisabledByRawTreeHelp')
              : t('jobs.fields.encryptionHelp')
          }}
        </div>
      </div>
    </n-form-item>

    <div v-if="form.encryptionEnabled && form.artifactFormat !== 'raw_tree_v1'" data-field="encryptionKeyName">
      <n-form-item
        :label="t('jobs.fields.encryptionKeyName')"
        required
        :validation-status="fieldErrors.encryptionKeyName ? 'error' : undefined"
        :feedback="fieldErrors.encryptionKeyName || undefined"
      >
        <div class="space-y-1 w-full">
          <n-input
            v-model:value="form.encryptionKeyName"
            :placeholder="t('jobs.fields.encryptionKeyNamePlaceholder')"
            @update:value="clearFieldError('encryptionKeyName')"
          />
          <div v-if="!fieldErrors.encryptionKeyName" class="text-xs opacity-70">
            {{ t('jobs.fields.encryptionKeyNameHelp') }}
          </div>
        </div>
      </n-form-item>
    </div>
  </div>
</template>
