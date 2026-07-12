# Decision 0002: preflight the complete USB initialization plan

- Date: 2026-07-11
- Status: accepted
- Audited upstream revision: SDL `82141a2439acc111661102ba6f968c85e71cff40`

## Context

SDL's BEE-021 USB sequence contains four packets with descriptive upstream
labels and six steps that are unknown, rumble-related, or specific to a
charging grip. Descriptive labels do not establish whether a setting is truly
volatile or persistent.

## Decision

Model the complete upstream ordering, but expose packet encodings only for the
four understood input-session candidates:

| Modeled operation | Current classification | Execution status |
| --- | --- | --- |
| Set feature-output mask | candidate volatile | blocked |
| Enable feature-output channels | candidate volatile | blocked |
| Select input report format `0x05` | candidate volatile | blocked |
| Start report streaming | candidate volatile | blocked |

Unknown-purpose, rumble-related, and non-BEE-021 grip steps remain explicit
blockers. The executor preflights the whole plan before any transfer; one
blocker or candidate classification prevents all I/O. Only commands promoted
to `VolatileSession` after independent review can be serialized by an
executable plan.

The production module provides no live USB transport and no public constructor
for an executable plan. Mock-only tests exercise ordering, bounded replies,
failure shutdown, and the no-partial-initialization rule.

## Consequences

The project can reason about command order and failure behavior without
touching hardware. Wired input remains nonfunctional. A later review must
either establish the missing steps' semantics or demonstrate that BEE-021 input
can start without them. Live execution still requires a separate implementation
and immediate user approval.
