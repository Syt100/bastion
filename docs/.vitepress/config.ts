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
  return {
    lang: 'en-US',
    title: 'Bastion',
    description: 'Self-hosted backup orchestrator (Hub + optional Agents) with a Web UI.',

    // Keep dev server at /, but build output under /<repo>/docs/ for GitHub Pages.
    base: command === 'serve' ? '/' : buildBase(),

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
              { text: 'Quickstart', link: '/user/getting-started' },
            ],
          },
          {
            text: 'Using Bastion',
            items: [
              { text: 'Agents', link: '/user/agents' },
              { text: 'Jobs', link: '/user/jobs' },
              { text: 'Backup snapshots', link: '/user/backup-snapshots' },
              { text: 'Bulk operations', link: '/user/bulk-operations' },
              { text: 'Storage (WebDAV)', link: '/user/storage' },
            ],
          },
          {
            text: 'Operations',
            items: [
              { text: 'Reverse proxy', link: '/user/operations/reverse-proxy' },
              { text: 'Logging', link: '/user/operations/logging' },
              { text: 'Data directory', link: '/user/operations/data-directory' },
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
  }
})
