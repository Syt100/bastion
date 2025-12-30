<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NForm, NFormItem, NInput, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { apiFetch } from '@/lib/api'

const router = useRouter()
const message = useMessage()
const { t } = useI18n()

const username = ref('admin')
const password = ref('')
const password2 = ref('')
const loading = ref(false)

const passwordsMatch = computed(() => password.value === password2.value)

async function onSubmit(): Promise<void> {
  if (!passwordsMatch.value) {
    message.error(t('errors.passwordsDoNotMatch'))
    return
  }

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
  } catch {
    message.error(t('errors.setupFailed'))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center p-6">
    <n-card class="w-full max-w-md" :title="t('auth.initTitle')">
      <n-form @submit.prevent="onSubmit">
        <n-form-item :label="t('auth.username')">
          <n-input v-model:value="username" autocomplete="username" />
        </n-form-item>
        <n-form-item :label="t('auth.password')">
          <n-input v-model:value="password" type="password" autocomplete="new-password" />
        </n-form-item>
        <n-form-item :label="t('auth.confirmPassword')">
          <n-input v-model:value="password2" type="password" autocomplete="new-password" />
        </n-form-item>
        <div class="flex justify-end">
          <n-button type="primary" attr-type="submit" :loading="loading">{{ t('auth.initialize') }}</n-button>
        </div>
      </n-form>
    </n-card>
  </div>
</template>
