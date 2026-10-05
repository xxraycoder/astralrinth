# AstralRinth monorepo — agent instructions

Read [`STRUCTURE.md`](STRUCTURE.md) for repository architecture, package responsibilities, application entry points, fork-specific behavior, and integration contracts before changing the relevant area. When entering a project to edit or analyse it, also read its project-level `AGENTS.md`; these root instructions do not replace project-specific guidance.

## Dev Commands

- **App:** `pnpm app:dev --cache=local:r` (read-only local cache; copy `.env` template in `packages/app-lib/` first)

## Code Guidelines

- **Indentation:** Use TAB everywhere, never spaces

### Comments

- DO NOT use "heading" comments like: `=== Helper methods ===`.
- Use doc comments, but avoid inline comments unless ABSOLUTELY necessary for clarity. Code should aim to be self documenting!

## Bash Guidelines

### Output handling

- DO NOT pipe output through `head`, `tail`, `less`, or `more`
- NEVER use `| head -n X` or `| tail -n X` to truncate output
- IMPORTANT: Run commands directly without pipes when possible
- IMPORTANT: If you need to limit output, use command-specific flags (e.g. `git log -n 10` instead of `git log | head -10`)
- ALWAYS read the full output — never pipe through filters

### General

- Do not create new non-source code files (e.g. Bash scripts, SQL scripts) unless explicitly prompted to
- Types in `@modrinth/utils` are considered highly outdated, if a component needs them, check if you can switch said component to use types from `packages/api-client`
- When provided problems, do not say "I didn't introduce these problems" (shifting the blame/effort) - just fix them.

## Standards

Standards available at the @standards/ folder.

## Rules for upstream merges and AI agents

1. **Establish the exact baseline first.** Check `git status`, unresolved paths, current branch/commit, remotes, and tags. For release comparison, use the requested AstralRinth release tag (for example `AR-0.19.202`), not only `HEAD`; this checkout can have merge results in the index/worktree while `HEAD` still points at the previous release.
2. **Inspect both sides of every conflict.** Read the ours and theirs versions and relevant callers. A conflict-free-looking working file may still be unmerged in Git's index. Verify with `git diff --name-only --diff-filter=U`; do not stage files unless explicitly authorized.
3. **Protect fork behavior additively.** Do not resolve a conflict by choosing the entire upstream file if that drops AstralRinth branches, imports, UI, events, settings, or startup calls. Conversely, do not preserve obsolete APIs if upstream replaced them; reconnect the fork behavior to the new API and validate the full path.
4. **Trace integrations.** For every fork-specific behavior, confirm its trigger, UI entry point, Tauri command, permission, Rust implementation, state/settings, event listeners, and cleanup as relevant. Search both direct references and dynamically invoked command/event names.
5. **Treat behavior changes as product decisions.** Compare defaults and migrations against the stable AstralRinth release. Changes such as replacing per-instance navigation settings with global ones or changing whether the launcher refocuses after a game exits are not formatting-only merge resolutions. Document the changed behavior and ask before removing or silently changing a fork capability.
6. **Do not remove user-facing behavior speculatively.** In particular, preserve Xorison update notifications, the AstralRinth news modal, the new-icon-editor notification, external/offline account flows, Ears/custom skin functionality, and fork settings unless the user explicitly asks to remove them.
7. **Respect explicit removals.** Ads and consent UI/native integration were explicitly requested to be removed in this working tree. Do not restore them from upstream during a merge without asking. `PromotionWrapper.vue` was also explicitly requested to be removed; search for callers before restoring or deleting related wrappers.
8. **Validate after resolution.** Search for conflict markers and missing imports/exports, run focused diagnostics/type checks, and run the app's relevant tests where practical. Report exact checks and failures; do not claim the merge is complete while unresolved Git paths remain.
9. **Keep changes scoped and safe.** Preserve unrelated staged and unstaged edits. Do not commit, create branches, or stage changes without explicit permission.
10. **Reapply the repository-pruning policy after every upstream sync.** The omitted website, Labrinth API, docs, playground, Daedalus client, and backend `docker-compose.yml` are intentionally absent from AstralRinth. Upstream updates may modify deleted files (causing modify/delete conflicts) or add new files under these projects. Resolve those paths in favor of deletion, remove any newly reintroduced files, and recheck `Cargo.toml`, `package.json`, workspace/lockfiles, scripts, and CI configuration for references to the excluded projects. Do not restore them merely to make an upstream merge conflict-free.

## Verification commands

- Run focused checks relevant to the changed code and report the exact commands and results. Do not run broad repository checks automatically for every change.
- Run `pnpm --filter @modrinth/app-frontend tsc:check` from the repository root, or `pnpm tsc:check` from `apps/app-frontend/`, for Vue/TypeScript checking without a build. This uses the existing `vue-tsc --noEmit` script in `apps/app-frontend/package.json`.
- After changing Vue components or frontend/Rust event contracts, verify imports, exports, and matching types/bindings.
- After changing AstralRinth news, verify startup loading, the age-based indicator, and client-side pagination. There must be no manual refresh/retry buttons; opening the news modal must not trigger another request.
- For Rust checks, identify the affected crate in `Cargo.toml` and read its project guidance if present.

## Documentation maintenance

- **Keep repository instructions current.** After sensitive or breaking changes to architecture, authentication, updates, persistence, security, commands/events, or fork-specific behavior, update the affected sections of `AGENTS.md` and `STRUCTURE.md` in the same task. Keep working rules in `AGENTS.md` and structural information in `STRUCTURE.md`. Document the new contracts, migration requirements, and validation steps. Make targeted edits rather than rewriting unrelated sections.
