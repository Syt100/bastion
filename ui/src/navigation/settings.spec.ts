// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import router from '@/router'
import { SETTINGS_NAV_ITEMS, getSettingsOverviewItems, getSettingsSidebarItems } from './settings'

describe('settings navigation', () => {
  it('keeps overview items included in sidebar', () => {
    const sidebar = new Set(getSettingsSidebarItems().map((i) => i.to))
    for (const item of getSettingsOverviewItems()) {
      expect(sidebar.has(item.to)).toBe(true)
    }
  })

  it('uses unique route keys and all routes resolve', () => {
    const seen = new Set<string>()
    for (const item of SETTINGS_NAV_ITEMS) {
      expect(seen.has(item.to)).toBe(false)
      seen.add(item.to)

      const resolved = router.resolve(item.to)
      expect(resolved.matched.length).toBeGreaterThan(0)
    }
  })
})

