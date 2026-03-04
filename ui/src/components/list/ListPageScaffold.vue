<template>
  <div class="app-list-scaffold" data-testid="list-page-scaffold">
    <div v-if="$slots.selection" data-testid="list-page-selection">
      <slot name="selection" />
    </div>

    <div v-if="$slots.toolbar" data-testid="list-page-toolbar">
      <slot name="toolbar" />
    </div>

    <div data-testid="list-page-content">
      <slot name="content" />
    </div>

    <div v-if="$slots.footer" data-testid="list-page-footer">
      <slot name="footer" />
    </div>
  </div>
</template>
