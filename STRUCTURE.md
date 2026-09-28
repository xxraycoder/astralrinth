# AstralRinth repository structure and upstream integration guide

## Purpose

This document describes the Modrinth monorepo as maintained by the AstralRinth fork, identifies where fork-specific behavior lives, and sets expectations for future upstream merges and AI-assisted changes. It supplements the root [`AGENTS.md`](AGENTS.md); it does not replace project-level instructions.

AstralRinth is a fork of Modrinth's codebase, not a separate launcher layered on top. Its desktop launcher shares the upstream app shell, frontend, and Rust app library. A change to an apparently generic Modrinth component, setting, API command, or database model can therefore break a fork-specific flow even when no file named `astralrinth` is edited.

## AstralRinth changes over upstream Modrinth

This section describes the fork-specific delta represented by the AstralRinth patch over the Modrinth App baseline. It is intentionally more detailed than the feature map below: use it when reviewing an upstream merge, deciding whether a file is fork-owned, or checking whether an apparent cleanup would remove a product capability.

### Product identity, branding, and visual presentation

AstralRinth replaces the visible Modrinth App identity in the desktop product with its own identity:

- The Tauri product name, executable name, application identifier, window titles, HTML title, settings label, startup messages, and user-facing error text are changed from Modrinth App to AstralRinth.
- Modrinth launcher artwork is replaced with AstralRinth artwork in the splash screen, application icons, favicon/bundled icons, settings header, and account-provider controls.
- `apps/app-frontend/src/components/ui/SplashScreen.vue` uses the AstralRinth app logo asset instead of the large inline Modrinth SVG and increases the logo presentation for the fork splash screen.
- `packages/assets/icons/astralrinth/` adds the AstralRinth logo and Microsoft, Ely.by, and offline-account icons. `packages/assets/styles/astralrinth/` adds reusable neon text, neon icon, neon button, and soft-input styles.
- `apps/app-frontend/src/App.vue`, settings, instance installation UI, update dialogs, and account dialogs use the fork's neon visual treatment. This is a product-level style layer, not only a logo replacement.
- `apps/app-frontend/package.json` carries an AstralRinth-specific application version, while `apps/app/tauri.conf.json`, `apps/app/tauri.linux.conf.json`, and `apps/app/tauri-release.conf.json` change the packaging identity and remove the upstream updater configuration.

### Account model and Minecraft authentication

AstralRinth expands the upstream Minecraft account model. This is separate from signing into a Modrinth website account.

- Microsoft accounts remain supported, but the account selector becomes a method chooser rather than a single Microsoft sign-in button.
- Offline accounts can be created from the UI with a validated player name. The name is limited to 3–20 ASCII letters, digits, and underscores, and the flow has dedicated input, validation, retry, and unexpected-error modals.
- External Minecraft authentication providers are discovered through native provider metadata. The patch defines Ely.by as a provider with its own OAuth endpoints, profile endpoint, Yggdrasil validation endpoint, Minecraft server name, skin-management URL, and `authlib-injector` release catalog.
- External sign-in uses an OAuth Device Authorization flow. The Tauri host opens a dedicated always-on-top verification WebView, polls the provider until authorization, handles pending/slow-down/denied/expired states, and persists the returned credentials.
- Account records gain `account_type`. Existing data is migrated from the old representation, Microsoft accounts are identified as `microsoft`, and offline accounts as `offline`.
- Credential refresh now distinguishes Microsoft, external-provider, offline, and unknown account types. External refresh tokens use the provider OAuth flow; offline accounts are not sent through online token refresh.
- `AccountsCard.vue`, `MinecraftRequiredModal.vue`, `Skins.vue`, and the new AstralRinth account components share the same account-type-aware flow. Do not restore a direct Microsoft-only button without reconnecting all supported account methods.

The main contract crosses these layers:

1. `apps/app-frontend/src/models/astralrinth/authentication.ts` defines provider metadata, frontend account types, provider loading, OAuth invocation, and library commands.
2. `apps/app-frontend/src/components/ui/AccountsCard.vue` and `components/ui/astralrinth/accounts/` render account methods and errors.
3. `apps/app/src/api/auth.rs` and `apps/app/src/api/astralrinth/mod.rs` expose Tauri commands and own the OAuth verification window.
4. `packages/app-lib/src/models/astralrinth/authentication.rs` owns provider registration, OAuth, token validation, credential persistence, and launch configuration.
5. `packages/app-lib/src/state/minecraft_auth.rs` stores `account_type`, refreshes credentials, and serializes the extended record.
6. `packages/app-lib/src/launcher/mod.rs` converts the selected account into JVM arguments before Minecraft starts.

### `authlib-injector` library management and external launches

AstralRinth adds a complete provider-library lifecycle that does not exist in the upstream Microsoft-only flow:

- `packages/app-lib/src/util/astralrinth/utils.rs` fetches provider release metadata, validates asset names, lists local JARs, downloads exact assets, stores the selected asset in SQLite, and can install the newest compatible library automatically.
- `apps/app-frontend/src/components/ui/settings/astralrinth/ExternalAuthLibrarySettings.vue` exposes per-provider version discovery, refresh, installation, reinstall, local-only fallback, and selection.
- `packages/app-lib/migrations/20260802201752_external-auth-libraries.sql` creates the provider-to-asset selection table.
- When an external account launches, the selected JAR is verified and passed as `-javaagent:<path>=<server>`. Missing or corrupt libraries produce the serializable `external_auth_library_not_installed` error and a dedicated recovery message pointing to AstralRinth settings.
- If no local library exists, the launcher attempts to install the latest compatible provider library. If local libraries exist but the selected one is missing, it reports the error rather than silently changing the user's selection.
- Asset names are restricted to safe JAR file names, preventing path traversal through remote release metadata or persisted selections.

### Offline Minecraft launch behavior

The offline flow changes both authentication and launch arguments:

- `packages/app-lib/src/state/minecraft_auth.rs` creates a local profile with a generated UUID and non-expiring placeholder tokens.
- For Minecraft 1.16.4 and 1.16.5, `packages/app-lib/src/models/astralrinth/authentication.rs` applies the vanilla multiplayer compatibility workaround by setting the Minecraft API hosts to an invalid endpoint and enabling the custom API environment.
- The launcher emits informational events when applying the compatibility workaround or loading an external provider library. These events are surfaced as frontend notifications through the new `info` event path.

### AstralRinth launcher updates through Xorison

The upstream Tauri updater is replaced for the fork's own launcher distribution:

- `apps/app-frontend/src/helpers/astralrinth/update.ts` queries the configured Xorison/Gitea latest-release endpoint, normalizes and compares versions, ignores developer/nightly/dirty release names, filters installers by operating system, and exposes update status and HTTP diagnostics.
- `apps/app-frontend/src/components/ui/astralrinth/LauncherUpdateModal.vue` shows the installed version, latest release tag/title, backup warnings, repository link, installer selection, download state, and failure recovery link.
- `apps/app-frontend/src/components/ui/settings/astralrinth/UpdateSettings.vue` displays release and distribution diagnostics: latest tag, release title, asset count, download count, HTTP status, and API URL.
- `apps/app-frontend/src/App.vue` checks for updates during startup, posts an update notification, marks the settings icon with a neon indicator, and mounts the fork update modal.
- `apps/app/src/api/utils.rs` and `packages/app-lib/src/api/astralrinth/update.rs` download the selected installer into the platform download directory, open or execute it according to the operating system, and exit the current process after a successful handoff.
- `packages/app-lib/.env.prod`, `apps/app-frontend/vite.config.ts`, Tauri capabilities, CSP, and HTTP permissions add the Xorison endpoints and Ely.by endpoints required by this path.
- The upstream `app-update` provider, automatic Modrinth updater prompts, updater signing metadata, updater capability, and `launcher-files.modrinth.com/updates.json` release path are removed or disconnected from the AstralRinth path. The two systems must not be merged accidentally during an upstream update.

### Skin and account-type restrictions

`apps/app-frontend/src/pages/Skins.vue` becomes account-type aware:

- Microsoft accounts retain in-app skin and cape loading, editing, applying, and deletion.
- Offline and external accounts are treated as read-only for Mojang skin operations.
- External providers can expose their own skin-management URL; `UnsupportedSkinAccount.vue` offers a provider-specific link instead of trying to call Mojang endpoints.
- The edit modal, file input, delete confirmation, cape loading, and skin loading are only mounted or executed for Microsoft accounts.
- Existing Ears and custom skin rendering remains part of the shared launcher behavior and must not be removed while adapting these account checks.

### Advertising, consent, promotion, and campaign removals

AstralRinth deliberately removes the Modrinth advertising and promotional surface from the fork:

- The desktop ad WebView, ads Tauri plugin, ads capability, ads bridge/controller/CMP scripts, ad event, ad helpers, consent notification, consent settings, ad window visibility holds, and ad-related modal hooks are removed.
- `PromotionWrapper.vue` and the desktop ad placement are deleted. Download-manager sizing no longer reserves 250 pixels for an ad.
- Website ad components and placements are removed or commented out in project, collection, discovery, organization, and user pages. `apps/frontend/src/public/ads.txt` is deleted.
- Advertising consent text and localization entries are removed from the app locales and privacy settings. The telemetry toggle is disabled when telemetry has already been forced off by the fork startup path.
- The Pride fundraiser banner, Pride badge export, Pride badge definition, Pride campaign assets, and related blog/site media are deleted from the patched tree.
- The app filters news articles whose title, summary, description, or excerpt contains entries from the fork's filtered phrase list. This affects the news feed, not only a single campaign component.

These removals are intentional fork behavior. Do not restore them from upstream as part of a mechanical conflict resolution without an explicit product decision.

### Privacy and analytics behavior

- `apps/app-frontend/src/App.vue` writes `telemetry = false` during startup and does not initialize the upstream analytics launch path in the patched code.
- The fork still keeps the surrounding analytics helper contracts where shared code requires them, so removing or renaming analytics imports must be checked against all callers rather than assuming the entire analytics package is gone.
- The privacy settings UI no longer offers Modrinth advertising consent management. The telemetry control is presented as disabled when the fork has forced telemetry off.

### Startup, notifications, and onboarding changes

- The upstream onboarding checklist is removed from the app shell and its component is deleted. Sidebar visibility is no longer gated by onboarding progress.
- A one-time new icon editor notification is added under `apps/app-frontend/src/components/ui/new-icon-editor-notification/`. It can open a modal that finds iconless instances and applies randomized custom icons through the existing icon editor.
- The app subscribes to a native `info` event and turns backend informational messages into user notifications.
- The upstream `ads_consent_required` event is removed from Rust, generated TypeScript event types, postcard decoding, and frontend event handling.
- Some upstream promotional and hosting-update UI is removed from `App.vue`, while ordinary instance, friend, and launcher functionality remains.

### Discord Rich Presence

AstralRinth replaces the upstream Discord Rich Presence presentation:

- A different Discord application ID is used.
- The asset changes from the Modrinth logo to `astralrinth_logo`.
- Presence includes the AstralRinth version, download and support buttons, and a start timestamp.
- Active and inactive status messages use randomized AstralRinth-specific phrases rather than only `Playing <instance>` or `Idling...`.
- The launcher imports its version from the frontend package metadata, so package-version changes can affect Rich Presence output.

Relevant implementation is in `packages/app-lib/src/state/discord.rs` and `packages/app-lib/src/launcher/mod.rs`.

### Desktop host, Tauri permissions, and security configuration

The fork adds native commands and changes the Tauri boundary:

- Auth commands: `offline_login`, provider discovery, provider-library state, provider-library installation/selection, and external OAuth authentication.
- Utility command: `init_update_launcher`.
- The ads plugin and all associated commands/capabilities are removed.
- `apps/app/capabilities/plugins.json` allows Xorison and Ely.by endpoints and removes `ads:default`.
- `apps/app/tauri.conf.json` changes the application identity and CSP, adds Xorison/Ely.by network origins, removes the ads capability, and uses `mise exec` for frontend build commands.
- `apps/app/tauri-release.conf.json` no longer enables the upstream updater feature, updater public key, Windows signing command, or updater capability in the shown patch.
- `apps/app/src/api/mod.rs` serializes the external-auth-library error into a frontend-stable error code.
- `apps/app-frontend/vite.config.ts` excludes Vue core packages from dependency optimization and accepts the `REPO_XORISON_` environment prefix.

Every new `invoke` call must remain connected to a registered Rust command, a capability, an allowed origin where applicable, and a matching frontend return type.

### Persistence and migration changes

AstralRinth changes initial defaults and adds account/library persistence:

- New installations default to an OLED theme rather than the upstream dark theme.
- Telemetry defaults to disabled.
- `minecraft_users.account_type` is added and existing rows are classified during migration. A follow-up migration renames the legacy `pirate` classification to `offline`.
- `external_auth_libraries` stores the selected provider library asset.
- SQLx offline query snapshots are regenerated to include `account_type` and the changed account upsert/select queries.
- The backup directory changes from a Modrinth-branded path to `AstralRinthApp/Backups/app-db`.

When rebasing migrations, preserve both fresh-install behavior and upgrades from existing Modrinth/AstralRinth databases. The account-type column is not cosmetic: it controls token refresh, skin capabilities, JVM arguments, and launch validation.

### Localization and user-facing text

- New AstralRinth message namespaces cover external authentication, offline accounts, unsupported skins, library management, update dialogs, update diagnostics, and settings.
- English and Russian contain the principal fork translations; the patch also updates or removes corresponding keys in the other generated locale files.
- Existing Modrinth App labels in settings, authentication errors, startup errors, and support text are replaced with AstralRinth wording where the desktop app is fork-owned.
- Support and release links are redirected to Xorison/AstralRinth endpoints in the relevant fork UI, although some upstream support links remain for generic Minecraft or Modrinth functionality.

### Repository, release, and CI/CD changes

The fork also changes repository operations rather than only application code:

- The root README is replaced with AstralRinth installation, feature, support, and Russian-language documentation. `readme/ru_ru/README.md` is added.
- `STRUCTURE.md`, `mise.toml`, the AstralRinth issue form, and an AstralRinth desktop build workflow are added.
- The build workflow targets Linux x86_64/aarch64, Windows x86_64/aarch64, and macOS x86_64/aarch64, installs the required Rust/Node/pnpm/Java tooling, builds Tauri bundles, marks experimental packages, generates SHA-256 checksum files, and uploads artifacts.
- The patch removes or replaces many upstream Modrinth workflows for website deployment, Labrinth deployment, app build/release, Crowdin automation, generic CI, PR cancellation, changelog comments, slash commands, and API-client publishing. These removals mean that upstream workflow files should not be restored blindly.
- `.gitignore` ignores `cmp_*.patch`, allowing local upstream-comparison patches to remain untracked.

### High-risk upstream merge boundaries

The following upstream changes require an explicit AstralRinth review rather than a file-level “take theirs” resolution:

- `App.vue`, `AccountsCard.vue`, `Skins.vue`, settings registration, generated app events, and locale extraction.
- `apps/app/src/api/auth.rs`, `apps/app/src/api/utils.rs`, `apps/app/src/api/mod.rs`, `apps/app/src/main.rs`, capabilities, and Tauri configuration.
- `packages/app-lib/src/state/minecraft_auth.rs`, launcher construction, Discord state, event emission, error serialization, and migrations.
- Any upstream updater, advertising, campaign, onboarding, or promotional changes.
- Any database query or model change involving `account_type`, external library selections, telemetry, theme defaults, or backup paths.

For each upstream update, classify the result as **preserved**, **adapted**, **intentionally removed**, or **requiring product-owner confirmation**. The fork's behavior is distributed across several layers, so a conflict-free file can still break the feature if its command, event, migration, capability, or generated binding counterpart is not updated.

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
| `packages/` (other) | Shared utilities, config, protocol, assets, analytics, and related libraries | Keep packages required by the desktop app or its frontend; verify references before pruning. |
| `scripts/`, `.github/`, `standards/` | Repository automation, fork-owned CI, and engineering standards | Keep only automation and policies that AstralRinth actively uses; upstream Modrinth workflows are not required just because this is a fork. |

The working tree intentionally excludes upstream projects not needed to build the desktop launcher: `apps/frontend/`, `apps/labrinth/`, `apps/docs/`, `apps/daedalus_client/`, and `apps/app-playground/`. The root `docker-compose.yml` for backend-service development is also excluded. These deletions are a fork-maintenance policy, not evidence that upstream has removed those projects.

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
10. **Reapply the repository-pruning policy after every upstream sync.** The omitted website, Labrinth API, docs, playground, Daedalus client, and backend `docker-compose.yml` are intentionally absent from AstralRinth. Upstream updates may modify deleted files (causing modify/delete conflicts) or add new files under these projects. Resolve those paths in favor of deletion, remove any newly reintroduced files, and recheck `Cargo.toml`, `package.json`, workspace/lockfiles, scripts, and CI configuration for references to the excluded projects. Do not restore them merely to make an upstream merge conflict-free.

## Verification commands

Follow root [`AGENTS.md`](AGENTS.md) and project instructions. For the app frontend, its package build script is `pnpm --filter @modrinth/app-frontend build`; repository frontend lint/PR checks should follow the prescribed `pnpm prepr:frontend:app` workflow when requested. A build/type diagnostic is especially useful after resolving Vue component imports and frontend/Rust event contracts. Rust backend checks are documented in the relevant project guidance.

## Baseline and limits

`AR-0.19.202` is the stable AstralRinth release reference used in the recent upstream comparison. It is a comparison point, not proof that every future upstream behavior must remain unchanged. For each upstream update, state explicitly which behavior is preserved, migrated, intentionally replaced, or needs product-owner confirmation. Recheck this file when folder ownership, release/update architecture, or fork-specific integration points change.
