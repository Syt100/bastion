<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NEmpty, NSkeleton, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import { useCommandCenterStore, type CommandCenterItem, type CommandCenterRangePreset } from '@/stores/commandCenter'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { COMMAND_CENTER_RANGE_OPTIONS, parseCommandCenterRangePreset, resolveRouteScope } from '@/lib/commandCenter'
import { scopeToNodeId } from '@/lib/scope'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const message = useMessage()

const ui = useUiStore()
const commandCenter = useCommandCenterStore()

const effectiveScope = computed(() => resolveRouteScope(route, ui.preferredScope))
const rangePreset = computed<CommandCenterRangePreset>(() => parseCommandCenterRangePreset(route.query.range))
const snapshot = computed(() => commandCenter.snapshot)
const primaryItems = computed(() => snapshot.value?.critical_activity.items ?? [])
const secondaryItems = computed(() => snapshot.value?.watchlist.items ?? [])
const showInitialSkeleton = computed(() => commandCenter.loading && !snapshot.value)
const scopeLabel = computed(() => {
  if (effectiveScope.value === 'all') return t('nav.scopePicker.all')
  if (effectiveScope.value === 'hub') return t('nav.scopePicker.hub')
  return scopeToNodeId(effectiveScope.value) ?? effectiveScope.value
})
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

async function refresh(): Promise<void> {
  try {
    await commandCenter.refresh({
      scope: effectiveScope.value,
      range: rangePreset.value,
    })
  } catch (error) {
    message.error(formatToastError(t('errors.fetchCommandCenterFailed'), error, t))
  }
}

function setRange(value: CommandCenterRangePreset): void {
  if (rangePreset.value === value) return
  void router.replace({
    query: {
      ...route.query,
      range: value,
    },
  })
}

function severityTagType(severity: CommandCenterItem['severity']): 'error' | 'warning' | 'info' | 'default' {
  if (severity === 'critical') return 'error'
  if (severity === 'warning') return 'warning'
  return 'info'
}

function openHref(href: string): void {
  void router.push(href)
}

watch([effectiveScope, rangePreset], () => {
  void refresh()
}, { immediate: true })
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('runs.title')" :subtitle="t('runs.subtitle')">
      <div class="flex items-center gap-2 flex-wrap">
        <div class="console-range-picker">
          <n-button
            v-for="option in COMMAND_CENTER_RANGE_OPTIONS"
            :key="option.value"
            size="small"
            :type="rangePreset === option.value ? 'primary' : 'default'"
            :secondary="rangePreset !== option.value"
            @click="setRange(option.value)"
          >
            {{ t(option.labelKey) }}
          </n-button>
        </div>
        <n-button size="small" :loading="commandCenter.loading" @click="refresh">
          {{ t('common.refresh') }}
        </n-button>
      </div>
    </PageHeader>

    <n-card class="app-card console-panel" :bordered="false">
      <div class="console-kicker">{{ t('runs.landing.kicker') }}</div>
      <div class="mt-2 text-xl font-semibold">{{ t('runs.landing.title') }}</div>
      <p class="mt-2 app-text-muted">
        {{ t('runs.landing.body', { scope: scopeLabel }) }}
      </p>
    </n-card>

    <div class="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1.6fr)_minmax(20rem,0.9fr)]">
      <n-card class="app-card console-panel" :bordered="false" :title="t('runs.landing.primaryTitle')">
        <div v-if="showInitialSkeleton" class="space-y-3">
          <div v-for="index in 4" :key="index" class="console-item">
            <n-skeleton text width="50%" />
            <n-skeleton text :repeat="2" />
          </div>
        </div>

        <div v-else-if="primaryItems.length === 0" class="py-4">
          <n-empty :description="t('runs.landing.empty')" />
        </div>

        <div v-else class="space-y-3">
          <article v-for="item in primaryItems" :key="item.id" class="console-item">
            <div class="min-w-0">
              <div class="flex items-start gap-2 flex-wrap">
                <div class="console-item-title">{{ item.title }}</div>
                <n-tag size="small" :bordered="false" :type="severityTagType(item.severity)">
                  {{ t(`commandCenter.severity.${item.severity}`) }}
                </n-tag>
              </div>
              <p class="console-item-summary">{{ item.summary }}</p>
              <div class="console-item-meta">
                <span>{{ formatUnixSeconds(item.occurred_at) }}</span>
                <span>{{ item.scope }}</span>
              </div>
            </div>

            <n-button size="small" tertiary @click="openHref(item.primary_action.href)">
              {{ item.primary_action.label }}
            </n-button>
          </article>
        </div>
      </n-card>

      <n-card class="app-card console-panel console-rail-panel" :bordered="false" :title="t('runs.landing.watchlistTitle')">
        <div v-if="showInitialSkeleton" class="space-y-3">
          <n-skeleton text :repeat="3" />
        </div>

        <div v-else-if="secondaryItems.length === 0" class="py-4">
          <n-empty :description="t('runs.landing.watchlistEmpty')" />
        </div>

        <div v-else class="space-y-3">
          <article v-for="item in secondaryItems" :key="item.id" class="console-item console-item-compact">
            <div class="min-w-0">
              <div class="console-item-title">{{ item.title }}</div>
              <p class="console-item-summary">{{ item.summary }}</p>
              <div class="console-item-meta">
                <span>{{ formatUnixSeconds(item.occurred_at) }}</span>
              </div>
            </div>

            <n-button size="small" quaternary @click="openHref(item.primary_action.href)">
              {{ item.primary_action.label }}
            </n-button>
          </article>
        </div>
      </n-card>
    </div>
  </div>
</template>
