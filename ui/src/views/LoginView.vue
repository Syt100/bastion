<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NForm, NFormItem, NInput, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useAuthStore } from '@/stores/auth'
import { apiFetch } from '@/lib/api'

const router = useRouter()
const message = useMessage()
const auth = useAuthStore()
const { t } = useI18n()

const username = ref('admin')
const password = ref('')
const loading = ref(false)

onMounted(async () => {
  try {
    const status = await apiFetch<{ needs_setup: boolean }>('/api/setup/status')
    if (status.needs_setup) {
      await router.replace('/setup')
    }
  } catch {
    // ignore
  }
})

async function onSubmit(): Promise<void> {
  loading.value = true
  try {
    await auth.login(username.value, password.value)
    await router.push('/')
  } catch {
    message.error(t('errors.loginFailed'))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center p-6">
    <n-card class="w-full max-w-md" :title="t('auth.signIn')">
      <n-form @submit.prevent="onSubmit">
        <n-form-item :label="t('auth.username')">
          <n-input v-model:value="username" autocomplete="username" />
        </n-form-item>
        <n-form-item :label="t('auth.password')">
          <n-input v-model:value="password" type="password" autocomplete="current-password" />
        </n-form-item>
        <div class="flex justify-end">
          <n-button type="primary" attr-type="submit" :loading="loading">{{ t('auth.login') }}</n-button>
        </div>
      </n-form>
    </n-card>
  </div>
</template>
