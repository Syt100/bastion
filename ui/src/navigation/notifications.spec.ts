// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import router from '@/router'
import { NOTIFICATIONS_NAV_ITEMS } from './notifications'

describe('notifications navigation', () => {
  it('keeps nav config and router metadata in sync', () => {
    const seenKeys = new Set<string>()
    const seenRoutes = new Set<string>()

    for (const item of NOTIFICATIONS_NAV_ITEMS) {
      expect(seenKeys.has(item.key)).toBe(false)
      seenKeys.add(item.key)

      expect(seenRoutes.has(item.to)).toBe(false)
      seenRoutes.add(item.to)

      const resolved = router.resolve(item.to)
      expect(resolved.matched.length).toBeGreaterThan(0)
      expect(resolved.meta.titleKey).toBe(item.titleKey)
    }
  })
})

