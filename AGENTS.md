<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## Security: Sensitive Environment Variables

The environment variable `GITHUB_PAT_TOKEN` MUST be treated as a secret.

- Do NOT print, log, echo, or otherwise reveal the value of `GITHUB_PAT_TOKEN` in any form.
- Do NOT run commands that may output environment variables (for example: `env`, `printenv`, `set`, `export -p`) if they could disclose `GITHUB_PAT_TOKEN`.
- Only allowed interaction: check whether `GITHUB_PAT_TOKEN` is set (existence) without displaying its value.
  - Example (safe): `test -n "${GITHUB_PAT_TOKEN+x}"` (checks existence only; prints nothing)
