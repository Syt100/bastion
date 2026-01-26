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
      nav: [{ text: 'Docs', link: '/' }],
      socialLinks: [{ icon: 'github', link: repoUrl }],

      search: {
        provider: 'local',
      },

      editLink: {
        pattern: `${repoUrl}/edit/main/docs/:path`,
      },

      sidebar: [
        {
          text: 'Product / Web UI',
          items: [
            { text: 'Agents', link: '/agents' },
            { text: 'Jobs', link: '/jobs' },
            { text: 'Backup snapshots', link: '/backup-snapshots' },
            { text: 'Bulk operations', link: '/bulk-operations' },
            { text: 'Storage (WebDAV)', link: '/storage' },
          ],
        },
        {
          text: 'Operations / Deployment',
          items: [
            { text: 'Reverse proxy', link: '/reverse-proxy' },
            { text: 'Logging', link: '/logging' },
            { text: 'Data directory', link: '/data-directory' },
          ],
        },
        {
          text: 'Recipes',
          items: [{ text: 'Vaultwarden', link: '/recipes/vaultwarden' }],
        },
      ],
    },
  }
})

