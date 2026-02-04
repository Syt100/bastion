import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { UI_THEME_IDS } from '@/theme/presets'

function readMainCss(): string {
  const candidates: string[] = []

  // Preferred: relative to this test file (works in most runners).
  try {
    const filename = fileURLToPath(import.meta.url)
    candidates.push(path.resolve(path.dirname(filename), '../styles/main.css'))
  } catch {
    // Some runners may not provide a `file:` import.meta.url. We'll fall back below.
  }

  // Fallbacks: support running from either `ui/` or repo root.
  candidates.push(path.resolve(process.cwd(), 'src/styles/main.css'))
  candidates.push(path.resolve(process.cwd(), 'ui/src/styles/main.css'))

  for (const p of candidates) {
    if (!existsSync(p)) continue
    return readFileSync(p, 'utf-8')
  }

  throw new Error(`Unable to load main.css from: ${candidates.join(', ')}`)
}

const css = readMainCss()

// Regression test:
// `background: var(--app-bg)` can't safely include a raw color token as a comma-separated "layer".
// If it does, the solid base becomes transparent and dark mode can show a white canvas.
describe('main.css background variables', () => {
  it('keeps --app-bg as images-only and sets the solid base on html', () => {
    const bgDefs = [...css.matchAll(/--app-bg:\s*[^;]*;/gs)].map((m) => m[0])
    expect(bgDefs.length).toBeGreaterThanOrEqual(2)
    for (const def of bgDefs) {
      expect(def).not.toContain('app-bg-solid')
    }

    expect(css).toMatch(/html\s*\{[^}]*background-color:\s*var\(--app-bg-solid\)\s*;?[^}]*\}/s)
  })

  it('defines both light and dark theme token blocks for every theme', () => {
    for (const themeId of UI_THEME_IDS) {
      const id = String(themeId).replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      // Light blocks start at the beginning of a line (no ".dark" prefix).
      expect(css).toMatch(new RegExp(`(?:^|\\s)\\[data-theme=['"]${id}['"]\\]\\s*\\{`))
      expect(css).toMatch(new RegExp(`\\.dark\\[data-theme=['"]${id}['"]\\]\\s*\\{`))
    }
  })

  it('defines background style overrides (solid/plain) with a neutral base token', () => {
    expect(css).toMatch(/:root\s*\{[^}]*--app-bg-neutral:\s*#[0-9a-fA-F]{6}\s*;?[^}]*\}/s)
    expect(css).toMatch(/:root\s*\{[^}]*--app-surface-neutral:\s*#[0-9a-fA-F]{6}\s*;?[^}]*\}/s)
    expect(css).toMatch(/:root\s*\{[^}]*--app-surface-2-neutral:\s*#[0-9a-fA-F]{6}\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\s*\{[^}]*--app-bg-neutral:\s*#[0-9a-fA-F]{6}\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\s*\{[^}]*--app-surface-neutral:\s*#[0-9a-fA-F]{6}\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\s*\{[^}]*--app-surface-2-neutral:\s*#[0-9a-fA-F]{6}\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\s*\{[^}]*--app-glass-bg-neutral:\s*rgba\(/s)
    expect(css).toMatch(/\.dark\s*\{[^}]*--app-glass-soft-bg-neutral:\s*rgba\(/s)

    expect(css).toMatch(/\[data-bg=['"]solid['"]\]\s*\{[^}]*--app-bg:\s*none\s*;?[^}]*\}/s)
    expect(css).toMatch(/\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-bg:\s*none\s*;?[^}]*\}/s)
    expect(css).toMatch(/\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-bg-solid:\s*var\(--app-bg-neutral\)\s*;?[^}]*\}/s)
    expect(css).toMatch(/\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-surface:\s*var\(--app-surface-neutral\)\s*;?[^}]*\}/s)
    expect(css).toMatch(/\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-surface-2:\s*var\(--app-surface-2-neutral\)\s*;?[^}]*\}/s)

    // Dark theme token blocks use `.dark[data-theme='...']` (higher specificity),
    // so background style overrides must also exist with `.dark[...]` selectors.
    expect(css).toMatch(/\.dark\[data-bg=['"]solid['"]\]\s*\{[^}]*--app-bg:\s*none\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-bg:\s*none\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-bg-solid:\s*var\(--app-bg-neutral\)\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-surface:\s*var\(--app-surface-neutral\)\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-surface-2:\s*var\(--app-surface-2-neutral\)\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-glass-bg:\s*var\(--app-glass-bg-neutral\)\s*;?[^}]*\}/s)
    expect(css).toMatch(/\.dark\[data-bg=['"]plain['"]\]\s*\{[^}]*--app-glass-soft-bg:\s*var\(--app-glass-soft-bg-neutral\)\s*;?[^}]*\}/s)
  })
})
