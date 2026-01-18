<template>
  <div class="flex items-center justify-end sm:justify-between gap-2">
    <div class="min-w-0">
      <slot name="left" />
    </div>
    <div class="flex items-center justify-end gap-2 ml-auto">
      <slot />
    </div>
  </div>
</template>

