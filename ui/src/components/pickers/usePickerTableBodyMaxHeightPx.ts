import { nextTick, ref, watch, type Ref } from 'vue'

import { useObservedElementHeightPx } from '@/lib/resizeObserver'

type Options = {
  onOpen?: () => void
  onClose?: () => void
}

function computeTableBodyMaxHeightPx(container: HTMLElement): number {
  const containerHeight = container.clientHeight
  const headerEl = container.querySelector('.n-data-table-base-table-header') as HTMLElement | null
  const theadEl = container.querySelector('thead') as HTMLElement | null
  const headerHeight = headerEl?.clientHeight || theadEl?.clientHeight || 0
  return containerHeight - headerHeight
}

export function usePickerTableBodyMaxHeightPx(show: Ref<boolean>, options: Options = {}) {
  const tableContainerEl = ref<HTMLElement | null>(null)
  const { heightPx, start, stop, measure } = useObservedElementHeightPx(tableContainerEl, computeTableBodyMaxHeightPx)

  watch(show, (open) => {
    if (!open) {
      options.onClose?.()
      stop()
      return
    }

    nextTick().then(() => {
      options.onOpen?.()
      start()
      requestAnimationFrame(() => {
        measure()
        requestAnimationFrame(() => {
          measure()
        })
      })
    })
  })

  return { tableContainerEl, tableBodyMaxHeightPx: heightPx, measureTableHeight: measure }
}

