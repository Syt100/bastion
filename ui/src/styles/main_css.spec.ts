import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

function readMainCss(): string {
  const candidates: string[] = []

  // Preferred: adjacent to this spec file.
  try {
    const filename = fileURLToPath(import.meta.url)
    candidates.push(path.resolve(path.dirname(filename), 'main.css'))
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
})
