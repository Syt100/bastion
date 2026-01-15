# Git hooks

This repo includes optional Git hooks under `.githooks/`.

To enable them locally:

```bash
git config core.hooksPath .githooks
```

## commit-msg

The `commit-msg` hook prevents a common mistake:

- Writing `git commit -m "subject\n\n- bullet"` and expecting `\n` to become real newlines.
- In most shells, `\n` inside quotes is NOT expanded, so the commit message ends up containing literal `\n`.

The hook blocks suspicious patterns like:

- `\n\n` (attempted blank line)
- `\n-` / `\n*` / `\n#` (attempted bullet/heading)

### Legitimate `\n` literals

If you need to mention `\n` in a commit message (e.g. code like `printf("\n")`), prefer wrapping it in backticks:

- `Fix the `printf("\n")` bug`

If you truly need to include the blocked patterns outside code spans, add this trailer line:

```
Allow-Literal-Backslash-N: yes
```

