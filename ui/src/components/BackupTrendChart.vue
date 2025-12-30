<script setup lang="ts">
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'

use([CanvasRenderer, LineChart, GridComponent, TooltipComponent, LegendComponent])

const option = {
  tooltip: { trigger: 'axis' },
  legend: { data: ['Success', 'Failed'] },
  grid: { left: 24, right: 24, top: 24, bottom: 24, containLabel: true },
  xAxis: {
    type: 'category',
    boundaryGap: false,
    data: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
  },
  yAxis: { type: 'value' },
  series: [
    {
      name: 'Success',
      type: 'line',
      smooth: true,
      data: [2, 4, 3, 6, 4, 7, 5],
    },
    {
      name: 'Failed',
      type: 'line',
      smooth: true,
      data: [0, 1, 0, 0, 1, 0, 0],
    },
  ],
} as const
</script>

<template>
  <VChart :option="option" autoresize />
</template>

