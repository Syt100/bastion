<script setup lang="ts">
import { NFormItem, NInput, NSwitch } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobEditorContext } from '../context'

const { t } = useI18n()

const { form, fieldErrors, clearFieldError, onEncryptionEnabledChanged } = useJobEditorContext()
</script>

<template>
  <div class="space-y-4 app-border-subtle rounded-lg p-3 app-glass-soft">
    <n-form-item :label="t('jobs.fields.encryptionEnabled')">
      <div class="space-y-1">
        <n-switch v-model:value="form.encryptionEnabled" @update:value="onEncryptionEnabledChanged" />
        <div class="text-xs opacity-70">{{ t('jobs.fields.encryptionHelp') }}</div>
      </div>
    </n-form-item>

    <n-form-item
      v-if="form.encryptionEnabled"
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
</template>

