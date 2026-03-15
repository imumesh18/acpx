# PLAN

This file tracks forward-looking work only.

## Current Priorities

- Revisit the transport boundary so callers can create a `Connection` from
  existing ACP-compatible streams, not only from subprocess launch helpers.
- Decide whether binary registry distributions should stay consumer-managed or
  gain an explicit install and cache story in `acpx`.
- Keep public examples and crate docs compile-checked as the API evolves.
- Continue tightening the public contract around the upstream ACP SDK's local
  `!Send` execution model without hiding it behind runtime-specific types.

## Open Design Work

### Transport Boundary

- Separate subprocess launch convenience from ACP connection wiring.
- Preserve the current `Connection` lifecycle guarantees when the transport is
  not owned by `async-process`.
- Keep the public API runtime neutral and explicit about local task spawning.

### Registry-Backed Launches

- Decide whether `acpx` should remain limited to already-invocable package
  managers such as `npx` and `uvx`.
- If binary installation is added later, define a small, testable contract for
  download, extraction, host-target resolution, and cleanup.
- Keep the committed registry snapshot as the offline source of truth for
  builds and tests.

### Client-Side ACP Hooks

- Evaluate whether the default client should stay minimal with only
  `session/update` capture and `method_not_found` for unsupported callbacks.
- If richer client behavior is added, expose it without obscuring the upstream
  ACP protocol surface.

## Decision Rules

- Stay close to upstream ACP names and request and response types.
- Keep the local `!Send` runtime model explicit.
- Prefer typed errors and deterministic offline tests.
- Write behavior into `SPEC.md` before implementing new ACP surface area.
- Avoid adding installer, persistence, or conversation policy unless the crate
  clearly reduces repeated boilerplate without hiding important ACP details.

## Not Planned

- Replacing ACP with a higher-level chat or conversation abstraction.
- Network-dependent tests or build-time registry fetches.
- Hidden retry, reconnection, or persistence behavior.
