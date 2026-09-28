# AstralRinth repository structure and upstream integration guide

## Purpose

This document describes the Modrinth monorepo as maintained by the AstralRinth fork, identifies where fork-specific behavior lives, and sets expectations for future upstream merges and AI-assisted changes. It supplements the root [`AGENTS.md`](AGENTS.md); it does not replace project-level instructions.

AstralRinth is a fork of Modrinth's codebase, not a separate launcher layered on top. Its desktop launcher shares the upstream app shell, frontend, and Rust app library. A change to an apparently generic Modrinth component, setting, API command, or database model can therefore break a fork-specific flow even when no file named `astralrinth` is edited.

## Monorepo map

The repository uses pnpm workspaces and Turborepo for JavaScript/TypeScript packages and a Cargo workspace for Rust.

| Path | Responsibility | Fork integration notes |
| --- | --- | --- |
| `apps/app-frontend/` | Vue 3 + Vite frontend for the Tauri desktop launcher | Main product surface for account flows, launcher UI, skins, settings, and Xorison release updates. |
| `apps/app/` | Tauri/Rust desktop host | Registers native plugins, commands, permissions, capabilities, window behavior, and desktop startup. |
| `packages/app-lib/` | Shared Rust launcher/backend library used by the desktop host | Owns settings persistence/migrations, Minecraft authentication and launching, process lifecycle, app events, and AstralRinth backend modules. |
| `packages/assets/` | Shared icons, styles, and design assets | Contains AstralRinth logos, provider icons, and fork-specific visual styles. |
| `packages/ui/` | Shared Modrinth Vue component library | App frontend composes these components; check API changes when replacing wrappers or migrating UI. |
| `packages/api-client/` | Shared API client | Prefer its current API types/contracts over deprecated utility-package types when touching shared frontend code. |
| `apps/frontend/` | Modrinth website (Nuxt) | A separate web product. Desktop-specific AstralRinth behavior should not be added here unless it is explicitly a shared feature. |
| `apps/labrinth/` | Modrinth API backend | Separate Rust service. Read `apps/labrinth/AGENTS.md` before working in it. |
| `apps/docs/`, `apps/daedalus_client/`, `apps/app-playground/` | Documentation, Daedalus client, and app playground | Supporting projects; see their package scripts and local conventions before changing them. |
| `packages/` (other) | Shared utilities, config, protocol, assets, analytics, and related libraries | Changes can affect both desktop and web consumers. |
| `scripts/`, `.github/`, `standards/` | Repository automation, fork-owned CI, and engineering standards | Keep only automation and policies that AstralRinth actively uses; upstream Modrinth workflows are not required just because this is a fork. |

## GitHub repository metadata (`.github/`)

Treat `.github/` as configuration for this repository, not as a place to preserve every file from upstream Modrinth. Keep files that serve an active AstralRinth workflow, GitHub feature, or repository instruction; remove upstream automation that is not part of AstralRinth's CI/CD or release process.

### Keep while they are in use

- `.github/workflows/astralrinth-build.yml`: AstralRinth's desktop build pipeline and its platform artifacts/checksums. Keep and evolve this workflow as the fork's own CI/CD contract.
- `.github/ISSUE_TEMPLATE/astralrinth-bug.yml`: AstralRinth-specific issue form. Keep if GitHub Issues are enabled and this form is still used.
- `.github/ISSUE_TEMPLATE/config.yml`: GitHub issue-form configuration and support links. Keep if the issue form is used; ensure its links point to AstralRinth support channels rather than upstream Modrinth support unless that is intentional.
- `.github/instructions/i18n-convert.instructions.md`: editor/AI instruction for Vue localization. Keep only if the team still uses this instruction; it is not required by GitHub Actions or the application build.

### Assets and upstream automation

- `.github/assets/` currently contains `api_cover.png`, `app_cover.png`, `monorepo_cover.png`, and `web_cover.png`. README files reference some of these images, so do not delete them without first replacing/removing those references. They are Modrinth-branded assets, and the repository's `COPYING.md` explicitly says forks must remove Modrinth branding. Replace referenced covers with AstralRinth artwork, update the READMEs, then remove unused upstream images.
- Do not retain Modrinth deployment, release, triage, merge-queue, or helper-script workflows/actions merely for parity with upstream. Keep a workflow/action only when AstralRinth's own process invokes it; verify callers, `uses:` references, and repository settings before removing any file.
- Root automation such as `scripts/` is separate from `.github/`: remove an upstream script only after checking for package scripts, Cargo scripts, workflow steps, or documentation that invokes it.

When importing upstream changes, review `.github/` as a fork-owned boundary: preserve AstralRinth build/release automation and issue policy, and do not reintroduce upstream CI jobs or helper actions that the fork does not use.

Useful entry points:

- `apps/app-frontend/src/main.js`: frontend application bootstrapping, global plugins, error reporting, and mount.
- `apps/app-frontend/src/App.vue`: desktop application shell, startup state, global navigation, notifications, and cross-cutting wiring.
- `apps/app-frontend/src/routes.js`: desktop routes.
- `apps/app/src/main.rs`: Tauri application/plugin setup and native startup.
- `apps/app/src/api/`: Tauri command/plugin modules and command registration.
- `packages/app-lib/src/lib.rs` and `packages/app-lib/src/`: shared launcher state, APIs, events, process handling, authentication, and game launch/install code.

## AstralRinth feature map

The fork-specific implementation is distributed across frontend, Tauri, and Rust library code. The directories named `astralrinth` are useful discovery points, but are not a complete list of fork behavior.

### External Minecraft authentication and provider libraries

- Frontend provider model and command adapters: `apps/app-frontend/src/models/astralrinth/authentication.ts`.
- Account flows and error/input dialogs: `apps/app-frontend/src/components/ui/astralrinth/accounts/`, called from `apps/app-frontend/src/components/ui/AccountsCard.vue`.
- Provider/library settings UI: `apps/app-frontend/src/components/ui/settings/astralrinth/ExternalAuthLibrarySettings.vue` and the AstralRinth settings page.
- Tauri commands and OAuth verification window: `apps/app/src/api/astralrinth/mod.rs`.
- Provider metadata, OAuth/device flows, and provider library lifecycle: `packages/app-lib/src/models/astralrinth/` and corresponding state/API modules.
- Provider icons and UI styling: `packages/assets/icons/astralrinth/` and `packages/assets/styles/astralrinth/`.

Treat this as an end-to-end contract: UI provider IDs, Tauri command names and serialized data, Rust provider metadata, credential storage, and launcher argument construction must remain consistent. When changing one layer, trace the call through the others. Do not substitute Modrinth-account sign-in for Minecraft-account sign-in; these are separate flows.

### AstralRinth launcher self-updates (Xorison)

The fork's launcher update is separate from upstream Modrinth app updates:

1. `apps/app-frontend/src/helpers/astralrinth/update.ts` checks the Xorison-hosted AstralRinth release API, compares versions, selects platform installer assets, and starts installation.
2. `apps/app-frontend/src/App.vue` runs the check and presents the update notification/modal entry point.
3. `apps/app-frontend/src/components/ui/astralrinth/LauncherUpdateModal.vue` presents release information and installer selection.
4. `apps/app-frontend/src/components/ui/settings/astralrinth/UpdateSettings.vue` provides the related settings surface.
5. `packages/app-lib/src/api/astralrinth/update.rs` and `apps/app/src/api/astralrinth/` provide native update support.

Do not conflate this with `apps/app-frontend/src/providers/app-update.ts` and `app-update-button/`, which represent the upstream app-update UI/state. The upstream provider's action wiring is not the Xorison update path. Preserve and test each update flow independently.

### Skins, Ears, and account-specific behavior

- Skin page and fork account UI: `apps/app-frontend/src/pages/Skins.vue` and `apps/app-frontend/src/components/ui/astralrinth/skin/UnsupportedSkinAccount.vue`.
- Ears controls and skin rendering: `apps/app-frontend/src/helpers/rendering/`, `apps/app-frontend/src/components/ui/skin/`, and the skin page.
- Account card/avatar rendering: `apps/app-frontend/src/components/ui/AccountsCard.vue` and `helpers/rendering/player-head.ts`.
- Backend authentication, provider metadata, and launcher integration: `packages/app-lib/src/models/astralrinth/` plus `packages/app-lib/src/launcher/`.

The current UI lazily acquires/releases baked previews through `BakedSkinButton` and `skin-previews.ts`. Verify exports at the actual import source: rendering helpers have been reorganized over time, and an import that used to exist in `batch-skin-renderer.ts` may no longer be exported there. Keep account-type checks, custom skin capabilities, Ears behavior, and URL/resource cleanup connected when changing the renderer.

### Other fork behavior

- Startup, launcher branding, and privacy override: `apps/app-frontend/src/App.vue`; the current fork sets telemetry off during app startup.
- New-icon-editor notification: `apps/app-frontend/src/components/ui/new-icon-editor-notification/`, invoked from the app startup path. It is a user-facing notification and must not be mistaken for an advertisement or discarded as dead UI without tracing its trigger.
- AstralRinth-specific startup/random text and launcher changes may live in shared files such as `packages/app-lib/src/launcher/mod.rs`, not just fork-named modules.
- Translations are maintained under `apps/app-frontend/src/locales/`; verify English and Russian fork strings when changing message IDs or moving shared UI.
- Fork workflows and release automation also live in `.github/`, including the AstralRinth build workflow.

## Shared upstream contracts and data migrations

The frontend and Rust library mirror several contracts. When touching these, follow the complete data path rather than updating only the visible component:

1. **Settings:** Rust model/defaults and SQLite migration under `packages/app-lib/src/state/` and `packages/app-lib/migrations/` → Tauri settings commands → frontend settings helpers/types → app startup initialization → consumers/settings UI → sync behavior where applicable.
2. **Events:** Rust event enum and emit/serialization code → generated TypeScript event types/codec → frontend listener/consumer. A Rust event schema change requires regenerated or otherwise demonstrably matching frontend bindings.
3. **Commands:** Rust Tauri command/plugin registration → permission/capability definitions → frontend `invoke` wrapper and argument/return types. Commands are not available merely because a Rust function exists.
4. **Minecraft launch/auth:** account credentials and selected external provider → Rust launch context and JVM/auth arguments → native process lifecycle → UI notifications and account state.
5. **Content and downloads:** frontend install/download manager → Tauri command/event contracts → app-lib job state, cancellation/pause, install, and recovery.

Review database migrations for both fresh installs and upgrades from the actual stable release. New settings with a default can silently change established behavior if no migration maps the old state. In particular, verify window refocus-on-game-close, tab visibility (including old per-instance `visible_tabs`), telemetry, and account/skin-related preferences when changing their representation.

## Rules for upstream merges and AI agents

1. **Establish the exact baseline first.** Check `git status`, unresolved paths, current branch/commit, remotes, and tags. For release comparison, use the requested AstralRinth release tag (for example `AR-0.19.202`), not only `HEAD`; this checkout can have merge results in the index/worktree while `HEAD` still points at the previous release.
2. **Inspect both sides of every conflict.** Read the ours and theirs versions and relevant callers. A conflict-free-looking working file may still be unmerged in Git's index. Verify with `git diff --name-only --diff-filter=U`; do not stage files unless explicitly authorized.
3. **Protect fork behavior additively.** Do not resolve a conflict by choosing the entire upstream file if that drops AstralRinth branches, imports, UI, events, settings, or startup calls. Conversely, do not preserve obsolete APIs if upstream replaced them; reconnect the fork behavior to the new API and validate the full path.
4. **Trace integrations.** For every fork-specific behavior, confirm its trigger, UI entry point, Tauri command, permission, Rust implementation, state/settings, event listeners, and cleanup as relevant. Search both direct references and dynamically invoked command/event names.
5. **Treat behavior changes as product decisions.** Compare defaults and migrations against the stable AstralRinth release. Changes such as replacing per-instance navigation settings with global ones or changing whether the launcher refocuses after a game exits are not formatting-only merge resolutions. Document the changed behavior and ask before removing or silently changing a fork capability.
6. **Do not remove user-facing behavior speculatively.** In particular, preserve Xorison update notifications, the new-icon-editor notification, external/offline account flows, Ears/custom skin functionality, and fork settings unless the user explicitly asks to remove them.
7. **Respect explicit removals.** Ads and consent UI/native integration were explicitly requested to be removed in this working tree. Do not restore them from upstream during a merge without asking. `PromotionWrapper.vue` was also explicitly requested to be removed; search for callers before restoring or deleting related wrappers.
8. **Validate after resolution.** Search for conflict markers and missing imports/exports, run focused diagnostics/build checks, and run the app's relevant tests where practical. Report exact checks and failures; do not claim the merge is complete while unresolved Git paths remain.
9. **Keep changes scoped and safe.** Preserve unrelated staged and unstaged edits. Do not commit, create branches, or stage changes without explicit permission.

## Verification commands

Follow root [`AGENTS.md`](AGENTS.md) and project instructions. For the app frontend, its package build script is `pnpm --filter @modrinth/app-frontend build`; repository frontend lint/PR checks should follow the prescribed `pnpm prepr:frontend:app` workflow when requested. A build/type diagnostic is especially useful after resolving Vue component imports and frontend/Rust event contracts. Rust backend checks are documented in the relevant project guidance.

## Baseline and limits

`AR-0.19.202` is the stable AstralRinth release reference used in the recent upstream comparison. It is a comparison point, not proof that every future upstream behavior must remain unchanged. For each upstream update, state explicitly which behavior is preserved, migrated, intentionally replaced, or needs product-owner confirmation. Recheck this file when folder ownership, release/update architecture, or fork-specific integration points change.
