import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'

export type ConfigValueSource = 'cli' | 'env' | 'env_rust_log' | 'db' | 'default'

export type PublicMetadataResponse = {
  public_base_url?: string | null
  source: ConfigValueSource
  command_generation_ready: boolean
}

export const useControlPlaneStore = defineStore('controlPlane', () => {
  async function getPublicMetadata(signal?: AbortSignal): Promise<PublicMetadataResponse> {
    return await apiFetch<PublicMetadataResponse>('/api/control-plane/public-metadata', { signal })
  }

  return {
    getPublicMetadata,
  }
})
