import { inject, type InjectionKey, type Ref } from 'vue'

import type { JobEditorField, JobEditorForm } from './types'

export type JobEditorContext = {
  form: JobEditorForm
  fieldErrors: Record<JobEditorField, string | null>
  lockedNodeId: Ref<'hub' | string | null>

  fsPathDraft: Ref<string>
  showJsonPreview: Ref<boolean>
  previewPayload: Ref<unknown>

  clearFieldError: (field: JobEditorField) => void
  clearAllFieldErrors: () => void

  onJobTypeChanged: () => void
  onTargetTypeChanged: () => void
  onEncryptionEnabledChanged: () => void

  prevStep: () => void
  nextStep: () => void

  openFsPicker: () => void
  openLocalBaseDirPicker: () => void
  addFsPathsFromDraft: () => void
  removeFsPath: (path: string) => void
  clearFsPaths: () => void
}

export const jobEditorContextKey: InjectionKey<JobEditorContext> = Symbol('bastion.jobEditorContext')

export function useJobEditorContext(): JobEditorContext {
  const ctx = inject(jobEditorContextKey, null)
  if (!ctx) {
    throw new Error('JobEditorContext is not provided')
  }
  return ctx
}

