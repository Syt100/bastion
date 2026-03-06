<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NAlert, NButton, NCard, NForm, NFormItem, NInput } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useAuthStore } from '@/stores/auth'
import { apiFetch } from '@/lib/api'
import { useSystemStore } from '@/stores/system'
import InsecureHttpBanner from '@/components/InsecureHttpBanner.vue'
import AuthLayout from '@/components/AuthLayout.vue'
import { resolveApiFieldErrors, toApiErrorInfo } from '@/lib/errors'

const router = useRouter()
const auth = useAuthStore()
const system = useSystemStore()
const { t } = useI18n()

const username = ref('admin')
const password = ref('')
const loading = ref(false)
const errorText = ref<string | null>(null)
const fieldErrors = reactive<{ username?: string; password?: string }>({})

function clearFieldErrors(): void {
  fieldErrors.username = undefined
  fieldErrors.password = undefined
}

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
  clearFieldErrors()
  loading.value = true
  try {
    await auth.login(username.value, password.value)
    await router.push('/')
  } catch (error) {
    const info = toApiErrorInfo(error, t)
    const mapped = resolveApiFieldErrors(info, { t })
    fieldErrors.username = mapped.username
    fieldErrors.password = mapped.password
    errorText.value = info.message || t('errors.loginFailed')
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
          <div class="text-lg font-semibold">{{ t('auth.signIn') }}</div>
          <div class="text-sm app-text-muted">{{ t('auth.signInSubtitle') }}</div>
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
          <n-input
            v-model:value="username"
            size="large"
            autocomplete="username"
            :input-props="{ name: 'username', 'aria-label': t('auth.username') }"
          />
        </n-form-item>
        <n-form-item
          :label="t('auth.password')"
          :validation-status="fieldErrors.password ? 'error' : undefined"
          :feedback="fieldErrors.password"
        >
          <n-input
            v-model:value="password"
            size="large"
            type="password"
            autocomplete="current-password"
            :input-props="{ name: 'current-password', 'aria-label': t('auth.password') }"
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

        <div class="mt-4 space-y-1 text-sm app-text-muted">
          <div>{{ t('auth.loginHelp') }}</div>
          <div>{{ t('auth.loginHelpDetail') }}</div>
        </div>
      </n-form>
    </n-card>
  </AuthLayout>
</template>
