import { inject, type ComputedRef, type InjectionKey, type Ref } from 'vue'

import type { JobDetail, JobWorkspaceDetail } from '@/stores/jobs'

export type JobDetailContext = {
  nodeId: ComputedRef<string>
  jobId: ComputedRef<string | null>
  job: Ref<JobDetail | null>
  workspace: Ref<JobWorkspaceDetail | null>
  loading: Ref<boolean>
  refresh: () => Promise<void>
}

export const JOB_DETAIL_CONTEXT: InjectionKey<JobDetailContext> = Symbol('job-detail')

export function useJobDetailContext(): JobDetailContext {
  const ctx = inject(JOB_DETAIL_CONTEXT)
  if (!ctx) throw new Error('JobDetailContext is not available')
  return ctx
}
