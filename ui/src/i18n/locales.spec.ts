import { describe, expect, it } from 'vitest'

import enUS from '@/i18n/locales/en-US'
import zhCN from '@/i18n/locales/zh-CN'

function flattenKeys(value: unknown, prefix = ''): string[] {
  if (!value || typeof value !== 'object') return prefix ? [prefix] : []
  if (Array.isArray(value)) return prefix ? [prefix] : []

  const out: string[] = []
  for (const [key, child] of Object.entries(value)) {
    const next = prefix ? `${prefix}.${key}` : key
    out.push(...flattenKeys(child, next))
  }
  return out
}

describe('i18n locales', () => {
  it('keeps zh-CN and en-US keys in sync', () => {
    const zhKeys = new Set(flattenKeys(zhCN).sort())
    const enKeys = new Set(flattenKeys(enUS).sort())

    const missingInEn = [...zhKeys].filter((k) => !enKeys.has(k)).sort()
    const missingInZh = [...enKeys].filter((k) => !zhKeys.has(k)).sort()

    if (missingInEn.length || missingInZh.length) {
      throw new Error(
        [
          missingInEn.length ? `Missing in en-US: ${missingInEn.join(', ')}` : null,
          missingInZh.length ? `Missing in zh-CN: ${missingInZh.join(', ')}` : null,
        ]
          .filter(Boolean)
          .join('\n'),
      )
    }

    expect(missingInEn).toEqual([])
    expect(missingInZh).toEqual([])
  })
})

