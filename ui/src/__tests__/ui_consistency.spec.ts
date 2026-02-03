import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

function resolveUiSrcDir(): string {
  const candidates: string[] = []

  // Preferred: relative to this test file.
  try {
    const filename = fileURLToPath(import.meta.url)
    candidates.push(path.resolve(path.dirname(filename), '..'))
  } catch {
    // Fall back to process.cwd()-based paths below.
  }

  // Support running from either `ui/` or repo root.
  candidates.push(path.resolve(process.cwd(), 'src'))
  candidates.push(path.resolve(process.cwd(), 'ui/src'))

  for (const candidate of candidates) {
    if (existsSync(candidate)) return candidate
  }

  throw new Error(`Unable to locate ui/src directory. Tried: ${candidates.join(', ')}`)
}

function listVueFiles(dir: string): string[] {
  const out: string[] = []
  const entries = readdirSync(dir, { withFileTypes: true })
  for (const entry of entries) {
    const p = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      out.push(...listVueFiles(p))
      continue
    }
    if (entry.isFile() && entry.name.endsWith('.vue')) {
      out.push(p)
    }
  }
  return out
}

type Violation = {
  file: string
  line: number
  tag: string
}

function findAppCardViolations(contents: string, file: string): Violation[] {
  const violations: Violation[] = []

  // Match opening tags, including multi-line ones.
  const tagRe = /<n-card\b[^>]*>/gs
  for (const match of contents.matchAll(tagRe)) {
    const tag = match[0]
    if (!/\bclass\s*=/.test(tag)) continue
    if (!/\bapp-card\b/.test(tag)) continue

    // Vue boolean props must be bound to avoid passing a truthy string.
    const hasBorderedFalse = /:bordered\s*=\s*(['"])false\1/.test(tag)
    if (hasBorderedFalse) continue

    const idx = match.index ?? 0
    const line = contents.slice(0, idx).split('\n').length
    violations.push({ file, line, tag })
  }

  return violations
}

describe('UI consistency', () => {
  it('requires app-card n-cards to be borderless', () => {
    const uiSrc = resolveUiSrcDir()
    const files = listVueFiles(uiSrc)

    const violations: Violation[] = []
    for (const file of files) {
      const contents = readFileSync(file, 'utf-8')
      violations.push(...findAppCardViolations(contents, file))
    }

    expect(violations).toEqual([])
  })
})

