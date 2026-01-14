import { describe, expect, it } from 'vitest'

import { supportedLocales } from '@/i18n'
import { getLocaleDropdownOptions, LOCALE_LABELS, NAIVE_UI_DATE_LOCALES, NAIVE_UI_LOCALES } from './language'

describe('i18n language helpers', () => {
  it('keeps dropdown options aligned with supportedLocales', () => {
    const options = getLocaleDropdownOptions()
    expect(options.map((o) => o.key)).toEqual(supportedLocales)
    for (const opt of options) {
      expect(opt.label.trim().length).toBeGreaterThan(0)
    }
  })

  it('maps every supported locale to labels and Naive UI locales', () => {
    for (const locale of supportedLocales) {
      expect(LOCALE_LABELS[locale].trim().length).toBeGreaterThan(0)
      expect(NAIVE_UI_LOCALES[locale]).toBeTruthy()
      expect(NAIVE_UI_DATE_LOCALES[locale]).toBeTruthy()
    }
  })
})

