import { fileURLToPath, URL } from 'node:url'
import fs from 'node:fs'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

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

// https://vite.dev/config/
export default defineConfig({
  plugins: [
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
    },
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
})
