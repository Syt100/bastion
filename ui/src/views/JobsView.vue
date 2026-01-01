<script setup lang="ts">
import { computed, h, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import {
  NAlert,
  NButton,
  NCard,
  NCode,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NPopconfirm,
  NSelect,
  NSpace,
  NSpin,
  NStep,
  NSteps,
  NSwitch,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type JobListItem, type JobType, type OverlapPolicy, type RunEvent, type RunListItem } from '@/stores/jobs'
import { useOperationsStore, type ConflictPolicy, type Operation, type OperationEvent } from '@/stores/operations'
import { useAgentsStore } from '@/stores/agents'
import { useSecretsStore } from '@/stores/secrets'
import { useUiStore } from '@/stores/ui'
import PageHeader from '@/components/PageHeader.vue'
import { MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'

type FsSymlinkPolicy = 'keep' | 'follow' | 'skip'
type FsHardlinkPolicy = 'copy' | 'keep'
type FsErrorPolicy = 'fail_fast' | 'skip_fail' | 'skip_ok'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const jobs = useJobsStore()
const operations = useOperationsStore()
const agents = useAgentsStore()
const secrets = useSecretsStore()

const isDesktop = useMediaQuery(MQ.mdUp)

const editorStep = ref<number>(1)

const EDITOR_STEPS_TOTAL = 5
const editorStepTitles = computed(() => [
  t('jobs.steps.basics'),
  t('jobs.steps.source'),
  t('jobs.steps.target'),
  t('jobs.steps.security'),
  t('jobs.steps.review'),
])
const editorStepTitle = computed(() => {
  const idx = Math.min(EDITOR_STEPS_TOTAL - 1, Math.max(0, editorStep.value - 1))
  return editorStepTitles.value[idx]
})
const editorStepPercent = computed(() =>
  Math.round((Math.min(EDITOR_STEPS_TOTAL, Math.max(1, editorStep.value)) / EDITOR_STEPS_TOTAL) * 100),
)

const editorOpen = ref<boolean>(false)
const editorMode = ref<'create' | 'edit'>('create')
const editorSaving = ref<boolean>(false)

const runsOpen = ref<boolean>(false)
const runsLoading = ref<boolean>(false)
const runsJobId = ref<string | null>(null)
const runs = ref<RunListItem[]>([])

const runEventsOpen = ref<boolean>(false)
const runEventsLoading = ref<boolean>(false)
const runEventsRunId = ref<string | null>(null)
const runEvents = ref<RunEvent[]>([])
const runEventsWsStatus = ref<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
let runEventsLastSeq = 0
let runEventsSocket: WebSocket | null = null

const restoreOpen = ref<boolean>(false)
const restoreStarting = ref<boolean>(false)
const restoreRunId = ref<string | null>(null)
const restoreDestinationDir = ref<string>('')
const restoreConflictPolicy = ref<ConflictPolicy>('overwrite')

const verifyOpen = ref<boolean>(false)
const verifyStarting = ref<boolean>(false)
const verifyRunId = ref<string | null>(null)

const opOpen = ref<boolean>(false)
const opLoading = ref<boolean>(false)
const opId = ref<string | null>(null)
const op = ref<Operation | null>(null)
const opEvents = ref<OperationEvent[]>([])
let opPollTimer: number | null = null

const form = reactive<{
  id: string | null
  name: string
  node: 'hub' | string
  schedule: string
  overlapPolicy: OverlapPolicy
  jobType: JobType
  encryptionEnabled: boolean
  encryptionKeyName: string
  fsRoot: string
  fsInclude: string
  fsExclude: string
  fsSymlinkPolicy: FsSymlinkPolicy
  fsHardlinkPolicy: FsHardlinkPolicy
  fsErrorPolicy: FsErrorPolicy
  sqlitePath: string
  sqliteIntegrityCheck: boolean
  vaultwardenDataDir: string
  targetType: 'webdav' | 'local_dir'
  webdavBaseUrl: string
  webdavSecretName: string
  localBaseDir: string
  partSizeMiB: number
}>({
  id: null,
  name: '',
  node: 'hub',
  schedule: '',
  overlapPolicy: 'queue',
  jobType: 'filesystem',
  encryptionEnabled: false,
  encryptionKeyName: 'default',
  fsRoot: '',
  fsInclude: '',
  fsExclude: '',
  fsSymlinkPolicy: 'keep',
  fsHardlinkPolicy: 'copy',
  fsErrorPolicy: 'fail_fast',
  sqlitePath: '',
  sqliteIntegrityCheck: false,
  vaultwardenDataDir: '',
  targetType: 'webdav',
  webdavBaseUrl: '',
  webdavSecretName: '',
  localBaseDir: '',
  partSizeMiB: 256,
})

const dateFormatter = computed(
  () =>
    new Intl.DateTimeFormat(ui.locale, {
      dateStyle: 'medium',
      timeStyle: 'medium',
    }),
)

function formatUnixSeconds(ts: number | null): string {
  if (!ts) return '-'
  return dateFormatter.value.format(new Date(ts * 1000))
}

function formatJobNode(agentId: string | null): string {
  if (!agentId) return t('jobs.nodes.hub')
  const agent = agents.items.find((a) => a.id === agentId)
  return agent?.name ?? agentId
}

function formatOverlap(policy: OverlapPolicy): string {
  return policy === 'queue' ? t('jobs.overlap.queue') : t('jobs.overlap.reject')
}

function wsUrl(path: string): string {
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${proto}//${window.location.host}${path}`
}

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function parseLines(text: string): string[] {
  return text
    .split(/\r?\n/g)
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
}

function parseStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value.filter((v): v is string => typeof v === 'string')
}

function normalizeSymlinkPolicy(value: unknown): FsSymlinkPolicy {
  if (value === 'follow') return 'follow'
  if (value === 'skip') return 'skip'
  return 'keep'
}

function normalizeHardlinkPolicy(value: unknown): FsHardlinkPolicy {
  if (value === 'keep') return 'keep'
  return 'copy'
}

function normalizeErrorPolicy(value: unknown): FsErrorPolicy {
  if (value === 'skip_fail') return 'skip_fail'
  if (value === 'skip_ok') return 'skip_ok'
  return 'fail_fast'
}

function openCreate(): void {
  editorMode.value = 'create'
  editorStep.value = 1
  form.id = null
  form.name = ''
  form.node = 'hub'
  form.schedule = ''
  form.overlapPolicy = 'queue'
  form.jobType = 'filesystem'
  form.encryptionEnabled = false
  form.encryptionKeyName = 'default'
  form.fsRoot = ''
  form.fsInclude = ''
  form.fsExclude = ''
  form.fsSymlinkPolicy = 'keep'
  form.fsHardlinkPolicy = 'copy'
  form.fsErrorPolicy = 'fail_fast'
  form.sqlitePath = ''
  form.sqliteIntegrityCheck = false
  form.vaultwardenDataDir = ''
  form.targetType = 'webdav'
  form.webdavBaseUrl = ''
  form.webdavSecretName = ''
  form.localBaseDir = ''
  form.partSizeMiB = 256
  editorOpen.value = true
}

async function openEdit(jobId: string): Promise<void> {
  editorMode.value = 'edit'
  editorStep.value = 1
  editorOpen.value = true
  editorSaving.value = true
  try {
    const job = await jobs.getJob(jobId)
    form.id = job.id
    form.name = job.name
    form.node = job.agent_id ? job.agent_id : 'hub'
    form.schedule = job.schedule ?? ''
    form.overlapPolicy = job.overlap_policy
    form.jobType = job.spec.type

    const pipeline = (job.spec as Record<string, unknown>).pipeline as Record<string, unknown> | undefined
    const enc = pipeline?.encryption as Record<string, unknown> | undefined
    const encType = typeof enc?.type === 'string' ? enc.type : 'none'
    if (encType === 'age_x25519') {
      form.encryptionEnabled = true
      form.encryptionKeyName = typeof enc?.key_name === 'string' ? enc.key_name : 'default'
    } else {
      form.encryptionEnabled = false
      form.encryptionKeyName = 'default'
    }

    const target = (job.spec as Record<string, unknown>).target as Record<string, unknown> | undefined
    const targetType = target?.type === 'local_dir' ? 'local_dir' : 'webdav'
    form.targetType = targetType
    form.webdavBaseUrl = typeof target?.base_url === 'string' ? target.base_url : ''
    form.webdavSecretName = typeof target?.secret_name === 'string' ? target.secret_name : ''
    form.localBaseDir = typeof target?.base_dir === 'string' ? target.base_dir : ''
    form.partSizeMiB =
      typeof target?.part_size_bytes === 'number' && target.part_size_bytes > 0
        ? Math.max(1, Math.round(target.part_size_bytes / (1024 * 1024)))
        : 256

    const source = (job.spec as Record<string, unknown>).source as Record<string, unknown> | undefined
    form.fsRoot = typeof source?.root === 'string' ? source.root : ''
    form.fsInclude = parseStringArray(source?.include).join('\n')
    form.fsExclude = parseStringArray(source?.exclude).join('\n')
    form.fsSymlinkPolicy = normalizeSymlinkPolicy(source?.symlink_policy)
    form.fsHardlinkPolicy = normalizeHardlinkPolicy(source?.hardlink_policy)
    form.fsErrorPolicy = normalizeErrorPolicy(source?.error_policy)
    form.sqlitePath = typeof source?.path === 'string' ? source.path : ''
    form.sqliteIntegrityCheck = typeof source?.integrity_check === 'boolean' ? source.integrity_check : false
    form.vaultwardenDataDir = typeof source?.data_dir === 'string' ? source.data_dir : ''
  } catch {
    message.error(t('errors.fetchJobFailed'))
    editorOpen.value = false
  } finally {
    editorSaving.value = false
  }
}

function validateEditorStep(step: number): boolean {
  if (step >= 1) {
    const name = form.name.trim()
    if (!name) {
      message.error(t('errors.jobNameRequired'))
      return false
    }
  }

  if (step >= 2) {
    if (form.jobType === 'filesystem' && !form.fsRoot.trim()) {
      message.error(t('errors.sourceRootRequired'))
      return false
    }
    if (form.jobType === 'sqlite' && !form.sqlitePath.trim()) {
      message.error(t('errors.sqlitePathRequired'))
      return false
    }
    if (form.jobType === 'vaultwarden' && !form.vaultwardenDataDir.trim()) {
      message.error(t('errors.vaultwardenDataDirRequired'))
      return false
    }
  }

  if (step >= 3) {
    if (form.targetType === 'webdav') {
      if (!form.webdavBaseUrl.trim()) {
        message.error(t('errors.webdavBaseUrlRequired'))
        return false
      }
      if (!form.webdavSecretName.trim()) {
        message.error(t('errors.webdavSecretRequired'))
        return false
      }
    } else {
      if (!form.localBaseDir.trim()) {
        message.error(t('errors.localBaseDirRequired'))
        return false
      }
    }

    if (!Number.isFinite(form.partSizeMiB) || form.partSizeMiB <= 0) {
      message.error(t('errors.partSizeInvalid'))
      return false
    }
  }

  if (step >= 4) {
    const encryptionKeyName = form.encryptionKeyName.trim()
    if (form.encryptionEnabled && !encryptionKeyName) {
      message.error(t('errors.encryptionKeyNameRequired'))
      return false
    }
  }

  return true
}

function prevStep(): void {
  editorStep.value = Math.max(1, editorStep.value - 1)
}

function nextStep(): void {
  if (!validateEditorStep(editorStep.value)) return
  editorStep.value = Math.min(5, editorStep.value + 1)
}

const previewPayload = computed(() => {
  const partSizeMiB = Math.max(1, Math.floor(form.partSizeMiB || 1))
  const partSizeBytes = partSizeMiB * 1024 * 1024

  const pipeline = {
    encryption: form.encryptionEnabled
      ? ({ type: 'age_x25519' as const, key_name: form.encryptionKeyName.trim() || 'default' } as const)
      : ({ type: 'none' as const } as const),
  }

  const source =
    form.jobType === 'filesystem'
      ? {
          root: form.fsRoot.trim(),
          include: parseLines(form.fsInclude),
          exclude: parseLines(form.fsExclude),
          symlink_policy: form.fsSymlinkPolicy,
          hardlink_policy: form.fsHardlinkPolicy,
          error_policy: form.fsErrorPolicy,
        }
      : form.jobType === 'sqlite'
        ? { path: form.sqlitePath.trim(), integrity_check: form.sqliteIntegrityCheck }
        : { data_dir: form.vaultwardenDataDir.trim() }

  const target =
    form.targetType === 'webdav'
      ? ({
          type: 'webdav' as const,
          base_url: form.webdavBaseUrl.trim(),
          secret_name: form.webdavSecretName.trim(),
          part_size_bytes: partSizeBytes,
        } as const)
      : ({
          type: 'local_dir' as const,
          base_dir: form.localBaseDir.trim(),
          part_size_bytes: partSizeBytes,
        } as const)

  return {
    name: form.name.trim(),
    agent_id: form.node === 'hub' ? null : form.node,
    schedule: form.schedule.trim() ? form.schedule.trim() : null,
    overlap_policy: form.overlapPolicy,
    spec: {
      v: 1 as const,
      type: form.jobType,
      pipeline,
      source,
      target,
    },
  }
})

async function save(): Promise<void> {
  if (!validateEditorStep(4)) return

  const name = form.name.trim()
  if (!name) {
    message.error(t('errors.jobNameRequired'))
    return
  }

  const agentId = form.node === 'hub' ? null : form.node

  const partSizeMiB = Math.max(1, Math.floor(form.partSizeMiB))
  const partSizeBytes = partSizeMiB * 1024 * 1024

  const targetType = form.targetType
  const webdavBaseUrl = form.webdavBaseUrl.trim()
  const webdavSecretName = form.webdavSecretName.trim()
  const localBaseDir = form.localBaseDir.trim()

  if (targetType === 'webdav') {
    if (!webdavBaseUrl) {
      message.error(t('errors.webdavBaseUrlRequired'))
      return
    }
    if (!webdavSecretName) {
      message.error(t('errors.webdavSecretRequired'))
      return
    }
  } else {
    if (!localBaseDir) {
      message.error(t('errors.localBaseDirRequired'))
      return
    }
  }

  const encryptionKeyName = form.encryptionKeyName.trim()
  if (form.encryptionEnabled && !encryptionKeyName) {
    message.error(t('errors.encryptionKeyNameRequired'))
    return
  }

  const pipeline = {
    encryption: form.encryptionEnabled
      ? ({ type: 'age_x25519' as const, key_name: encryptionKeyName } as const)
      : ({ type: 'none' as const } as const),
  }

  const source =
    form.jobType === 'filesystem'
      ? {
          root: form.fsRoot.trim(),
          include: parseLines(form.fsInclude),
          exclude: parseLines(form.fsExclude),
          symlink_policy: form.fsSymlinkPolicy,
          hardlink_policy: form.fsHardlinkPolicy,
          error_policy: form.fsErrorPolicy,
        }
      : form.jobType === 'sqlite'
        ? { path: form.sqlitePath.trim(), integrity_check: form.sqliteIntegrityCheck }
        : { data_dir: form.vaultwardenDataDir.trim() }

  if (form.jobType === 'filesystem' && !source.root) {
    message.error(t('errors.sourceRootRequired'))
    return
  }
  if (form.jobType === 'sqlite' && !source.path) {
    message.error(t('errors.sqlitePathRequired'))
    return
  }
  if (form.jobType === 'vaultwarden' && !source.data_dir) {
    message.error(t('errors.vaultwardenDataDirRequired'))
    return
  }

  editorSaving.value = true
  try {
    const target =
      targetType === 'webdav'
        ? ({
            type: 'webdav' as const,
            base_url: webdavBaseUrl,
            secret_name: webdavSecretName,
            part_size_bytes: partSizeBytes,
          } as const)
        : ({
            type: 'local_dir' as const,
            base_dir: localBaseDir,
            part_size_bytes: partSizeBytes,
          } as const)

    const payload = {
      name,
      agent_id: agentId,
      schedule: form.schedule.trim() ? form.schedule.trim() : null,
      overlap_policy: form.overlapPolicy,
      spec: {
        v: 1 as const,
        type: form.jobType,
        pipeline,
        source,
        target,
      },
    }

    if (editorMode.value === 'create') {
      await jobs.createJob(payload)
      message.success(t('messages.jobCreated'))
    } else if (form.id) {
      await jobs.updateJob(form.id, payload)
      message.success(t('messages.jobUpdated'))
    }

    editorOpen.value = false
    await refresh()
  } catch (error) {
    const msg =
      error && typeof error === 'object' && 'message' in error
        ? String((error as { message: unknown }).message)
        : t('errors.saveJobFailed')
    message.error(msg)
  } finally {
    editorSaving.value = false
  }
}

async function refresh(): Promise<void> {
  try {
    await jobs.refresh()
  } catch {
    message.error(t('errors.fetchJobsFailed'))
  }
}

async function removeJob(jobId: string): Promise<void> {
  try {
    await jobs.deleteJob(jobId)
    message.success(t('messages.jobDeleted'))
    await refresh()
  } catch {
    message.error(t('errors.deleteJobFailed'))
  }
}

async function runNow(jobId: string): Promise<void> {
  try {
    const res = await jobs.runNow(jobId)
    if (res.status === 'rejected') {
      message.warning(t('messages.runRejected'))
    } else {
      message.success(t('messages.runQueued'))
    }
  } catch {
    message.error(t('errors.runNowFailed'))
  }
}

async function openRuns(jobId: string): Promise<void> {
  runsOpen.value = true
  runsJobId.value = jobId
  runsLoading.value = true
  runs.value = []
  try {
    runs.value = await jobs.listRuns(jobId)
  } catch {
    message.error(t('errors.fetchRunsFailed'))
  } finally {
    runsLoading.value = false
  }
}

function closeRunEventsSocket(): void {
  if (runEventsSocket) {
    runEventsSocket.close()
    runEventsSocket = null
  }
  runEventsWsStatus.value = 'disconnected'
}

function connectRunEventsWs(runId: string): void {
  closeRunEventsSocket()
  runEventsWsStatus.value = 'connecting'

  const socket = new WebSocket(wsUrl(`/api/runs/${encodeURIComponent(runId)}/events/ws`))
  runEventsSocket = socket

  socket.onopen = () => {
    runEventsWsStatus.value = 'connected'
  }

  socket.onmessage = async (evt: MessageEvent) => {
    let parsed: unknown
    try {
      parsed = JSON.parse(String(evt.data)) as unknown
    } catch {
      return
    }

    if (!parsed || typeof parsed !== 'object') return
    const e = parsed as RunEvent
    if (typeof e.seq !== 'number' || typeof e.ts !== 'number') return
    if (e.seq <= runEventsLastSeq) return

    runEventsLastSeq = e.seq
    runEvents.value.push(e)
    await nextTick()
    const el = document.getElementById('run-events-scroll')
    if (el) el.scrollTop = el.scrollHeight
  }

  socket.onerror = () => {
    runEventsWsStatus.value = 'error'
  }

  socket.onclose = () => {
    runEventsWsStatus.value = 'disconnected'
  }
}

async function openRunEvents(runId: string): Promise<void> {
  runEventsOpen.value = true
  runEventsRunId.value = runId
  runEventsLoading.value = true
  runEvents.value = []
  runEventsLastSeq = 0
  try {
    const events = await jobs.listRunEvents(runId)
    runEvents.value = events
    runEventsLastSeq = events.reduce((m, e) => Math.max(m, e.seq), 0)
    await nextTick()
    const el = document.getElementById('run-events-scroll')
    if (el) el.scrollTop = el.scrollHeight
  } catch {
    message.error(t('errors.fetchRunEventsFailed'))
  } finally {
    runEventsLoading.value = false
  }

  connectRunEventsWs(runId)
}

function openRestoreWizard(runId: string): void {
  restoreRunId.value = runId
  restoreDestinationDir.value = ''
  restoreConflictPolicy.value = 'overwrite'
  restoreOpen.value = true
}

function openVerifyWizard(runId: string): void {
  verifyRunId.value = runId
  verifyOpen.value = true
}

function stopOpPolling(): void {
  if (opPollTimer !== null) {
    window.clearInterval(opPollTimer)
    opPollTimer = null
  }
}

async function refreshOp(): Promise<void> {
  if (!opId.value) return
  const [nextOp, events] = await Promise.all([operations.getOperation(opId.value), operations.listEvents(opId.value)])
  op.value = nextOp
  opEvents.value = events
  if (nextOp.status !== 'running') {
    stopOpPolling()
  }
}

async function openOperation(id: string): Promise<void> {
  opId.value = id
  op.value = null
  opEvents.value = []
  opOpen.value = true
  opLoading.value = true
  try {
    await refreshOp()
  } finally {
    opLoading.value = false
  }

  stopOpPolling()
  opPollTimer = window.setInterval(async () => {
    try {
      await refreshOp()
    } catch {
      stopOpPolling()
    }
  }, 1000)
}

async function startRestore(): Promise<void> {
  const runId = restoreRunId.value
  if (!runId) return

  const destination = restoreDestinationDir.value.trim()
  if (!destination) {
    message.error(t('errors.restoreDestinationRequired'))
    return
  }

  restoreStarting.value = true
  try {
    const id = await operations.startRestore(runId, destination, restoreConflictPolicy.value)
    restoreOpen.value = false
    await openOperation(id)
  } catch (error) {
    const msg =
      error && typeof error === 'object' && 'message' in error
        ? String((error as { message: unknown }).message)
        : t('errors.restoreStartFailed')
    message.error(msg)
  } finally {
    restoreStarting.value = false
  }
}

async function startVerify(): Promise<void> {
  const runId = verifyRunId.value
  if (!runId) return

  verifyStarting.value = true
  try {
    const id = await operations.startVerify(runId)
    verifyOpen.value = false
    await openOperation(id)
  } catch (error) {
    const msg =
      error && typeof error === 'object' && 'message' in error
        ? String((error as { message: unknown }).message)
        : t('errors.verifyStartFailed')
    message.error(msg)
  } finally {
    verifyStarting.value = false
  }
}

const conflictOptions = computed(() => [
  { label: t('restore.conflict.overwrite'), value: 'overwrite' },
  { label: t('restore.conflict.skip'), value: 'skip' },
  { label: t('restore.conflict.fail'), value: 'fail' },
])

const overlapOptions = computed(() => [
  { label: t('jobs.overlap.queue'), value: 'queue' },
  { label: t('jobs.overlap.reject'), value: 'reject' },
])

const nodeOptions = computed(() => [
  { label: t('jobs.nodes.hub'), value: 'hub' },
  ...agents.items.map((a) => ({
    label: a.name ? `${a.name} (${a.id.slice(0, 8)}…)` : a.id,
    value: a.id,
  })),
])

const targetTypeOptions = computed(() => [
  { label: t('jobs.targets.webdav'), value: 'webdav' },
  { label: t('jobs.targets.localDir'), value: 'local_dir' },
])

const jobTypeOptions = computed(() => [
  { label: t('jobs.types.filesystem'), value: 'filesystem' },
  { label: t('jobs.types.sqlite'), value: 'sqlite' },
  { label: t('jobs.types.vaultwarden'), value: 'vaultwarden' },
])

const fsSymlinkPolicyOptions = computed(() => [
  { label: t('jobs.fs.symlink.keep'), value: 'keep' },
  { label: t('jobs.fs.symlink.follow'), value: 'follow' },
  { label: t('jobs.fs.symlink.skip'), value: 'skip' },
])

const fsHardlinkPolicyOptions = computed(() => [
  { label: t('jobs.fs.hardlink.copy'), value: 'copy' },
  { label: t('jobs.fs.hardlink.keep'), value: 'keep' },
])

const fsErrorPolicyOptions = computed(() => [
  { label: t('jobs.fs.error.failFast'), value: 'fail_fast' },
  { label: t('jobs.fs.error.skipFail'), value: 'skip_fail' },
  { label: t('jobs.fs.error.skipOk'), value: 'skip_ok' },
])

function statusTagType(status: RunListItem['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  return 'default'
}

function opStatusTagType(status: Operation['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'running') return 'warning'
  return 'default'
}

function runEventLevelTagType(level: string): 'success' | 'error' | 'warning' | 'default' {
  if (level === 'error') return 'error'
  if (level === 'warn' || level === 'warning') return 'warning'
  if (level === 'info') return 'success'
  return 'default'
}

const columns = computed<DataTableColumns<JobListItem>>(() => [
  { title: t('jobs.columns.name'), key: 'name' },
  {
    title: t('jobs.columns.node'),
    key: 'agent_id',
    render: (row) => {
      return formatJobNode(row.agent_id)
    },
  },
  {
    title: t('jobs.columns.schedule'),
    key: 'schedule',
    render: (row) => row.schedule ?? '-',
  },
  {
    title: t('jobs.columns.overlap'),
    key: 'overlap_policy',
    render: (row) => formatOverlap(row.overlap_policy),
  },
  {
    title: t('jobs.columns.updatedAt'),
    key: 'updated_at',
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('jobs.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NButton,
              { size: 'small', type: 'primary', onClick: () => runNow(row.id) },
              { default: () => t('jobs.actions.runNow') },
            ),
            h(
              NButton,
              { size: 'small', onClick: () => openRuns(row.id) },
              { default: () => t('jobs.actions.runs') },
            ),
            h(NButton, { size: 'small', onClick: () => openEdit(row.id) }, { default: () => t('common.edit') }),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => removeJob(row.id),
                positiveText: t('common.delete'),
                negativeText: t('common.cancel'),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { size: 'small', type: 'error', tertiary: true },
                    { default: () => t('common.delete') },
                  ),
                default: () => t('jobs.deleteConfirm'),
              },
            ),
          ],
        },
      ),
  },
])

const runColumns = computed<DataTableColumns<RunListItem>>(() => [
  {
    title: t('runs.columns.status'),
    key: 'status',
    render: (row) =>
      h(NTag, { type: statusTagType(row.status) }, { default: () => row.status }),
  },
  { title: t('runs.columns.startedAt'), key: 'started_at', render: (row) => formatUnixSeconds(row.started_at) },
  { title: t('runs.columns.endedAt'), key: 'ended_at', render: (row) => formatUnixSeconds(row.ended_at) },
  { title: t('runs.columns.error'), key: 'error', render: (row) => row.error ?? '-' },
  {
    title: t('runs.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(NButton, { size: 'small', onClick: () => openRunEvents(row.id) }, { default: () => t('runs.actions.events') }),
            h(
              NButton,
              { size: 'small', disabled: row.status !== 'success', onClick: () => openRestoreWizard(row.id) },
              { default: () => t('runs.actions.restore') },
            ),
            h(
              NButton,
              { size: 'small', disabled: row.status !== 'success', onClick: () => openVerifyWizard(row.id) },
              { default: () => t('runs.actions.verify') },
            ),
          ],
        },
      ),
  },
])

const webdavSecretOptions = computed(() =>
  secrets.webdav.map((s) => ({ label: s.name, value: s.name })),
)

onMounted(async () => {
  await refresh()
  try {
    await agents.refresh()
  } catch {
    message.error(t('errors.fetchAgentsFailed'))
  }
  try {
    await secrets.refreshWebdav()
  } catch {
    message.error(t('errors.fetchWebdavSecretsFailed'))
  }
})

watch(opOpen, (open) => {
  if (!open) {
    stopOpPolling()
  }
})

watch(runEventsOpen, (open) => {
  if (!open) {
    closeRunEventsSocket()
  }
})

onBeforeUnmount(() => {
  stopOpPolling()
  closeRunEventsSocket()
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('jobs.title')" :subtitle="t('jobs.subtitle')">
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button type="primary" @click="openCreate">{{ t('jobs.actions.create') }}</n-button>
    </PageHeader>

    <div v-if="!isDesktop" class="space-y-3" data-testid="jobs-cards">
      <n-card
        v-if="!jobs.loading && jobs.items.length === 0"
        class="shadow-sm border border-black/5 dark:border-white/10"
      >
        <div class="text-sm opacity-70">{{ t('common.noData') }}</div>
      </n-card>

      <n-card
        v-for="job in jobs.items"
        :key="job.id"
        size="small"
        class="shadow-sm border border-black/5 dark:border-white/10"
      >
        <template #header>
          <div class="flex items-center justify-between gap-3">
            <div class="font-medium truncate">{{ job.name }}</div>
          </div>
        </template>

        <div class="text-sm">
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('jobs.columns.node') }}</div>
            <div class="text-right">{{ formatJobNode(job.agent_id) }}</div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('jobs.columns.schedule') }}</div>
            <div class="text-right">{{ job.schedule ?? '-' }}</div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('jobs.columns.overlap') }}</div>
            <div class="text-right">{{ formatOverlap(job.overlap_policy) }}</div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('jobs.columns.updatedAt') }}</div>
            <div class="text-right">{{ formatUnixSeconds(job.updated_at) }}</div>
          </div>
        </div>

        <template #footer>
          <div class="flex flex-wrap justify-end gap-2">
            <n-button size="small" type="primary" @click="runNow(job.id)">{{ t('jobs.actions.runNow') }}</n-button>
            <n-button size="small" @click="openRuns(job.id)">{{ t('jobs.actions.runs') }}</n-button>
            <n-button size="small" @click="openEdit(job.id)">{{ t('common.edit') }}</n-button>
            <n-popconfirm
              :positive-text="t('common.delete')"
              :negative-text="t('common.cancel')"
              @positive-click="removeJob(job.id)"
            >
              <template #trigger>
                <n-button size="small" type="error" tertiary>{{ t('common.delete') }}</n-button>
              </template>
              {{ t('jobs.deleteConfirm') }}
            </n-popconfirm>
          </div>
        </template>
      </n-card>
    </div>

    <div v-else data-testid="jobs-table">
      <n-card class="shadow-sm border border-black/5 dark:border-white/10">
        <div class="overflow-x-auto">
          <n-data-table :loading="jobs.loading" :columns="columns" :data="jobs.items" />
        </div>
      </n-card>
    </div>

    <n-modal
      v-model:show="editorOpen"
      preset="card"
      :style="{ width: MODAL_WIDTH.lg }"
      :title="editorMode === 'create' ? t('jobs.createTitle') : t('jobs.editTitle')"
    >
      <div class="space-y-4">
        <div v-if="isDesktop">
          <n-steps :current="editorStep" size="small">
            <n-step :title="t('jobs.steps.basics')" />
            <n-step :title="t('jobs.steps.source')" />
            <n-step :title="t('jobs.steps.target')" />
            <n-step :title="t('jobs.steps.security')" />
            <n-step :title="t('jobs.steps.review')" />
          </n-steps>
        </div>
        <div v-else class="space-y-2">
          <div class="flex items-center justify-between gap-3">
            <div class="text-sm font-medium">{{ editorStepTitle }}</div>
            <div class="text-xs opacity-70">{{ t('common.stepOf', { current: editorStep, total: EDITOR_STEPS_TOTAL }) }}</div>
          </div>
          <div class="h-1.5 rounded bg-black/5 dark:bg-white/10 overflow-hidden">
            <div
              class="h-full bg-[var(--n-primary-color)]"
              :style="{ width: `${editorStepPercent}%` }"
            />
          </div>
        </div>

        <n-form label-placement="top">
          <template v-if="editorStep === 1">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
              <n-form-item :label="t('jobs.fields.name')">
                <n-input v-model:value="form.name" />
              </n-form-item>
              <n-form-item :label="t('jobs.fields.node')">
                <n-select v-model:value="form.node" :options="nodeOptions" filterable />
              </n-form-item>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
              <n-form-item :label="t('jobs.fields.type')">
                <n-select v-model:value="form.jobType" :options="jobTypeOptions" />
              </n-form-item>
              <n-form-item :label="t('jobs.fields.overlap')">
                <n-select v-model:value="form.overlapPolicy" :options="overlapOptions" />
              </n-form-item>
            </div>

            <n-form-item :label="t('jobs.fields.schedule')">
              <n-input
                v-model:value="form.schedule"
                :placeholder="t('jobs.fields.schedulePlaceholder')"
              />
              <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.scheduleHelp') }}</div>
            </n-form-item>
          </template>

          <template v-else-if="editorStep === 2">
            <n-alert type="info" :bordered="false">
              {{ t('jobs.steps.sourceHelp') }}
            </n-alert>

            <template v-if="form.jobType === 'filesystem'">
              <n-form-item :label="t('jobs.fields.sourceRoot')">
                <n-input
                  v-model:value="form.fsRoot"
                  :placeholder="t('jobs.fields.sourceRootPlaceholder')"
                />
              </n-form-item>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
                <n-form-item :label="t('jobs.fields.fsSymlinkPolicy')">
                  <n-select v-model:value="form.fsSymlinkPolicy" :options="fsSymlinkPolicyOptions" />
                </n-form-item>
                <n-form-item :label="t('jobs.fields.fsHardlinkPolicy')">
                  <n-select v-model:value="form.fsHardlinkPolicy" :options="fsHardlinkPolicyOptions" />
                </n-form-item>
              </div>
              <n-form-item :label="t('jobs.fields.fsErrorPolicy')">
                <n-select v-model:value="form.fsErrorPolicy" :options="fsErrorPolicyOptions" />
                <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.fsErrorPolicyHelp') }}</div>
              </n-form-item>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
                <n-form-item :label="t('jobs.fields.fsInclude')">
                  <div class="space-y-1 w-full">
                    <n-input
                      v-model:value="form.fsInclude"
                      type="textarea"
                      :autosize="{ minRows: 2, maxRows: 6 }"
                      :placeholder="t('jobs.fields.fsIncludePlaceholder')"
                    />
                    <div class="text-xs opacity-70">{{ t('jobs.fields.fsIncludeHelp') }}</div>
                  </div>
                </n-form-item>
                <n-form-item :label="t('jobs.fields.fsExclude')">
                  <div class="space-y-1 w-full">
                    <n-input
                      v-model:value="form.fsExclude"
                      type="textarea"
                      :autosize="{ minRows: 2, maxRows: 6 }"
                      :placeholder="t('jobs.fields.fsExcludePlaceholder')"
                    />
                    <div class="text-xs opacity-70">{{ t('jobs.fields.fsExcludeHelp') }}</div>
                  </div>
                </n-form-item>
              </div>
            </template>

            <template v-else-if="form.jobType === 'sqlite'">
              <n-form-item :label="t('jobs.fields.sqlitePath')">
                <n-input v-model:value="form.sqlitePath" :placeholder="t('jobs.fields.sqlitePathPlaceholder')" />
              </n-form-item>
              <n-form-item :label="t('jobs.fields.sqliteIntegrityCheck')">
                <div class="space-y-1">
                  <n-switch v-model:value="form.sqliteIntegrityCheck" />
                  <div class="text-xs opacity-70">{{ t('jobs.fields.sqliteIntegrityCheckHelp') }}</div>
                </div>
              </n-form-item>
            </template>

            <template v-else>
              <n-form-item :label="t('jobs.fields.vaultwardenDataDir')">
                <div class="space-y-1">
                  <n-input
                    v-model:value="form.vaultwardenDataDir"
                    :placeholder="t('jobs.fields.vaultwardenDataDirPlaceholder')"
                  />
                  <div class="text-xs opacity-70">{{ t('jobs.fields.vaultwardenDataDirHelp') }}</div>
                </div>
              </n-form-item>
            </template>
          </template>

          <template v-else-if="editorStep === 3">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
              <n-form-item :label="t('jobs.fields.targetType')">
                <n-select v-model:value="form.targetType" :options="targetTypeOptions" />
              </n-form-item>

              <n-form-item :label="t('jobs.fields.partSizeMiB')">
                <n-input-number v-model:value="form.partSizeMiB" :min="1" class="w-full" />
                <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.partSizeMiBHelp') }}</div>
              </n-form-item>
            </div>

            <template v-if="form.targetType === 'webdav'">
              <n-form-item :label="t('jobs.fields.webdavBaseUrl')">
                <n-input v-model:value="form.webdavBaseUrl" :placeholder="t('jobs.fields.webdavBaseUrlPlaceholder')" />
              </n-form-item>
              <n-form-item :label="t('jobs.fields.webdavSecret')">
                <n-select v-model:value="form.webdavSecretName" :options="webdavSecretOptions" filterable />
              </n-form-item>
            </template>
            <template v-else>
              <n-form-item :label="t('jobs.fields.localBaseDir')">
                <div class="space-y-1">
                  <n-input v-model:value="form.localBaseDir" :placeholder="t('jobs.fields.localBaseDirPlaceholder')" />
                  <div class="text-xs opacity-70">{{ t('jobs.fields.localBaseDirHelp') }}</div>
                </div>
              </n-form-item>
            </template>
          </template>

          <template v-else-if="editorStep === 4">
            <n-form-item :label="t('jobs.fields.encryptionEnabled')">
              <div class="space-y-1">
                <n-switch v-model:value="form.encryptionEnabled" />
                <div class="text-xs opacity-70">{{ t('jobs.fields.encryptionHelp') }}</div>
              </div>
            </n-form-item>
            <n-form-item v-if="form.encryptionEnabled" :label="t('jobs.fields.encryptionKeyName')">
              <div class="space-y-1 w-full">
                <n-input
                  v-model:value="form.encryptionKeyName"
                  :placeholder="t('jobs.fields.encryptionKeyNamePlaceholder')"
                />
                <div class="text-xs opacity-70">{{ t('jobs.fields.encryptionKeyNameHelp') }}</div>
              </div>
            </n-form-item>
          </template>

          <template v-else>
            <n-alert type="info" :bordered="false">
              {{ t('jobs.steps.reviewHelp') }}
            </n-alert>
            <n-code class="mt-2" :code="formatJson(previewPayload)" language="json" />
          </template>
        </n-form>

        <n-space justify="space-between">
          <n-button @click="editorOpen = false">{{ t('common.cancel') }}</n-button>
          <n-space>
            <n-button v-if="editorStep > 1" @click="prevStep">{{ t('common.back') }}</n-button>
            <n-button v-if="editorStep < 5" type="primary" @click="nextStep">
              {{ t('common.next') }}
            </n-button>
            <n-button v-else type="primary" :loading="editorSaving" @click="save">
              {{ t('common.save') }}
            </n-button>
          </n-space>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="runsOpen" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('runs.title')">
      <div class="space-y-3">
        <div class="text-sm opacity-70">{{ runsJobId }}</div>
        <n-data-table :loading="runsLoading" :columns="runColumns" :data="runs" />
        <n-space justify="end">
          <n-button @click="runsOpen = false">{{ t('common.close') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="runEventsOpen" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('runEvents.title')">
      <div class="space-y-3">
        <div class="text-sm opacity-70 flex items-center gap-2">
          <span>{{ runEventsRunId }}</span>
          <n-tag size="small" :type="runEventsWsStatus === 'connected' ? 'success' : runEventsWsStatus === 'error' ? 'error' : 'default'">
            {{ t(`runEvents.ws.${runEventsWsStatus}`) }}
          </n-tag>
        </div>

        <n-spin v-if="runEventsLoading" size="small" />

        <div id="run-events-scroll" class="max-h-96 overflow-auto border rounded-md p-2 bg-[var(--n-color)]">
          <div v-if="runEvents.length === 0" class="text-sm opacity-70">{{ t('runEvents.noEvents') }}</div>
          <div v-for="e in runEvents" :key="e.seq" class="font-mono text-xs py-1 border-b last:border-b-0 opacity-90">
            <div class="flex flex-wrap gap-2 items-center">
              <span class="opacity-70">{{ formatUnixSeconds(e.ts) }}</span>
              <n-tag size="tiny" :type="runEventLevelTagType(e.level)">{{ e.level }}</n-tag>
              <span class="opacity-70">{{ e.kind }}</span>
              <span>{{ e.message }}</span>
            </div>
            <n-code v-if="e.fields" class="mt-1" :code="formatJson(e.fields)" language="json" />
          </div>
        </div>

        <n-space justify="end">
          <n-button @click="runEventsOpen = false">{{ t('common.close') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="restoreOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('restore.title')">
      <div class="space-y-4">
        <div class="text-sm opacity-70">{{ restoreRunId }}</div>
        <n-form label-placement="top">
          <n-form-item :label="t('restore.fields.destinationDir')">
            <div class="space-y-1 w-full">
              <n-input v-model:value="restoreDestinationDir" :placeholder="t('restore.fields.destinationDirPlaceholder')" />
              <div class="text-xs opacity-70">{{ t('restore.fields.destinationDirHelp') }}</div>
            </div>
          </n-form-item>
          <n-form-item :label="t('restore.fields.conflictPolicy')">
            <n-select v-model:value="restoreConflictPolicy" :options="conflictOptions" />
          </n-form-item>
        </n-form>
        <n-space justify="end">
          <n-button @click="restoreOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="restoreStarting" @click="startRestore">{{ t('restore.actions.start') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="verifyOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('verify.title')">
      <div class="space-y-4">
        <div class="text-sm opacity-70">{{ verifyRunId }}</div>
        <n-alert type="info" :title="t('verify.helpTitle')">
          {{ t('verify.helpBody') }}
        </n-alert>
        <n-space justify="end">
          <n-button @click="verifyOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="verifyStarting" @click="startVerify">{{ t('verify.actions.start') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="opOpen" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('operations.title')">
      <div class="space-y-4">
        <div class="text-sm opacity-70">{{ opId }}</div>

        <div v-if="op" class="flex items-center gap-2">
          <n-tag :type="opStatusTagType(op.status)">{{ op.status }}</n-tag>
          <span class="text-sm opacity-70">{{ t('operations.kind') }}: {{ op.kind }}</span>
          <span class="text-sm opacity-70">{{ t('operations.startedAt') }}: {{ formatUnixSeconds(op.started_at) }}</span>
          <span v-if="op.ended_at" class="text-sm opacity-70">{{ t('operations.endedAt') }}: {{ formatUnixSeconds(op.ended_at) }}</span>
        </div>

        <n-spin v-if="opLoading" size="small" />

        <n-alert v-if="op?.error" type="error" :title="t('operations.errorTitle')">
          {{ op.error }}
        </n-alert>

        <div v-if="op?.summary" class="space-y-2">
          <div class="text-sm font-medium">{{ t('operations.summary') }}</div>
          <n-code :code="formatJson(op.summary)" language="json" show-line-numbers />
        </div>

        <div class="space-y-2">
          <div class="text-sm font-medium">{{ t('operations.events') }}</div>
          <div class="max-h-80 overflow-auto border rounded-md p-2 bg-[var(--n-color)]">
            <div v-if="opEvents.length === 0" class="text-sm opacity-70">{{ t('operations.noEvents') }}</div>
            <div v-for="e in opEvents" :key="e.seq" class="font-mono text-xs py-1 border-b last:border-b-0 opacity-90">
              <div class="flex flex-wrap gap-2">
                <span class="opacity-70">{{ formatUnixSeconds(e.ts) }}</span>
                <span class="opacity-70">{{ e.level }}</span>
                <span class="opacity-70">{{ e.kind }}</span>
                <span>{{ e.message }}</span>
              </div>
              <n-code v-if="e.fields" class="mt-1" :code="formatJson(e.fields)" language="json" />
            </div>
          </div>
        </div>

        <n-space justify="end">
          <n-button @click="opOpen = false">{{ t('common.close') }}</n-button>
        </n-space>
      </div>
    </n-modal>
  </div>
</template>
