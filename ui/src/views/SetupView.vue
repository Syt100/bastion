<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NForm, NFormItem, NInput, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { apiFetch } from '@/lib/api'
import { useSystemStore } from '@/stores/system'
import InsecureHttpBanner from '@/components/InsecureHttpBanner.vue'
import AuthLayout from '@/components/AuthLayout.vue'

const router = useRouter()
const message = useMessage()
const { t } = useI18n()
const system = useSystemStore()

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
  <AuthLayout>
    <n-card class="shadow-sm border border-black/5 dark:border-white/10" :segmented="{ content: true }">
      <template #header>
        <div class="space-y-1">
          <div class="text-lg font-semibold">{{ t('auth.initTitle') }}</div>
          <div class="text-sm opacity-70">{{ t('auth.initSubtitle') }}</div>
        </div>
      </template>

      <InsecureHttpBanner v-if="system.insecureHttp" class="mb-4" />

      <n-form label-placement="top" @submit.prevent="onSubmit">
        <n-form-item :label="t('auth.username')">
          <n-input v-model:value="username" size="large" autocomplete="username" />
        </n-form-item>
        <n-form-item :label="t('auth.password')">
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
