## ADDED Requirements

### Requirement: Hub Serves Product Documentation Under /docs
The Hub SHALL serve a static product documentation site under `/docs/` without requiring authentication by default.

#### Scenario: /docs redirects and docs pages render
- **WHEN** a client requests `GET /docs`
- **THEN** the Hub redirects to `/docs/`
- **AND WHEN** a client requests `GET /docs/`
- **THEN** the Hub serves the documentation index page

### Requirement: Docs Can Be Served From FS Or Embedded
The Hub SHALL support serving docs either from the filesystem (via `BASTION_DOCS_DIR`) or from embedded assets when built with an `embed-docs` feature.

#### Scenario: Filesystem docs dir override is honored
- **GIVEN** `BASTION_DOCS_DIR` points to a directory containing `index.html`
- **WHEN** a client requests `GET /docs/`
- **THEN** the Hub serves `index.html` from that directory

