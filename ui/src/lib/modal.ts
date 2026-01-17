export const MODAL_WIDTH = {
  sm: 'min(560px, calc(100vw - 32px))',
  md: 'min(720px, calc(100vw - 32px))',
  lg: 'min(980px, calc(100vw - 32px))',
} as const

export const MODAL_HEIGHT = {
  // Keep modals within the viewport while leaving room for the app header.
  max: 'calc(100vh - 64px)',
  // "Loose" desktop height to avoid content-driven shrink/grow that makes footers jump.
  desktopLoose: 'min(85vh, calc(100vh - 64px))',
} as const
