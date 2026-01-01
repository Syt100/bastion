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

const option = computed(
  () =>
    ({
      tooltip: { trigger: 'axis' },
      legend: { data: [t('dashboard.chart.success'), t('dashboard.chart.failed')] },
      grid: { left: 24, right: 24, top: 24, bottom: 24, containLabel: true },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: [
          t('dashboard.chart.mon'),
          t('dashboard.chart.tue'),
          t('dashboard.chart.wed'),
          t('dashboard.chart.thu'),
          t('dashboard.chart.fri'),
          t('dashboard.chart.sat'),
          t('dashboard.chart.sun'),
        ],
      },
      yAxis: { type: 'value' },
      series: [
        {
          name: t('dashboard.chart.success'),
          type: 'line',
          smooth: true,
          data: [2, 4, 3, 6, 4, 7, 5],
        },
        {
          name: t('dashboard.chart.failed'),
          type: 'line',
          smooth: true,
          data: [0, 1, 0, 0, 1, 0, 0],
        },
      ],
    }) as const,
)
</script>

<template>
  <VChart :option="option" autoresize />
</template>
