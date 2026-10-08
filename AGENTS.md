# AstralRinth monorepo — agent instructions

Read [`STRUCTURE.md`](STRUCTURE.md) for repository architecture, package responsibilities, application entry points, fork-specific behavior, and integration contracts before changing the relevant area. When entering a project to edit or analyse it, also read its project-level `AGENTS.md`; these root instructions do not replace project-specific guidance.

## Development workflow

- Use the development commands and environment setup documented in [`STRUCTURE.md`](STRUCTURE.md#development-tooling-and-commands). Prepare the app library's environment from its template before starting app development.

## Code Guidelines

- **Indentation:** Use TAB everywhere, never spaces

### Internationalization

- When reviewing, adding, or updating i18n messages in frontend files, always check the corresponding entries in `apps/app-frontend/src/locales/en-US/index.json` and `apps/app-frontend/src/locales/ru-RU/index.json`. Add missing entries and update stale translations in both locales as part of the same task. Keep message IDs and ICU placeholders consistent with the frontend definitions; do not rely on `defaultMessage` fallback alone. This repository rule takes precedence over skill guidance that says not to edit localization JSON files.

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

Read the applicable standards in `standards/` before editing the relevant area.

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
11. **Review fork-owned automation.** Preserve AstralRinth build automation and issue/support policy. Do not retain upstream workflows/actions merely for parity. Check callers, `uses:` references, repository settings, package/Cargo scripts, and documentation before deleting automation or its assets. Follow `COPYING.md` when replacing upstream branding; replace referenced cover artwork and update its callers before removing the old images.
12. **Record behavior outcomes.** Classify each upstream behavior change as preserved, adapted/migrated, intentionally removed/replaced, or requiring product-owner confirmation. Review the cross-layer integration boundaries documented in `STRUCTURE.md`, not only files with merge conflicts.

## Integration safeguards

- Preserve the shared Minecraft account-method chooser across account cards, required-account dialogs, and the skin page. Do not replace it with a direct Microsoft-only action or confuse Minecraft authentication with Modrinth account sign-in.
- Keep the upstream updater disconnected from the Xorison update flow unless the user explicitly authorizes reactivation.
- Trace new or changed frontend `invoke` calls through Rust command/plugin registration, permissions/capabilities, allowed origins where applicable, and matching argument/return types.
- Preserve fresh-install behavior and upgrades from existing Modrinth/AstralRinth databases when adapting migrations. Check window refocus-on-game-close, tab visibility (including legacy per-instance settings), telemetry, and account/skin preferences against the requested stable-release baseline.
- Verify analytics helper callers before removing or renaming imports; disabled usage analytics does not imply that every reporting integration or helper is unused.
- Verify rendering-helper exports at their current import sources. Keep account-type checks, Ears/custom skin behavior, preview acquisition/release, and URL/resource cleanup connected.
- Keep Liquid Glass limited to background transparency and decoration. Do not override foreground `color`, `fill`, `stroke`, font styles, theme foreground tokens, or whole-element `opacity`/`filter`; use `backdrop-filter` for glass blur. Preserve theme colors, opaque/accessibility fallbacks, native component interaction colors, and the AstralRinth Visual control.
- Contain decorative layers within their controls; do not repair background defects by recoloring text or icons. Check CSS specificity and selector reachability against app markup, dynamic classes/attributes, teleported UI, and browser pseudo-elements before pruning selectors.
- After changing glass overrides or shared component markup, verify mouse, keyboard, scrolling, nested menus, modal transitions, focus indicators, disabled states, hit areas, and fixed-position anchoring in the running app. Non-intercepting pseudo-elements alone do not establish that interactions remain intact.

## Verification

- Run focused checks relevant to the changed code and report the exact commands and results. Do not run broad repository checks automatically for every change.
- Use the frontend type-check command documented in [`STRUCTURE.md`](STRUCTURE.md#development-tooling-and-commands) for Vue/TypeScript validation without a build.
- After changing Vue components or frontend/Rust event contracts, verify imports, exports, and matching types/bindings.
- Keep skin editing gated to Microsoft accounts. With no selected account, preserve the sign-in prompt and all shared authentication methods; do not restore demo editing. Verify no-account, Microsoft, offline, and external-provider states after changing these gates.
- After changing AstralRinth news, verify startup loading, the age-based indicator, and client-side pagination. There must be no manual refresh/retry buttons; opening the news modal must not trigger another request.
- For Rust checks, identify the affected crate in `Cargo.toml` and read its project guidance if present.
- After changing AstralRinth-specific Rust behavior, run the global fork test selection from the repository root: `cargo test -p theseus --lib astralrinth -- --nocapture`. This selects library tests whose names/module paths contain `astralrinth`; it does not cover every fork integration or live authentication/launch flow. Run relevant tests outside that selection when shared Rust code is affected; use `cargo test -p theseus --lib` when broader app-library regression coverage is warranted.
- After changing external-provider library catalogs, verify flat remote assets, nullable latest-injector metadata, missing download URLs without automatic fallback, and continued use of existing local selections (including legacy files).

## Documentation maintenance

- **Keep repository instructions current.** After sensitive or breaking changes to architecture, authentication, updates, persistence, security, commands/events, or fork-specific behavior, update the affected sections of `AGENTS.md` and `STRUCTURE.md` in the same task. Keep AI working rules and validation requirements in `AGENTS.md`; keep architecture, technical behavior, commands, and integration/migration contracts in `STRUCTURE.md`. Keep the root README and `readme/ru_ru/README.md` aligned and user-friendly: installation, features, usage, support, privacy notices, and project information, without implementation details. Link to `STRUCTURE.md` instead of duplicating technical explanations. Make targeted edits rather than rewriting unrelated sections.

<!-- BEGIN:turborepo-agent-rules -->

# This is NOT the Turborepo you know

Turborepo configuration, task behavior, and CLI commands can vary between installed versions and may differ from your training data. Resolve the `turbo` package from this file's directory or relevant workspace; in monorepos, it may not be visible from the repository root. For example, run `node -p "require.resolve('turbo/package.json')"` from a workspace that depends on `turbo`.

Read `docs/README.md` inside that installed package first, then read the relevant pages from its `docs/` directory before changing Turborepo configuration or commands. Heed deprecation notices. These bundled docs match the installed package version and are available without network access.

This block is written and re-added by `turbo` before repository-scoped commands when an AI agent is detected. In the Turborepo source repository, its template is defined in `crates/turborepo-cli/src/cli/agent_guidance.rs`. Removing the managed block while updates are enabled means a later qualifying invocation will add it again. Set `"agentGuidance": false` in the root `turbo.json` or `turbo.jsonc` to opt out; this does not remove an existing block. Keep the block committed with your work to avoid an uncommitted change on the next agent invocation.
<!-- END:turborepo-agent-rules -->
