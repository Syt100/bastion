import { defineConfig } from 'vitepress'

function buildBase(): string {
  // GitHub Pages project site:
  // - root: https://<owner>.github.io/<repo>/
  // - docs: https://<owner>.github.io/<repo>/docs/
  //
  // Allow overriding for forks and local builds.
  const envBase = process.env.DOCS_BASE?.trim()
  if (envBase) return envBase.endsWith('/') ? envBase : `${envBase}/`

  const repo = process.env.GITHUB_REPOSITORY?.split('/')[1]?.trim()
  if (repo) return `/${repo}/docs/`

  return '/docs/'
}

const repoSlug = process.env.DOCS_REPO || process.env.GITHUB_REPOSITORY || 'Syt100/bastion'
const repoUrl = `https://github.com/${repoSlug}`

export default defineConfig(({ command }) => {
  function withLocale(localePrefix: '' | '/zh', path: string): string {
    if (!path.startsWith('/')) return `${localePrefix}/${path}`
    return `${localePrefix}${path}`
  }

  return {
    // Keep dev server at /, but build output under /<repo>/docs/ for GitHub Pages.
    base: command === 'serve' ? '/' : buildBase(),

    locales: {
      root: {
        label: 'English',
        lang: 'en-US',
        title: 'Bastion',
        description: 'Self-hosted backup orchestrator (Hub + optional Agents) with a Web UI.',
        themeConfig: {
          nav: [
            { text: 'Home', link: '/' },
            { text: 'User Manual', link: '/user/' },
            { text: 'Developer Docs', link: '/dev/' },
          ],
          socialLinks: [{ icon: 'github', link: repoUrl }],

          search: {
            provider: 'local',
          },

          editLink: {
            pattern: `${repoUrl}/edit/main/docs/:path`,
          },

          sidebar: {
            '/user/': [
              {
                text: 'Getting started',
                items: [
                  { text: 'Overview', link: '/user/' },
                  { text: 'Concepts', link: '/user/concepts' },
                  { text: 'Quickstart', link: '/user/getting-started' },
                ],
              },
              {
                text: 'Using Bastion',
                items: [
                  { text: 'Agents', link: '/user/agents' },
                  { text: 'Jobs', link: '/user/jobs' },
                  { text: 'Runs', link: '/user/runs' },
                  { text: 'Restore and verify', link: '/user/restore-verify' },
                  { text: 'Backup snapshots', link: '/user/backup-snapshots' },
                  { text: 'Bulk operations', link: '/user/bulk-operations' },
                  { text: 'Storage (WebDAV)', link: '/user/storage' },
                ],
              },
              {
                text: 'Operations',
                items: [
                  { text: 'Runtime config', link: '/user/operations/runtime-config' },
                  { text: 'Notifications', link: '/user/operations/notifications' },
                  { text: 'Logging', link: '/user/operations/logging' },
                  { text: 'Maintenance', link: '/user/operations/maintenance' },
                  { text: 'Data directory', link: '/user/operations/data-directory' },
                  { text: 'Reverse proxy', link: '/user/operations/reverse-proxy' },
                ],
              },
              {
                text: 'Recipes',
                items: [{ text: 'Vaultwarden', link: '/user/recipes/vaultwarden' }],
              },
            ],
            '/dev/': [
              {
                text: 'Development',
                items: [
                  { text: 'Overview', link: '/dev/' },
                  { text: 'Build and run', link: '/dev/build' },
                  { text: 'Docs site', link: '/dev/docs-site' },
                  { text: 'Architecture', link: '/dev/architecture' },
                ],
              },
              {
                text: 'Design notes',
                items: [{ text: 'Backup snapshots', link: '/dev/design/backup-snapshots' }],
              },
            ],
          },
        },
      },
      zh: {
        label: '简体中文',
        lang: 'zh-CN',
        title: 'Bastion',
        description: '自托管备份编排器（Hub + 可选 Agents），带 Web UI。',
        themeConfig: {
          nav: [
            { text: '首页', link: withLocale('/zh', '/') },
            { text: '用户手册', link: withLocale('/zh', '/user/') },
            { text: '开发文档', link: withLocale('/zh', '/dev/') },
          ],
          socialLinks: [{ icon: 'github', link: repoUrl }],

          search: {
            provider: 'local',
          },

          editLink: {
            pattern: `${repoUrl}/edit/main/docs/:path`,
          },

          sidebar: {
            '/zh/user/': [
              {
                text: '入门',
                items: [
                  { text: '概览', link: '/zh/user/' },
                  { text: '概念与术语', link: '/zh/user/concepts' },
                  { text: '快速开始', link: '/zh/user/getting-started' },
                ],
              },
              {
                text: '使用 Bastion',
                items: [
                  { text: 'Agents（代理）', link: '/zh/user/agents' },
                  { text: 'Jobs（作业）', link: '/zh/user/jobs' },
                  { text: 'Runs（运行记录）', link: '/zh/user/runs' },
                  { text: '恢复与校验', link: '/zh/user/restore-verify' },
                  { text: '备份快照', link: '/zh/user/backup-snapshots' },
                  { text: '批量操作', link: '/zh/user/bulk-operations' },
                  { text: '存储（WebDAV）', link: '/zh/user/storage' },
                ],
              },
              {
                text: '运维',
                items: [
                  { text: '运行时配置', link: '/zh/user/operations/runtime-config' },
                  { text: '通知', link: '/zh/user/operations/notifications' },
                  { text: '日志', link: '/zh/user/operations/logging' },
                  { text: '维护', link: '/zh/user/operations/maintenance' },
                  { text: '数据目录', link: '/zh/user/operations/data-directory' },
                  { text: '反向代理', link: '/zh/user/operations/reverse-proxy' },
                ],
              },
              {
                text: '配方',
                items: [{ text: 'Vaultwarden', link: '/zh/user/recipes/vaultwarden' }],
              },
            ],
            '/zh/dev/': [
              {
                text: '开发',
                items: [
                  { text: '概览', link: '/zh/dev/' },
                  { text: '构建与运行', link: '/zh/dev/build' },
                  { text: '文档站', link: '/zh/dev/docs-site' },
                  { text: '架构', link: '/zh/dev/architecture' },
                ],
              },
              {
                text: '设计说明',
                items: [{ text: '备份快照', link: '/zh/dev/design/backup-snapshots' }],
              },
            ],
          },
        },
      },
    },
  }
})
