import { fileURLToPath, URL } from 'node:url'
import fs from 'node:fs'
import path from 'node:path'

import type { Plugin } from 'vite'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'
import { build } from 'esbuild'

import { DEFAULT_UI_THEME_ID, UI_THEME_META_COLORS } from './src/theme/presets'

function readWorkspaceVersion(): string | null {
  try {
    const cargoTomlPath = fileURLToPath(new URL('../Cargo.toml', import.meta.url))
    const cargoToml = fs.readFileSync(cargoTomlPath, 'utf8')
    const sectionIndex = cargoToml.indexOf('[workspace.package]')
    if (sectionIndex < 0) return null
    const section = cargoToml.slice(sectionIndex)
    const match = section.match(/^\s*version\s*=\s*"(.*?)"\s*$/m)
    return match?.[1] ?? null
  } catch {
    return null
  }
}

const workspaceVersion = readWorkspaceVersion()
const uiVersion = (process.env.BASTION_VERSION?.trim() || workspaceVersion || 'dev').trim()

const uiBuildTimeUnixEnv = process.env.BASTION_UI_BUILD_TIME_UNIX || process.env.BASTION_BUILD_TIME_UNIX || process.env.SOURCE_DATE_EPOCH
const uiBuildTimeUnixParsed = uiBuildTimeUnixEnv ? Number.parseInt(uiBuildTimeUnixEnv, 10) : Number.NaN
const uiBuildTimeUnix = Number.isFinite(uiBuildTimeUnixParsed) ? uiBuildTimeUnixParsed : Math.floor(Date.now() / 1000)

const uiGitSha = (process.env.GITHUB_SHA?.trim() || null)
const defaultThemeColor = UI_THEME_META_COLORS[DEFAULT_UI_THEME_ID].light

function themeBootstrapPlugin(): Plugin {
  const entryPath = fileURLToPath(new URL('./src/theme/bootstrap.ts', import.meta.url))
  let cached: string | null = null

  async function buildInlineScript(): Promise<string> {
    if (cached) return cached
    const result = await build({
      entryPoints: [entryPath],
      bundle: true,
      write: false,
      platform: 'browser',
      format: 'iife',
      target: ['es2019'],
      minify: true,
      sourcemap: false,
    })
    const out = result.outputFiles?.[0]?.text ?? ''
    cached = out.trim()
    return cached
  }

  return {
    name: 'bastion-theme-bootstrap',
    enforce: 'pre',
    handleHotUpdate(ctx) {
      // Invalidate the inlined script when theme sources change.
      if (ctx.file.includes(`${path.sep}src${path.sep}theme${path.sep}`)) {
        cached = null
        ctx.server.ws.send({ type: 'full-reload' })
        return []
      }
    },
    async transformIndexHtml(html) {
      const script = await buildInlineScript()
      if (!script) return html
      return {
        html,
        tags: [
          {
            tag: 'meta',
            injectTo: 'head-prepend',
            attrs: {
              name: 'theme-color',
              content: defaultThemeColor,
            },
          },
          {
            tag: 'script',
            injectTo: 'head-prepend',
            children: script,
          },
        ],
      }
    },
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    themeBootstrapPlugin(),
    vue(),
    vueDevTools(),
  ],
  define: {
    __BASTION_UI_VERSION__: JSON.stringify(uiVersion),
    __BASTION_UI_BUILD_TIME_UNIX__: JSON.stringify(uiBuildTimeUnix),
    __BASTION_UI_GIT_SHA__: JSON.stringify(uiGitSha),
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:9876',
        changeOrigin: false,
        ws: true,
      },
      '/agent': {
        target: 'http://127.0.0.1:9876',
        changeOrigin: true,
        ws: true,
      },
      // Let "Help" (/docs) work when the UI is served from the Vite dev server.
      '/docs': {
        target: 'http://127.0.0.1:9876',
        changeOrigin: false,
      },
    },
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
})
