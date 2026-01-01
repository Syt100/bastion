<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NAlert, NButton, NCard, NForm, NFormItem, NInput } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useAuthStore } from '@/stores/auth'
import { apiFetch } from '@/lib/api'
import { useSystemStore } from '@/stores/system'
import InsecureHttpBanner from '@/components/InsecureHttpBanner.vue'
import AuthLayout from '@/components/AuthLayout.vue'
import { toApiErrorInfo } from '@/lib/errors'

const router = useRouter()
const auth = useAuthStore()
const system = useSystemStore()
const { t } = useI18n()

const username = ref('admin')
const password = ref('')
const loading = ref(false)
const errorText = ref<string | null>(null)

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
  errorText.value = null
  loading.value = true
  try {
    await auth.login(username.value, password.value)
    await router.push('/')
  } catch (error) {
    errorText.value = toApiErrorInfo(error, t).message || t('errors.loginFailed')
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
          <div class="text-lg font-semibold">{{ t('auth.signIn') }}</div>
          <div class="text-sm opacity-70">{{ t('auth.signInSubtitle') }}</div>
        </div>
      </template>

      <InsecureHttpBanner v-if="system.insecureHttp" class="mb-4" />

      <n-alert v-if="errorText" type="error" :bordered="false" class="mb-4">
        {{ errorText }}
      </n-alert>

      <n-form label-placement="top" @submit.prevent="onSubmit">
        <n-form-item :label="t('auth.username')">
          <n-input v-model:value="username" size="large" autocomplete="username" />
        </n-form-item>
        <n-form-item :label="t('auth.password')">
          <n-input
            v-model:value="password"
            size="large"
            type="password"
            autocomplete="current-password"
          />
        </n-form-item>
        <n-button
          block
          type="primary"
          size="large"
          attr-type="submit"
          :loading="loading"
        >
          {{ t('auth.login') }}
        </n-button>
      </n-form>
    </n-card>
  </AuthLayout>
</template>
