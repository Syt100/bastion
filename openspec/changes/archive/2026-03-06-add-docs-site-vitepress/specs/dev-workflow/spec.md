## ADDED Requirements

### Requirement: Documentation Site Is Published To GitHub Pages
The project SHALL publish a documentation site built from the repository `docs/` content to GitHub Pages at the entry path `/<repo>/docs/`.

#### Scenario: GitHub Pages deploys docs site under /docs/
- **WHEN** a commit is pushed to `main`
- **THEN** GitHub Actions builds the docs site
- **AND** deploys it to GitHub Pages with the entry path `/<repo>/docs/`

