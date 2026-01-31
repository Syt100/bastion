<script setup lang="ts">
import { computed } from 'vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'
import { useI18n } from 'vue-i18n'

use([CanvasRenderer, LineChart, GridComponent, TooltipComponent, LegendComponent])

const { t } = useI18n()

const props = defineProps<{
  days: string[]
  success: number[]
  failed: number[]
}>()

const option = computed(() => {
  const successName = t('dashboard.chart.success')
  const failedName = t('dashboard.chart.failed')

  return {
    tooltip: { trigger: 'axis' },
    legend: { data: [successName, failedName] },
    grid: { left: 24, right: 24, top: 24, bottom: 24, containLabel: true },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: props.days,
      axisLabel: { formatter: (value: string) => String(value).slice(5) },
    },
    yAxis: { type: 'value', minInterval: 1 },
    series: [
      {
        name: successName,
        type: 'line',
        smooth: true,
        data: props.success,
        areaStyle: { opacity: 0.08 },
      },
      {
        name: failedName,
        type: 'line',
        smooth: true,
        data: props.failed,
        areaStyle: { opacity: 0.08 },
      },
    ],
  } as const
})
</script>

<template>
  <VChart :option="option" autoresize />
</template>
