<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NButton,
  NCard,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NPopconfirm,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useAgentsStore, type AgentListItem, type EnrollmentToken } from '@/stores/agents'
import { useUiStore } from '@/stores/ui'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const agents = useAgentsStore()

const tokenModalOpen = ref<boolean>(false)
const tokenCreating = ref<boolean>(false)
const tokenResult = ref<EnrollmentToken | null>(null)
const ttlSeconds = ref<number>(60 * 60)
const remainingUses = ref<number | null>(null)

const rotateModalOpen = ref<boolean>(false)
const rotateRotating = ref<boolean>(false)
const rotateResult = ref<{ agent_id: string; agent_key: string } | null>(null)

const dateFormatter = computed(
  () =>
    new Intl.DateTimeFormat(ui.locale, {
      dateStyle: 'medium',
      timeStyle: 'medium',
    }),
)

function shortId(value: string): string {
  if (value.length <= 12) return value
  return `${value.slice(0, 8)}…${value.slice(-4)}`
}

function formatUnixSeconds(ts: number | null): string {
  if (!ts) return '-'
  return dateFormatter.value.format(new Date(ts * 1000))
}

async function refresh(): Promise<void> {
  try {
    await agents.refresh()
  } catch {
    message.error(t('errors.fetchAgentsFailed'))
  }
}

function openTokenModal(): void {
  tokenResult.value = null
  ttlSeconds.value = 60 * 60
  remainingUses.value = null
  tokenModalOpen.value = true
}

async function createToken(): Promise<void> {
  tokenCreating.value = true
  try {
    tokenResult.value = await agents.createEnrollmentToken({
      ttlSeconds: ttlSeconds.value,
      remainingUses: remainingUses.value,
    })
    message.success(t('messages.enrollmentTokenCreated'))
  } catch {
    message.error(t('errors.createEnrollmentTokenFailed'))
  } finally {
    tokenCreating.value = false
  }
}

async function copyToClipboard(value: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(value)
    message.success(t('messages.copied'))
  } catch {
    message.error(t('errors.copyFailed'))
  }
}

async function revokeAgent(agentId: string): Promise<void> {
  try {
    await agents.revokeAgent(agentId)
    await refresh()
    message.success(t('messages.agentRevoked'))
  } catch {
    message.error(t('errors.revokeAgentFailed'))
  }
}

async function rotateAgentKey(agentId: string): Promise<void> {
  rotateRotating.value = true
  try {
    rotateResult.value = await agents.rotateAgentKey(agentId)
    rotateModalOpen.value = true
    message.success(t('messages.agentKeyRotated'))
  } catch {
    message.error(t('errors.rotateAgentKeyFailed'))
  } finally {
    rotateRotating.value = false
  }
}

const columns = computed<DataTableColumns<AgentListItem>>(() => [
  {
    title: t('agents.columns.name'),
    key: 'name',
    render: (row) => row.name ?? '-',
  },
  {
    title: t('agents.columns.id'),
    key: 'id',
    render: (row) =>
      h('div', { class: 'flex items-center gap-2' }, [
        h('span', { class: 'font-mono text-xs' }, shortId(row.id)),
        h(
          NButton,
          { quaternary: true, size: 'small', onClick: () => copyToClipboard(row.id) },
          { default: () => t('agents.actions.copy') },
        ),
      ]),
  },
  {
    title: t('agents.columns.status'),
    key: 'status',
    render: (row) => {
      if (row.revoked) {
        return h(NTag, { type: 'error' }, { default: () => t('agents.status.revoked') })
      }
      if (row.online) {
        return h(NTag, { type: 'success' }, { default: () => t('agents.status.online') })
      }
      return h(NTag, null, { default: () => t('agents.status.offline') })
    },
  },
  {
    title: t('agents.columns.lastSeen'),
    key: 'last_seen_at',
    render: (row) => formatUnixSeconds(row.last_seen_at),
  },
  {
    title: t('agents.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NPopconfirm,
              {
                onPositiveClick: () => rotateAgentKey(row.id),
                positiveText: t('agents.actions.rotateKey'),
                negativeText: t('common.cancel'),
                disabled: row.revoked,
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    {
                      tertiary: true,
                      size: 'small',
                      type: 'warning',
                      loading: rotateRotating.value,
                      disabled: row.revoked,
                    },
                    { default: () => t('agents.actions.rotateKey') },
                  ),
                default: () => t('agents.rotateConfirm'),
              },
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => revokeAgent(row.id),
                positiveText: t('agents.actions.revoke'),
                negativeText: t('common.cancel'),
                disabled: row.revoked,
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { tertiary: true, size: 'small', type: 'error', disabled: row.revoked },
                    { default: () => t('agents.actions.revoke') },
                  ),
                default: () => t('agents.revokeConfirm'),
              },
            ),
          ],
        },
      ),
  },
])

onMounted(refresh)
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
      <div>
        <h1 class="text-xl font-semibold">{{ t('agents.title') }}</h1>
        <p class="text-sm opacity-70">{{ t('agents.subtitle') }}</p>
      </div>
      <n-space>
        <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
        <n-button type="primary" @click="openTokenModal">{{ t('agents.newToken') }}</n-button>
      </n-space>
    </div>

    <n-card>
      <n-data-table :loading="agents.loading" :columns="columns" :data="agents.items" />
    </n-card>

    <n-modal v-model:show="tokenModalOpen" preset="card" :title="t('agents.tokenModal.title')">
      <div class="space-y-4">
        <n-form label-placement="top">
          <n-form-item :label="t('agents.tokenModal.ttl')">
            <n-input-number v-model:value="ttlSeconds" :min="60" class="w-full" />
          </n-form-item>
          <n-form-item :label="t('agents.tokenModal.remainingUses')">
            <n-input-number v-model:value="remainingUses" :min="1" clearable class="w-full" />
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="tokenModalOpen = false">{{ t('common.close') }}</n-button>
          <n-button type="primary" :loading="tokenCreating" @click="createToken">
            {{ t('agents.tokenModal.create') }}
          </n-button>
        </n-space>

        <div v-if="tokenResult" class="space-y-2">
          <div class="text-sm opacity-70">{{ t('agents.tokenModal.help') }}</div>

          <n-form label-placement="top">
            <n-form-item :label="t('agents.tokenModal.token')">
              <n-input :value="tokenResult.token" readonly />
            </n-form-item>
            <n-form-item :label="t('agents.tokenModal.expiresAt')">
              <n-input :value="formatUnixSeconds(tokenResult.expires_at)" readonly />
            </n-form-item>
          </n-form>

          <n-space>
            <n-button @click="copyToClipboard(tokenResult.token)">{{ t('agents.actions.copyToken') }}</n-button>
          </n-space>
        </div>
      </div>
    </n-modal>

    <n-modal v-model:show="rotateModalOpen" preset="card" :title="t('agents.rotateModal.title')">
      <div class="space-y-4">
        <div class="text-sm opacity-70">{{ t('agents.rotateModal.help') }}</div>

        <n-form v-if="rotateResult" label-placement="top">
          <n-form-item :label="t('agents.rotateModal.agentKey')">
            <n-input :value="rotateResult.agent_key" readonly />
          </n-form-item>
        </n-form>

        <n-space v-if="rotateResult">
          <n-button @click="copyToClipboard(rotateResult.agent_key)">{{ t('agents.actions.copyKey') }}</n-button>
        </n-space>

        <n-space justify="end">
          <n-button @click="rotateModalOpen = false">{{ t('common.close') }}</n-button>
        </n-space>
      </div>
    </n-modal>
  </div>
</template>
