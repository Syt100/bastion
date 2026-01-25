import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'
import { ensureCsrfToken } from '@/stores/csrf'

export type ConfigValueSource = 'cli' | 'env' | 'env_rust_log' | 'db' | 'default'

export type HubRuntimeConfig = {
  hub_timezone?: string | null
  run_retention_days?: number | null
  incomplete_cleanup_days?: number | null
  log_filter?: string | null
  log_file?: string | null
  log_rotation?: string | null
  log_keep_files?: number | null
  default_backup_retention?: BackupRetentionPolicy | null
}

export type BackupRetentionPolicy = {
  enabled: boolean
  keep_last?: number | null
  keep_days?: number | null
  max_delete_per_tick: number
  max_delete_per_day: number
}

export type HubRuntimeConfigFieldMeta = {
  env: string
  source: ConfigValueSource
  editable: boolean
}

export type HubRuntimeConfigFieldsMeta = {
  bind_host: HubRuntimeConfigFieldMeta
  bind_port: HubRuntimeConfigFieldMeta
  data_dir: HubRuntimeConfigFieldMeta
  insecure_http: HubRuntimeConfigFieldMeta
  trusted_proxies: HubRuntimeConfigFieldMeta
  debug_errors: HubRuntimeConfigFieldMeta

  hub_timezone: HubRuntimeConfigFieldMeta
  run_retention_days: HubRuntimeConfigFieldMeta
  incomplete_cleanup_days: HubRuntimeConfigFieldMeta

  log_filter: HubRuntimeConfigFieldMeta
  log_file: HubRuntimeConfigFieldMeta
  log_rotation: HubRuntimeConfigFieldMeta
  log_keep_files: HubRuntimeConfigFieldMeta
}

export type HubRuntimeConfigEffective = {
  bind_host: string
  bind_port: number
  data_dir: string
  insecure_http: boolean
  trusted_proxies: string[]
  debug_errors: boolean

  hub_timezone: string
  run_retention_days: number
  incomplete_cleanup_days: number

  log_filter: string
  log_file?: string | null
  log_rotation: string
  log_keep_files: number
}

export type HubRuntimeConfigGetResponse = {
  requires_restart: boolean
  effective: HubRuntimeConfigEffective
  saved: HubRuntimeConfig
  fields: HubRuntimeConfigFieldsMeta
}

export const useHubRuntimeConfigStore = defineStore('hubRuntimeConfig', () => {
  async function get(signal?: AbortSignal): Promise<HubRuntimeConfigGetResponse> {
    return await apiFetch<HubRuntimeConfigGetResponse>('/api/settings/hub-runtime-config', { signal })
  }

  async function save(config: HubRuntimeConfig): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>('/api/settings/hub-runtime-config', {
      method: 'PUT',
      expectedStatus: 204,
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify(config),
    })
  }

  return { get, save }
})
