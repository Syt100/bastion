<script setup lang="ts">
const dotIndexes = [0, 1, 2] as const
</script>

<template>
  <span class="app-inline-loading-dots" aria-hidden="true">
    <span
      v-for="i in dotIndexes"
      :key="i"
      class="app-inline-loading-dots__dot"
      :style="{ animationDelay: `${i * 0.14}s` }"
    />
  </span>
</template>

<style scoped>
.app-inline-loading-dots {
  display: inline-flex;
  align-items: center;
  gap: 0.22rem;
}

.app-inline-loading-dots__dot {
  width: 0.34rem;
  height: 0.34rem;
  border-radius: 9999px;
  background: var(--app-text-muted);
  opacity: 0.28;
  animation: app-inline-loading-dot-pulse 1s ease-in-out infinite;
}

@keyframes app-inline-loading-dot-pulse {
  0%,
  80%,
  100% {
    transform: translateY(0);
    opacity: 0.28;
  }

  40% {
    transform: translateY(-0.1rem);
    opacity: 0.82;
  }
}
</style>
