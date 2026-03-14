<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NAlert, NButton, NCard, NEmpty, NSkeleton, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import { useCommandCenterStore, type CommandCenterItem, type CommandCenterRangePreset } from '@/stores/commandCenter'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { COMMAND_CENTER_RANGE_OPTIONS, parseCommandCenterRangePreset, resolveRouteScope } from '@/lib/commandCenter'
import {
  formatCommandCenterScopeLabel,
  presentCommandCenterBlocker,
  presentCommandCenterItem,
} from '@/lib/commandCenterPresentation'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const message = useMessage()

const ui = useUiStore()
const commandCenter = useCommandCenterStore()

const effectiveScope = computed(() => resolveRouteScope(route, ui.preferredScope))
const rangePreset = computed<CommandCenterRangePreset>(() => parseCommandCenterRangePreset(route.query.range))
const snapshot = computed(() => commandCenter.snapshot)
const attentionItems = computed(() => snapshot.value?.attention.items ?? [])
const criticalItems = computed(() => snapshot.value?.critical_activity.items ?? [])
const watchlistItems = computed(() => snapshot.value?.watchlist.items ?? [])
const readiness = computed(() => snapshot.value?.recovery_readiness ?? null)
const showInitialSkeleton = computed(() => commandCenter.loading && !snapshot.value)
const totalAttention = computed(() => attentionItems.value.length)
const totalCriticalActivity = computed(() => criticalItems.value.length)
const totalWatchlist = computed(() => watchlistItems.value.length)
const healthTone = computed(() => {
  if (readiness.value?.overall === 'healthy' && totalAttention.value === 0) return 'success'
  if (readiness.value?.overall === 'empty') return 'default'
  return 'warning'
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

function openHref(href: string): void {
  void router.push(href)
}

function severityTagType(severity: CommandCenterItem['severity']): 'error' | 'warning' | 'info' | 'success' | 'default' {
  if (severity === 'critical') return 'error'
  if (severity === 'warning') return 'warning'
  return 'info'
}

function readinessTagType(overall: string | null | undefined): 'success' | 'warning' | 'default' {
  if (overall === 'healthy') return 'success'
  if (overall === 'degraded') return 'warning'
  return 'default'
}

function formatScopeLabel(scope: string): string {
  return formatCommandCenterScopeLabel(scope, t)
}

function itemTitle(item: CommandCenterItem): string {
  return presentCommandCenterItem(item, t).title
}

function itemSummary(item: CommandCenterItem): string {
  return presentCommandCenterItem(item, t).summary
}

function primaryActionLabel(item: CommandCenterItem): string {
  return presentCommandCenterItem(item, t).primaryActionLabel
}

function secondaryActionLabel(item: CommandCenterItem): string | null {
  return presentCommandCenterItem(item, t).secondaryActionLabel
}

function blockerTitle(blocker: NonNullable<typeof readiness.value>['blockers'][number]): string {
  return presentCommandCenterBlocker(blocker, t).title
}

function blockerSummary(blocker: NonNullable<typeof readiness.value>['blockers'][number]): string {
  return presentCommandCenterBlocker(blocker, t).summary
}

watch([effectiveScope, rangePreset], () => {
  void refresh()
}, { immediate: true })
</script>

<template>
  <div class="console-shell space-y-6">
    <PageHeader :title="t('commandCenter.title')" :subtitle="t('commandCenter.subtitle')">
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

    <section class="console-hero app-card">
      <div class="console-kicker">{{ t('commandCenter.hero.kicker') }}</div>
      <div class="mt-3 flex items-start justify-between gap-6 flex-wrap">
        <div class="max-w-3xl">
          <div class="console-hero-title">{{ t('commandCenter.hero.title') }}</div>
          <p class="console-hero-copy">
            {{
              readiness?.overall === 'healthy' && totalAttention === 0
                ? t('commandCenter.hero.healthy')
                : t('commandCenter.hero.attention')
            }}
          </p>
        </div>

        <div class="console-hero-metrics">
          <div class="console-hero-metric">
            <span class="console-hero-metric-label">{{ t('commandCenter.hero.metrics.attention') }}</span>
            <span class="console-hero-metric-value">
              <n-skeleton v-if="showInitialSkeleton" text width="2rem" />
              <template v-else>{{ totalAttention }}</template>
            </span>
          </div>
          <div class="console-hero-metric">
            <span class="console-hero-metric-label">{{ t('commandCenter.hero.metrics.activity') }}</span>
            <span class="console-hero-metric-value">
              <n-skeleton v-if="showInitialSkeleton" text width="2rem" />
              <template v-else>{{ totalCriticalActivity }}</template>
            </span>
          </div>
          <div class="console-hero-metric">
            <span class="console-hero-metric-label">{{ t('commandCenter.hero.metrics.readiness') }}</span>
            <span class="console-hero-metric-value">
              <n-skeleton v-if="showInitialSkeleton" text width="4rem" />
              <template v-else>{{ t(`commandCenter.readiness.overall.${readiness?.overall ?? 'empty'}`) }}</template>
            </span>
          </div>
        </div>
      </div>

      <div class="mt-4 flex items-center gap-2 flex-wrap">
        <n-tag :type="healthTone" :bordered="false">
          {{ t(`commandCenter.readiness.overall.${readiness?.overall ?? 'empty'}`) }}
        </n-tag>
        <n-tag size="small" :bordered="false">
          {{ t('commandCenter.hero.scope', { scope: formatScopeLabel(snapshot?.scope.effective ?? effectiveScope) }) }}
        </n-tag>
        <n-tag size="small" :bordered="false">
          {{ t('commandCenter.hero.generatedAt', { time: snapshot ? formatUnixSeconds(snapshot.generated_at) : '—' }) }}
        </n-tag>
      </div>
    </section>

    <div class="console-grid">
      <div class="space-y-4">
        <n-card class="app-card console-panel" :bordered="false" :title="t('commandCenter.sections.attention.title')">
          <template #header-extra>
            <n-tag size="small" :bordered="false" :type="totalAttention > 0 ? 'warning' : 'success'">
              {{ totalAttention }}
            </n-tag>
          </template>

          <div v-if="showInitialSkeleton" class="space-y-3">
            <div v-for="index in 3" :key="index" class="console-item">
              <n-skeleton text width="55%" />
              <n-skeleton text :repeat="2" />
            </div>
          </div>

          <div v-else-if="attentionItems.length === 0" class="py-4">
            <n-empty :description="t('commandCenter.empty.attention')" />
          </div>

          <div v-else class="space-y-3">
            <article v-for="item in attentionItems" :key="item.id" class="console-item">
              <div class="min-w-0">
                <div class="flex items-start gap-2 flex-wrap">
                  <div class="console-item-title">{{ itemTitle(item) }}</div>
                  <n-tag size="small" :bordered="false" :type="severityTagType(item.severity)">
                    {{ t(`commandCenter.severity.${item.severity}`) }}
                  </n-tag>
                </div>
                <p class="console-item-summary">{{ itemSummary(item) }}</p>
                <div class="console-item-meta">
                  <span>{{ formatUnixSeconds(item.occurred_at) }}</span>
                  <span>{{ formatScopeLabel(item.scope) }}</span>
                </div>
              </div>

              <div class="console-item-actions">
                <n-button size="small" type="primary" @click="openHref(item.primary_action.href)">
                  {{ primaryActionLabel(item) }}
                </n-button>
                <n-button
                  v-if="item.secondary_action"
                  size="small"
                  quaternary
                  @click="openHref(item.secondary_action.href)"
                >
                  {{ secondaryActionLabel(item) }}
                </n-button>
              </div>
            </article>
          </div>
        </n-card>

        <n-card class="app-card console-panel" :bordered="false" :title="t('commandCenter.sections.activity.title')">
          <template #header-extra>
            <n-tag size="small" :bordered="false">{{ totalCriticalActivity }}</n-tag>
          </template>

          <div v-if="showInitialSkeleton" class="space-y-3">
            <div v-for="index in 4" :key="index" class="console-item">
              <n-skeleton text width="50%" />
              <n-skeleton text />
            </div>
          </div>

          <div v-else-if="criticalItems.length === 0" class="py-4">
            <n-empty :description="t('commandCenter.empty.activity')" />
          </div>

          <div v-else class="space-y-3">
            <article v-for="item in criticalItems" :key="item.id" class="console-item">
              <div class="min-w-0">
                <div class="flex items-start gap-2 flex-wrap">
                  <div class="console-item-title">{{ itemTitle(item) }}</div>
                  <n-tag size="small" :bordered="false" :type="severityTagType(item.severity)">
                    {{ t(`commandCenter.severity.${item.severity}`) }}
                  </n-tag>
                </div>
                <p class="console-item-summary">{{ itemSummary(item) }}</p>
                <div class="console-item-meta">
                  <span>{{ formatUnixSeconds(item.occurred_at) }}</span>
                  <span>{{ formatScopeLabel(item.scope) }}</span>
                </div>
              </div>

              <div class="console-item-actions">
                <n-button size="small" tertiary @click="openHref(item.primary_action.href)">
                  {{ primaryActionLabel(item) }}
                </n-button>
              </div>
            </article>
          </div>
        </n-card>
      </div>

      <div class="space-y-4">
        <n-card class="app-card console-panel console-rail-panel" :bordered="false" :title="t('commandCenter.sections.readiness.title')">
          <template #header-extra>
            <n-tag size="small" :bordered="false" :type="readinessTagType(readiness?.overall)">
              {{ t(`commandCenter.readiness.overall.${readiness?.overall ?? 'empty'}`) }}
            </n-tag>
          </template>

          <div v-if="showInitialSkeleton" class="space-y-3">
            <n-skeleton text :repeat="4" />
          </div>

          <template v-else>
            <div class="console-signal-grid">
              <div class="console-signal">
                <div class="console-signal-label">{{ t('commandCenter.readiness.backup.title') }}</div>
                <div class="console-signal-value">
                  {{
                    readiness?.backup.recent_success_at
                      ? formatUnixSeconds(readiness.backup.recent_success_at)
                      : t('commandCenter.readiness.none')
                  }}
                </div>
                <div class="console-signal-meta">
                  {{ t('commandCenter.readiness.coverage', { covered: readiness?.backup.covered_jobs ?? 0, total: readiness?.backup.active_jobs ?? 0 }) }}
                </div>
              </div>

              <div class="console-signal">
                <div class="console-signal-label">{{ t('commandCenter.readiness.verify.title') }}</div>
                <div class="console-signal-value">
                  {{
                    readiness?.verify.recent_success_at
                      ? formatUnixSeconds(readiness.verify.recent_success_at)
                      : t('commandCenter.readiness.none')
                  }}
                </div>
                <div class="console-signal-meta">
                  {{ t('commandCenter.readiness.coverage', { covered: readiness?.verify.covered_jobs ?? 0, total: readiness?.verify.active_jobs ?? 0 }) }}
                </div>
              </div>
            </div>

            <n-alert
              v-if="readiness?.blockers.length"
              class="mt-4"
              type="warning"
              :bordered="false"
              :title="t('commandCenter.readiness.blockersTitle')"
            >
              <ul class="space-y-2 pl-4 list-disc">
                <li v-for="blocker in readiness.blockers" :key="blocker.kind">
                  <div class="font-medium">{{ blockerTitle(blocker) }}</div>
                  <div class="text-sm app-text-muted">{{ blockerSummary(blocker) }}</div>
                </li>
              </ul>
            </n-alert>
            <div v-else class="mt-4 console-success-note">
              {{
                readiness?.overall === 'empty'
                  ? t('commandCenter.readiness.emptyNote')
                  : t('commandCenter.readiness.healthyNote')
              }}
            </div>
          </template>
        </n-card>

        <n-card class="app-card console-panel console-rail-panel" :bordered="false" :title="t('commandCenter.sections.watchlist.title')">
          <template #header-extra>
            <n-tag size="small" :bordered="false">{{ totalWatchlist }}</n-tag>
          </template>

          <div v-if="showInitialSkeleton" class="space-y-3">
            <n-skeleton text :repeat="3" />
          </div>

          <div v-else-if="watchlistItems.length === 0" class="py-4">
            <n-empty :description="t('commandCenter.empty.watchlist')" />
          </div>

          <div v-else class="space-y-3">
            <article v-for="item in watchlistItems" :key="item.id" class="console-item console-item-compact">
              <div class="min-w-0">
                <div class="console-item-title">{{ itemTitle(item) }}</div>
                <p class="console-item-summary">{{ itemSummary(item) }}</p>
                <div class="console-item-meta">
                  <span>{{ formatUnixSeconds(item.occurred_at) }}</span>
                </div>
              </div>

              <n-button size="small" quaternary @click="openHref(item.primary_action.href)">
                {{ primaryActionLabel(item) }}
              </n-button>
            </article>
          </div>
        </n-card>
      </div>
    </div>
  </div>
</template>
