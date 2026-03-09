## Decisions

### Decision: Field chips are capped at 2 per row
- Rows must remain compact and scannable.
- Show at most 2 chips per event, chosen by priority from `event.fields` (object-only).

### Decision: Relative time is used for retry scheduling fields
- `next_attempt_at` is displayed as a relative value (e.g., `2m后` / `3m前`), updated periodically while the modal is open.

### Decision: Follow/tail behavior matches common log viewers
- “Follow” is enabled by default and scrolls to the latest event.
- If the user scrolls away from the bottom, “Follow” automatically turns off to avoid fighting the user.
- When follow is off, new events increment a counter; clicking “Latest” scrolls to bottom and re-enables follow.

### Decision: WS reconnection is automatic by default
- Auto reconnect uses exponential backoff with an upper bound.
- Manual reconnect is always available.
- Connection state is exposed in the UI with a short countdown until the next reconnect attempt.

### Decision: Mobile details use a bottom drawer (half-screen)
- Desktop continues using a modal detail view.
- Mobile uses a bottom drawer (~70vh) to keep context while reading long messages/JSON.

## Chip Selection Heuristics (priority)
Try the following keys in order and render the first 2 found:
1. `error_kind` / `last_error_kind`
2. `attempt` / `attempts` (render as `#N`)
3. `next_attempt_at` (render relative)
4. `duration_ms` (render compact)
5. `errors_total` / `warnings_total` (render as `E#/W#`)
6. `channel` / `secret_name`
7. `agent_id` (short id) / `source` / `executed_offline`

Keys with large values (arrays, long strings) are never shown inline; they remain available in details.

