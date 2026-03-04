// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'

import type { JobListItem } from '@/stores/jobs'

vi.mock('naive-ui', () => ({
  NButton: defineComponent({
    name: 'NButton',
    setup(_props, { slots, attrs }) {
      return () => h('button', { 'data-stub': 'NButton', ...attrs }, slots.default?.())
    },
  }),
  NCheckbox: defineComponent({
    name: 'NCheckbox',
    props: ['checked'],
    emits: ['update:checked'],
    setup(_props, { slots, attrs }) {
      return () => h('div', { 'data-stub': 'NCheckbox', ...attrs }, slots.default?.())
    },
  }),
  NIcon: defineComponent({
    name: 'NIcon',
    setup(_props, { slots }) {
      return () => h('div', { 'data-stub': 'NIcon' }, slots.default?.())
    },
  }),
  NTag: defineComponent({
    name: 'NTag',
    setup(_props, { slots }) {
      return () => h('span', { 'data-stub': 'NTag' }, slots.default?.())
    },
  }),
}))

vi.mock('@/components/list/OverflowActionsButton.vue', () => ({
  default: defineComponent({
    name: 'OverflowActionsButton',
    emits: ['select'],
    setup(_props, { emit }) {
      return () => h('button', { 'data-testid': 'overflow', onClick: () => emit('select', 'more') }, 'overflow')
    },
  }),
}))

import JobsListRowItem from './JobsListRowItem.vue'

const sampleJob: JobListItem = {
  id: 'job-1',
  name: 'Job 1',
  agent_id: null,
  schedule: null,
  schedule_timezone: 'UTC',
  overlap_policy: 'queue',
  created_at: 1,
  updated_at: 1,
  archived_at: null,
  latest_run_id: null,
  latest_run_status: null,
  latest_run_started_at: null,
  latest_run_ended_at: null,
}

describe('JobsListRowItem', () => {
  it('emits row actions and selection updates', async () => {
    const wrapper = mount(JobsListRowItem, {
      props: {
        job: sampleJob,
        selected: true,
        selectable: true,
        checked: false,
        mainTriggerTestId: 'main-trigger',
        runNowTestId: 'run-now-trigger',
        openDetailsLabel: 'open',
        archivedLabel: 'archived',
        neverRanLabel: 'never',
        runNowLabel: 'run now',
        nodeLabel: 'hub',
        scheduleLabel: 'manual',
        latestRunStatusLabel: null,
        latestRunStatusType: null,
        latestRunStartedAtLabel: null,
        latestRunStartedAtTitle: null,
        runNowLoading: false,
        runNowDisabled: false,
        overflowOptions: [{ label: 'more', key: 'more' }],
      },
    })

    await wrapper.find('[data-testid="main-trigger"]').trigger('click')
    await wrapper.find('[data-testid="run-now-trigger"]').trigger('click')
    wrapper.findComponent({ name: 'NCheckbox' }).vm.$emit('update:checked', true)
    await wrapper.find('[data-testid="overflow"]').trigger('click')

    expect(wrapper.emitted('main-click')).toHaveLength(1)
    expect(wrapper.emitted('run-now')).toHaveLength(1)
    expect(wrapper.emitted('update:checked')?.[0]).toEqual([true])
    expect(wrapper.emitted('overflow-select')?.[0]).toEqual(['more'])
  })
})
