<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NAlert, NButton, NCard, NForm, NFormItem, NInput, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { apiFetch } from '@/lib/api'
import { useSystemStore } from '@/stores/system'
import InsecureHttpBanner from '@/components/InsecureHttpBanner.vue'
import AuthLayout from '@/components/AuthLayout.vue'
import { resolveApiFieldErrors, toApiErrorInfo } from '@/lib/errors'

const router = useRouter()
const message = useMessage()
const { t } = useI18n()
const system = useSystemStore()

const username = ref('admin')
const password = ref('')
const password2 = ref('')
const loading = ref(false)
const errorText = ref<string | null>(null)
const fieldErrors = reactive<{ username?: string; password?: string }>({})

const passwordsMatch = computed(() => password.value === password2.value)

function clearFieldErrors(): void {
  fieldErrors.username = undefined
  fieldErrors.password = undefined
}

async function onSubmit(): Promise<void> {
  clearFieldErrors()

  if (!passwordsMatch.value) {
    errorText.value = t('errors.passwordsDoNotMatch')
    return
  }

  errorText.value = null
  loading.value = true
  try {
    await apiFetch<void>('/api/setup/initialize', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: username.value, password: password.value }),
      expectedStatus: 204,
    })
    message.success(t('messages.initializedPleaseSignIn'))
    await router.push('/login')
  } catch (error) {
    const info = toApiErrorInfo(error, t)
    const mapped = resolveApiFieldErrors(info, { t })
    fieldErrors.username = mapped.username
    fieldErrors.password = mapped.password
    errorText.value = info.message || t('errors.setupFailed')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <AuthLayout>
    <n-card class="app-card" :bordered="false" :segmented="{ content: true }">
      <template #header>
        <div class="space-y-1">
          <div class="text-lg font-semibold">{{ t('auth.initTitle') }}</div>
          <div class="text-sm app-text-muted">{{ t('auth.initSubtitle') }}</div>
        </div>
      </template>

      <InsecureHttpBanner v-if="system.insecureHttp" class="mb-4" />

      <n-alert v-if="errorText" type="error" :bordered="false" class="mb-4">
        {{ errorText }}
      </n-alert>

      <n-form label-placement="top" @submit.prevent="onSubmit">
        <n-form-item
          :label="t('auth.username')"
          :validation-status="fieldErrors.username ? 'error' : undefined"
          :feedback="fieldErrors.username"
        >
          <n-input v-model:value="username" size="large" autocomplete="username" />
        </n-form-item>
        <n-form-item
          :label="t('auth.password')"
          :validation-status="fieldErrors.password ? 'error' : undefined"
          :feedback="fieldErrors.password"
        >
          <n-input v-model:value="password" size="large" type="password" autocomplete="new-password" />
        </n-form-item>
        <n-form-item :label="t('auth.confirmPassword')">
          <n-input v-model:value="password2" size="large" type="password" autocomplete="new-password" />
        </n-form-item>
        <n-button block type="primary" size="large" attr-type="submit" :loading="loading">
          {{ t('auth.initialize') }}
        </n-button>
      </n-form>
    </n-card>
  </AuthLayout>
</template>
