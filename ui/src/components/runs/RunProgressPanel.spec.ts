// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('@/lib/media', async () => {
  const vue = await import('vue')
  return { useMediaQuery: () => vue.ref(true) }
})

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['title', 'percentage', 'processing', 'showIndicator'],
      setup(props, { slots }) {
        return () =>
          vue.h(
            'div',
            {
              'data-stub': name,
              'data-title': (props as { title?: unknown }).title as string | undefined,
              'data-percentage':
                typeof (props as { percentage?: unknown }).percentage === 'number'
                  ? String((props as { percentage?: number }).percentage)
                  : undefined,
              'data-processing':
                typeof (props as { processing?: unknown }).processing === 'boolean'
                  ? String(Boolean((props as { processing?: boolean }).processing))
                  : undefined,
              'data-show-indicator':
                typeof (props as { showIndicator?: unknown }).showIndicator === 'boolean'
                  ? String(Boolean((props as { showIndicator?: boolean }).showIndicator))
                  : undefined,
            },
            [slots.trigger?.(), slots.default?.(), slots.header?.()],
          )
      },
    })

  return {
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NIcon: stub('NIcon'),
    NPopover: stub('NPopover'),
    NProgress: stub('NProgress'),
    NSteps: stub('NSteps'),
    NStep: stub('NStep'),
    NTag: stub('NTag'),
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

import RunProgressPanel from './RunProgressPanel.vue'

describe('RunProgressPanel', () => {
  it('computes a weighted overall percentage during packaging when totals exist', () => {
    const wrapper = mount(RunProgressPanel, {
      props: {
        progress: {
          stage: 'packaging',
          ts: 1,
          done: { files: 1, dirs: 1, bytes: 50 },
          total: { files: 2, dirs: 2, bytes: 100 },
        },
      },
    })

    // Overall = scan(5%) + packaging(45% * 0.5) = 27.5% -> rounded to 28.
    const progress = wrapper.findAll('[data-stub=\"NProgress\"]')[0]!
    expect(progress.attributes('data-percentage')).toBe('28')

    // Shows source totals (from total in non-upload stages).
    expect(wrapper.text()).toContain('100 B')
  })

  it('computes a weighted overall percentage during upload from transfer totals', () => {
    const wrapper = mount(RunProgressPanel, {
      props: {
        progress: {
          stage: 'upload',
          ts: 1,
          done: { files: 0, dirs: 0, bytes: 40 },
          total: { files: 0, dirs: 0, bytes: 100 },
          detail: {
            backup: {
              source_total: { files: 10, dirs: 2, bytes: 200 },
              transfer_total_bytes: 100,
              transfer_done_bytes: 40,
            },
          },
        },
      },
    })

    // Overall = scan(5%) + packaging(45%) + upload(50% * 0.4) = 70%
    const progress = wrapper.findAll('[data-stub=\"NProgress\"]')[0]!
    expect(progress.attributes('data-percentage')).toBe('70')

    // Shows transfer totals.
    expect(wrapper.text()).toContain('40 B')
    expect(wrapper.text()).toContain('100 B')
  })

  it('keeps scan help accessible even when current stage is upload', () => {
    const wrapper = mount(RunProgressPanel, {
      props: {
        progress: {
          stage: 'upload',
          ts: 1,
          done: { files: 0, dirs: 0, bytes: 0 },
          total: { files: 0, dirs: 0, bytes: 1 },
          detail: {
            backup: {
              source_total: { files: 1, dirs: 1, bytes: 1 },
              transfer_total_bytes: 1,
              transfer_done_bytes: 0,
            },
          },
        },
      },
    })

    // Help content should still include scan even when stage is upload.
    expect(wrapper.text()).toContain('runs.progress.help.scanTitle')
    expect(wrapper.text()).toContain('runs.progress.help.packagingTitle')
  })

  it('renders upload at 100% as finished (no current-stage progress bar)', () => {
    const wrapper = mount(RunProgressPanel, {
      props: {
        progress: {
          stage: 'upload',
          ts: 1,
          done: { files: 0, dirs: 0, bytes: 10 },
          total: { files: 0, dirs: 0, bytes: 10 },
          detail: {
            backup: {
              source_total: { files: 1, dirs: 1, bytes: 10 },
              transfer_total_bytes: 10,
              transfer_done_bytes: 10,
            },
          },
        },
      },
    })

    // Only the overall progress bar should remain.
    expect(wrapper.findAll('[data-stub="NProgress"]')).toHaveLength(1)
    expect(wrapper.find('[data-stub="NProgress"]').attributes('data-percentage')).toBe('100')
  })

  it('shows a final transfer speed after completion when live rate is missing', () => {
    const wrapper = mount(RunProgressPanel, {
      props: {
        events: [
          { run_id: 'r1', seq: 1, ts: 1, level: 'info', kind: 'upload', message: 'upload', fields: null },
          { run_id: 'r1', seq: 2, ts: 2, level: 'info', kind: 'complete', message: 'complete', fields: null },
        ],
        progress: {
          stage: 'upload',
          ts: 1,
          done: { files: 0, dirs: 0, bytes: 10 },
          total: { files: 0, dirs: 0, bytes: 10 },
          // No rate_bps on purpose.
          detail: {
            backup: {
              source_total: { files: 1, dirs: 1, bytes: 10 },
              transfer_total_bytes: 10,
              transfer_done_bytes: 10,
            },
          },
        },
      },
    })

    expect(wrapper.text()).toContain('10 B/s')
  })

  it('shows stage durations from stage boundary events', () => {
    const wrapper = mount(RunProgressPanel, {
      props: {
        runStartedAt: 10,
        events: [
          { run_id: 'r1', seq: 1, ts: 20, level: 'info', kind: 'scan', message: 'scan', fields: null },
          { run_id: 'r1', seq: 2, ts: 30, level: 'info', kind: 'packaging', message: 'packaging', fields: null },
          { run_id: 'r1', seq: 3, ts: 50, level: 'info', kind: 'upload', message: 'upload', fields: null },
          { run_id: 'r1', seq: 4, ts: 80, level: 'info', kind: 'complete', message: 'complete', fields: null },
        ],
        progress: {
          stage: 'upload',
          ts: 80,
          done: { files: 0, dirs: 0, bytes: 0 },
          total: { files: 0, dirs: 0, bytes: 1 },
          detail: {
            backup: {
              source_total: { files: 1, dirs: 1, bytes: 1 },
              transfer_total_bytes: 1,
              transfer_done_bytes: 0,
            },
          },
        },
      },
    })

    expect(wrapper.text()).toContain('10s')
    expect(wrapper.text()).toContain('20s')
    expect(wrapper.text()).toContain('30s')
    expect(wrapper.text()).toContain('1m 10s')
  })

  it('indicates a failure stage when the run ends in failed', () => {
    const wrapper = mount(RunProgressPanel, {
      props: {
        runStatus: 'failed',
        events: [
          { run_id: 'r1', seq: 1, ts: 10, level: 'info', kind: 'packaging', message: 'packaging', fields: null },
          { run_id: 'r1', seq: 2, ts: 20, level: 'info', kind: 'upload', message: 'upload', fields: null },
          { run_id: 'r1', seq: 3, ts: 25, level: 'error', kind: 'failed', message: 'failed', fields: null },
        ],
        progress: {
          stage: 'upload',
          ts: 25,
          done: { files: 0, dirs: 0, bytes: 0 },
          total: { files: 0, dirs: 0, bytes: 1 },
        },
      },
    })

    expect(wrapper.text()).toContain('runs.progress.failureStage')
    expect(wrapper.text()).toContain('runs.progress.stages.upload')
  })
})
