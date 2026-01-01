export const BREAKPOINT_PX = {
  md: 768,
} as const

export const MQ = {
  mdUp: `(min-width: ${BREAKPOINT_PX.md}px)`,
} as const

