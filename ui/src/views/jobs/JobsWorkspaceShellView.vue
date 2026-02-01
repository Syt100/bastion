<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NInput, NSelect, NSwitch, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import ListToolbar from '@/components/list/ListToolbar.vue'
import { useJobsStore, type JobListItem } from '@/stores/jobs'
import { useAgentsStore } from '@/stores/agents'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatToastError } from '@/lib/errors'
import JobEditorModal, { type JobEditorModalExpose } from '@/components/jobs/JobEditorModal.vue'

type JobSortKey = 'updated_desc' | 'updated_asc' | 'name_asc' | 'name_desc'

const { t } = useI18n()
const message = useMessage()
const route = useRoute()
const router = useRouter()

const isDesktop = useMediaQuery(MQ.mdUp)

const jobs = useJobsStore()
const agents = useAgentsStore()

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : 'hub'))
const selectedJobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))

const editorModal = ref<JobEditorModalExpose | null>(null)

const showArchived = ref<boolean>(false)
const searchText = ref<string>('')
const sortKey = ref<JobSortKey>('updated_desc')

const sortOptions = computed(() => [
  { label: t('jobs.sort.updatedDesc'), value: 'updated_desc' },
  { label: t('jobs.sort.updatedAsc'), value: 'updated_asc' },
  { label: t('jobs.sort.nameAsc'), value: 'name_asc' },
  { label: t('jobs.sort.nameDesc'), value: 'name_desc' },
])

const nodeScopedJobs = computed<JobListItem[]>(() => {
  const id = nodeId.value
  if (id === 'hub') return jobs.items.filter((j) => j.agent_id === null)
  return jobs.items.filter((j) => j.agent_id === id)
})

const filteredJobs = computed<JobListItem[]>(() => {
  const q = searchText.value.trim().toLowerCase()
  const list = nodeScopedJobs.value.filter((j) => {
    if (!q) return true
    return j.name.toLowerCase().includes(q) || j.id.toLowerCase().includes(q)
  })

  const sorted = list.slice()
  sorted.sort((a, b) => {
    if (sortKey.value === 'updated_asc') return a.updated_at - b.updated_at
    if (sortKey.value === 'updated_desc') return b.updated_at - a.updated_at
    if (sortKey.value === 'name_asc') return a.name.localeCompare(b.name)
    if (sortKey.value === 'name_desc') return b.name.localeCompare(a.name)
    return 0
  })
  return sorted
})

async function refresh(): Promise<void> {
  try {
    await jobs.refresh({ includeArchived: showArchived.value })
  } catch (error) {
    message.error(formatToastError(t('errors.fetchJobsFailed'), error, t))
  }
}

function clearFilters(): void {
  searchText.value = ''
  showArchived.value = false
  sortKey.value = 'updated_desc'
}

function openCreate(): void {
  editorModal.value?.openCreate({ nodeId: nodeId.value })
}

function openJob(jobId: string): void {
  void router.push(`/n/${encodeURIComponent(nodeId.value)}/jobs/${encodeURIComponent(jobId)}/overview`)
}

function isSelected(jobId: string): boolean {
  return selectedJobId.value === jobId
}

function formatNodeLabel(agentId: string | null): string {
  if (!agentId) return t('jobs.nodes.hub')
  const agent = agents.items.find((a) => a.id === agentId)
  return agent?.name ?? agentId
}

onMounted(async () => {
  await refresh()
  try {
    // Ensure node context labels are friendly (agent name vs id).
    await agents.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  }
})

watch(showArchived, () => void refresh())
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      v-if="isDesktop || !selectedJobId"
      :title="t('jobs.title')"
      :subtitle="t('jobs.subtitle')"
    >
      <template #titleSuffix>
        <NodeContextTag :node-id="nodeId" />
      </template>

      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button type="primary" @click="openCreate">{{ t('jobs.actions.create') }}</n-button>
    </PageHeader>

    <template v-if="isDesktop">
      <div class="grid grid-cols-1 gap-4 md:grid-cols-[minmax(0,360px)_minmax(0,1fr)]">
        <n-card class="app-card" :bordered="false">
          <ListToolbar compact embedded>
            <template #search>
              <n-input
                v-model:value="searchText"
                size="small"
                clearable
                :placeholder="t('jobs.filters.searchPlaceholder')"
              />
            </template>

            <template #filters>
              <div class="flex items-center gap-2 w-full md:w-auto">
                <span class="text-sm opacity-70">{{ t('jobs.showArchived') }}</span>
                <n-switch v-model:value="showArchived" />
              </div>
            </template>

            <template #sort>
              <div class="w-full md:w-56 md:flex-none">
                <n-select v-model:value="sortKey" size="small" :options="sortOptions" />
              </div>
            </template>

            <template #actions>
              <n-button size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
            </template>
          </ListToolbar>

          <div class="mt-3">
            <AppEmptyState v-if="jobs.loading && filteredJobs.length === 0" :title="t('common.loading')" loading />
            <AppEmptyState
              v-else-if="!jobs.loading && filteredJobs.length === 0"
              :title="jobs.items.length === 0 ? t('jobs.empty.title') : t('common.noData')"
              :description="jobs.items.length === 0 ? t('jobs.empty.description') : undefined"
            >
              <template #actions>
                <n-button v-if="jobs.items.length === 0" type="primary" size="small" @click="openCreate">
                  {{ t('jobs.actions.create') }}
                </n-button>
                <n-button v-else size="small" @click="clearFilters">
                  {{ t('common.clear') }}
                </n-button>
              </template>
            </AppEmptyState>

            <div v-else class="divide-y divide-black/5 dark:divide-white/10">
              <button
                v-for="job in filteredJobs"
                :key="job.id"
                type="button"
                class="app-list-row"
                :class="isSelected(job.id) ? 'bg-[var(--app-primary-soft)]' : ''"
                @click="openJob(job.id)"
              >
                <div class="min-w-0">
                  <div class="flex items-center gap-2 min-w-0">
                    <div class="font-medium truncate">{{ job.name }}</div>
                    <n-tag v-if="job.archived_at" size="small" :bordered="false" type="warning">
                      {{ t('jobs.archived') }}
                    </n-tag>
                  </div>
                  <div class="mt-1 flex items-center gap-2 min-w-0 text-xs opacity-70">
                    <n-tag size="small" :bordered="false" :type="job.agent_id ? 'default' : 'info'">
                      {{ formatNodeLabel(job.agent_id) }}
                    </n-tag>
                    <span class="min-w-0 truncate">{{ job.schedule ?? t('jobs.scheduleMode.manual') }}</span>
                  </div>
                </div>
              </button>
            </div>
          </div>
        </n-card>

        <div class="min-w-0">
          <router-view v-if="selectedJobId" />
          <AppEmptyState
            v-else
            :title="t('jobs.workspace.emptyTitle')"
            :description="t('jobs.workspace.emptyDescription')"
          />
        </div>
      </div>
    </template>

    <template v-else>
      <div v-if="!selectedJobId" class="space-y-4">
        <ListToolbar>
          <template #search>
            <n-input
              v-model:value="searchText"
              size="small"
              clearable
              :placeholder="t('jobs.filters.searchPlaceholder')"
            />
          </template>

          <template #filters>
            <div class="flex items-center gap-2 w-full md:w-auto">
              <span class="text-sm opacity-70">{{ t('jobs.showArchived') }}</span>
              <n-switch v-model:value="showArchived" />
            </div>
          </template>

          <template #sort>
            <div class="w-full md:w-56 md:flex-none">
              <n-select v-model:value="sortKey" size="small" :options="sortOptions" />
            </div>
          </template>

          <template #actions>
            <n-button size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
          </template>
        </ListToolbar>

        <AppEmptyState v-if="jobs.loading && filteredJobs.length === 0" :title="t('common.loading')" loading />
        <AppEmptyState
          v-else-if="!jobs.loading && filteredJobs.length === 0"
          :title="jobs.items.length === 0 ? t('jobs.empty.title') : t('common.noData')"
          :description="jobs.items.length === 0 ? t('jobs.empty.description') : undefined"
        >
          <template #actions>
            <n-button v-if="jobs.items.length === 0" type="primary" size="small" @click="openCreate">
              {{ t('jobs.actions.create') }}
            </n-button>
            <n-button v-else size="small" @click="clearFilters">
              {{ t('common.clear') }}
            </n-button>
          </template>
        </AppEmptyState>

        <n-card v-else class="app-card" :bordered="false">
          <div class="divide-y divide-black/5 dark:divide-white/10">
            <button
              v-for="job in filteredJobs"
              :key="job.id"
              type="button"
              class="app-list-row"
              @click="openJob(job.id)"
            >
              <div class="min-w-0">
                <div class="flex items-center gap-2 min-w-0">
                  <div class="font-medium truncate">{{ job.name }}</div>
                  <n-tag v-if="job.archived_at" size="small" :bordered="false" type="warning">
                    {{ t('jobs.archived') }}
                  </n-tag>
                </div>
                <div class="mt-1 flex items-center gap-2 min-w-0 text-xs opacity-70">
                  <n-tag size="small" :bordered="false" :type="job.agent_id ? 'default' : 'info'">
                    {{ formatNodeLabel(job.agent_id) }}
                  </n-tag>
                  <span class="min-w-0 truncate">{{ job.schedule ?? t('jobs.scheduleMode.manual') }}</span>
                </div>
              </div>
            </button>
          </div>
        </n-card>
      </div>

      <router-view v-else />
    </template>

    <JobEditorModal ref="editorModal" @saved="refresh" />
  </div>
</template>
