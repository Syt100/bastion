<script setup lang="ts">
import JobSnapshotsView from '@/views/JobSnapshotsView.vue'
import JobDetailRetentionView from '@/views/jobs/JobDetailRetentionView.vue'
</script>

<template>
  <div class="space-y-4">
    <JobDetailRetentionView />
    <JobSnapshotsView embedded />
  </div>
</template>
