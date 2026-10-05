# AstralRinth repository structure and upstream integration guide

## Purpose

This document describes the repository architecture, package responsibilities, application entry points, fork-specific behavior, and shared integration contracts. Read [`AGENTS.md`](AGENTS.md) for working rules, upstream merge procedures, verification commands, and documentation maintenance requirements.

AstralRinth is a fork of Modrinth's codebase, not a separate launcher layered on top. Its desktop launcher shares the upstream app shell, frontend, and Rust app library. A change to an apparently generic Modrinth component, setting, API command, or database model can therefore break a fork-specific flow even when no file named `astralrinth` is edited.

## Architecture

- **Monorepo tooling:** [Turborepo](https://turbo.build/) (`turbo.jsonc`) + [pnpm workspaces](https://pnpm.io/workspaces) (`pnpm-workspace.yaml`)
- **Frontend:** Vue 3 + Vite for the desktop app, shared Vue components, Tailwind CSS v3; Nuxt 3 in upstream web projects
- **Desktop backend:** Rust + Tauri, shared launcher library, SQLite persistence
- **Upstream backend:** Labrinth API, Postgres, and ClickHouse; the Labrinth app is intentionally excluded from this checkout

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

### Additional shared packages (`packages/`)

| Package            | Description                                           |
| ------------------ | ----------------------------------------------------- |
| `blog`             | Blog system and changelog data                        |
| `utils`            | Shared utility functions (mostly deprecated)          |
| `moderation`       | Moderation utilities                                  |
| `daedalus`         | Daedalus protocol                                     |
| `tooling-config`   | ESLint, Prettier, TypeScript configs                  |
| `ariadne`          | Analytics library                                     |
| `modrinth-log`     | Logging utilities                                     |
| `modrinth-maxmind` | MaxMind GeoIP                                         |
| `modrinth-util`    | General utilities                                     |
| `muralpay`         | Payment processing                                    |
| `path-util`        | Path utilities                                        |
| `sqlx-tracing`     | SQLx query tracing                                    |

## AstralRinth changes over upstream Modrinth

This section describes fork-specific behavior in the current checkout and relevant removals from the Modrinth App baseline. It is intentionally more detailed than the feature map below: use it when reviewing an upstream merge, deciding whether a file is fork-owned, or checking whether an apparent cleanup would remove a product capability.

### Product identity, branding, and visual presentation

AstralRinth replaces the main desktop product identity with its own identity; some upstream wording and metadata remain (for example in privacy and survey text, Cargo metadata, and the API user agent):

- The Tauri product name, executable name, application identifier, window titles, HTML title, settings label, startup messages, and user-facing error text are changed from Modrinth App to AstralRinth.
- Modrinth launcher artwork is replaced with AstralRinth artwork in the splash screen, application icons, favicon/bundled icons, settings header, and account-provider controls.
- `apps/app-frontend/src/components/ui/SplashScreen.vue` uses the AstralRinth app logo asset instead of the large inline Modrinth SVG and increases the logo presentation for the fork splash screen.
- `packages/assets/icons/astralrinth/` adds the AstralRinth logo and Microsoft, Ely.by, and offline-account icons.
- The frontend uses Modrinth components and theme colors. `apps/app-frontend/src/assets/stylesheets/liquid-glass.scss` adds optional translucent surfaces, blur, and layered depth while retaining the selected color theme. It replaces the removed neon text, icon, button, and soft-input styles formerly under `packages/assets/styles/astralrinth/`.
- `apps/app-frontend/src/components/ui/settings/astralrinth/VisualSettings.vue` exposes Liquid Glass in the beta-marked AstralRinth Visual tab. It exposes a three-position Glass level slider (Matte, Standard, Transparent), uses the existing `advanced_rendering` setting for the toggle and local storage for the level, and participates in the settings modal's save/reset flow; the upstream Advanced rendering control is hidden in the app's Appearance settings.
- `apps/app-frontend/package.json` carries an AstralRinth-specific application version, while `apps/app/tauri.conf.json`, `apps/app/tauri.linux.conf.json`, and `apps/app/tauri-release.conf.json` change the packaging identity and remove the upstream updater configuration.

### Account model and Minecraft authentication

AstralRinth expands the upstream Minecraft account model. This is separate from signing into a Modrinth website account.

- Microsoft accounts remain supported, but the account selector becomes a method chooser rather than a single Microsoft sign-in button.
- Offline accounts can be created from the UI with a validated player name. The frontend limits the name to 3–20 ASCII letters, digits, and underscores, and the flow has dedicated input, validation, retry, and unexpected-error modals.
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
- Ely.by release metadata comes from an unauthenticated GET to `https://api.xorison.dev/v1/public/lib/elyby`, without query parameters. Both the frontend catalog and Rust installer consume only `assets["authlib-injector"]`; `assets.authlib` is not a Java agent fallback. Release labels, download counts, and unrelated upstream asset fields are not required by the launcher. Asset names may use a case-insensitive `old_authlib-injector-` prefix. A missing download URL does not prevent listing an asset, but explicit installation reports an error; automatic installation chooses the newest compatible asset with a nonblank `browser_download_url`. HTTP failures, including 404, 502, and non-JSON 429 responses, do not enter success-response parsing; the frontend retains its local-library fallback and refresh cooldown.
- `apps/app-frontend/src/components/ui/settings/astralrinth/ExternalAuthLibrarySettings.vue` exposes per-provider version discovery, refresh, installation, reinstall, local-only fallback, and selection.
- `packages/app-lib/migrations/20260802201752_external-auth-libraries.sql` creates the provider-to-asset selection table.
- When an external account launches, the selected JAR's asset name and local file existence are checked before it is passed as `-javaagent:<path>=<server>`. Missing selections/files or invalid asset names produce the serializable `external_auth_library_not_installed` error; the file-existence check does not validate JAR contents. The frontend provides a dedicated recovery message pointing to AstralRinth settings.
- If no local library exists, the launcher attempts to install the latest compatible provider library. If local libraries exist but the selected one is missing, it reports the error rather than silently changing the user's selection.
- Asset names are restricted to safe JAR file names, preventing path traversal through remote release metadata or persisted selections.

### Offline Minecraft launch behavior

The offline flow changes both authentication and launch arguments:

- `packages/app-lib/src/state/minecraft_auth.rs` creates a local profile with a random UUID, `null` placeholder tokens, and an expiry 99 years in the future. Offline accounts bypass online token refresh.
- For Minecraft 1.16.4 and 1.16.5, `packages/app-lib/src/models/astralrinth/authentication.rs` applies the vanilla multiplayer compatibility workaround by setting the Minecraft API hosts to an invalid endpoint and enabling the custom API environment.
- The launcher emits informational events when applying the compatibility workaround or loading an external provider library. These events are surfaced as frontend notifications through the new `info` event path.

### AstralRinth launcher updates through Xorison

The upstream Tauri updater is replaced for the fork's own launcher distribution:

- `apps/app-frontend/src/helpers/astralrinth/update.ts` queries `${XORISON_API_URL}public/product/astralrinth?version=latest`, normalizes and compares version number parts, selects installers from the release's architecture/OS asset groups, and excludes asset names starting with `dev` or `nightly` (not `dirty`). It does not filter release tags/titles by these prefixes; it exposes update status and HTTP diagnostics. The shared `latestLauncherRelease` ref groups the release payload (`data`) and response status (`httpStatus`); the update modal and settings consumers use this contract. The repository link uses `XORISON_REPO_URL`, while the checked-in `.env.prod` defines `XORISON_GIT_URL` instead; without an externally supplied `XORISON_REPO_URL`, that link is malformed.
- `apps/app-frontend/src/components/ui/astralrinth/LauncherUpdateModal.vue` shows the installed version, latest release tag/title, backup warnings, repository link, installer selection, download state, and failure recovery link.
- `apps/app-frontend/src/components/ui/settings/astralrinth/UpdateSettings.vue` displays release and distribution diagnostics: operating system, architecture, latest tag, release title, asset count, download count, HTTP status, and API URL. OS and architecture come from the Tauri OS plugin; outside Tauri, the UI shows that system information is unavailable.
- `apps/app-frontend/src/App.vue` checks for updates during startup, posts an update notification, marks the settings icon with the theme's brand color, and mounts the fork update modal.
- `apps/app/src/api/utils.rs`, `packages/app-lib/src/util/astralrinth/utils.rs`, and `packages/app-lib/src/api/astralrinth/update.rs` download the selected package into the platform Downloads directory. Windows executes `.exe`/`.msi` installers (with a folder fallback); macOS opens the package; Linux opens the containing folder for manual installation. The helper exits after `get_resource` returns success, but open failures are logged rather than propagated, and the Tauri command discards download errors. UI success therefore does not prove installation or handoff succeeded.
- `packages/app-lib/.env.prod`, `apps/app-frontend/vite.config.ts`, Tauri capabilities, CSP, and HTTP permissions add the Xorison endpoints and Ely.by endpoints required by this path.
- Upstream updater code remains, including `app-update.ts`, `app-update-button/`, Rust updater implementations/dependencies, and `apps/app/capabilities/updater.json`, but it is not wired into the Xorison update path. The default Cargo features and current build workflow do not enable `updater`, and the Tauri configs do not select the updater capability or supply upstream signing configuration. Do not accidentally reactivate it during an upstream update.

### Skin and account-type restrictions

`apps/app-frontend/src/pages/Skins.vue` becomes account-type aware:

- Microsoft accounts retain in-app skin and cape loading, editing, applying, and deletion.
- Offline and external accounts are treated as read-only for Mojang skin operations.
- External providers can expose their own skin-management URL; `UnsupportedSkinAccount.vue` offers a provider-specific link instead of trying to call Mojang endpoints.
- The edit modal, file input, delete confirmation, cape loading, and skin loading are only mounted or executed for Microsoft accounts.
- Existing Ears and custom skin rendering remains part of the shared launcher behavior and must not be removed while adapting these account checks.

### Advertising, consent, promotion, and campaign removals

AstralRinth removes the desktop advertising integration and selected promotional surfaces, but not every upstream campaign or survey feature:

- The desktop ad WebView, ads Tauri plugin, ads capability, ads bridge/controller/CMP scripts, ad event, ad helpers, consent notification, consent settings, ad window visibility holds, and ad-related modal hooks are removed.
- `PromotionWrapper.vue` and the desktop ad placement are deleted. Download-manager sizing no longer reserves 250 pixels for an ad.
- `apps/frontend/`, including its ad placements and `src/public/ads.txt`, is absent from this desktop-only checkout. This is not a claim about a maintained website build.
- Advertising consent text and localization entries are removed from the app locales and privacy settings. The telemetry toggle is disabled when telemetry has already been forced off by the fork startup path.
- Earlier campaign/banner and badge-icon removals are not a blanket removal of campaign support. The current tree still contains `packages/ui/src/components/content/PrideCollectionWidget.vue`, campaign blog articles/media, and campaign-gated Pride skins in `apps/app-frontend/src/pages/Skins.vue`.
- The app filters news articles whose title, summary, description, or excerpt contains entries from the fork's filtered phrase list. This affects the news feed, not only a single campaign component.

These removals are intentional fork behavior. Do not restore them from upstream as part of a mechanical conflict resolution without an explicit product decision.

### Privacy and analytics behavior

- `apps/app-frontend/src/App.vue` writes `telemetry = false` during startup and does not initialize the upstream analytics launch path in the patched code.
- The fork still keeps the surrounding analytics helper contracts where shared code requires them, so removing or renaming analytics imports must be checked against all callers rather than assuming the entire analytics package is gone.
- The privacy settings UI no longer offers Modrinth advertising consent management. The telemetry control is presented as disabled when the fork has forced telemetry off.
- This disables the normal PostHog usage-analytics path, not all reporting or external requests. `apps/app-frontend/src/main.js` still calls `setupErrorReporting`; its production-only Sentry integration initializes on user interaction or errors, enables browser tracing, and does not check `telemetry`. `SurveyPopup.vue` remains mounted and fetches Modrinth surveys on Windows; opening a survey passes the Modrinth user ID to Tally when available. Hosting Intercom integration also remains.

### Startup, notifications, and onboarding changes

- The upstream onboarding checklist UI component is deleted and sidebar visibility is no longer gated by onboarding progress. Its provider/state, native commands, events, and migration remain; `App.vue` still initializes the checklist.
- A new icon editor notification, shown once when its local-storage marker can be persisted, is added under `apps/app-frontend/src/components/ui/new-icon-editor-notification/`. It can open a modal that finds iconless instances and applies randomized custom icons through the existing icon editor.
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
- `apps/app-frontend/vite.config.ts` excludes Vue core packages from dependency optimization and accepts the `XORISON_` environment prefix.

Every new `invoke` call must remain connected to a registered Rust command, a capability, an allowed origin where applicable, and a matching frontend return type.

### Persistence and migration changes

AstralRinth changes initial defaults and adds account/library persistence:

- New installations default to an OLED theme rather than the upstream dark theme.
- Telemetry defaults to disabled.
- `minecraft_users.account_type` is added and existing rows are classified during migration. A follow-up migration renames the legacy `pirate` classification to `offline`.
- `external_auth_libraries` stores the selected provider library asset.
- SQLx offline query snapshots are regenerated to include `account_type` and the changed account upsert/select queries.
- The backup directory changes from a Modrinth-branded path to `AstralRinthApp/Backups/app-db`.
- Liquid Glass enablement reuses SQLite's `advanced_rendering` setting (default `true`); the Glass level is frontend-local under `astralrinth-glass-level`, defaults to `standard`, and is not part of database migrations or account appearance sync.

When rebasing migrations, preserve both fresh-install behavior and upgrades from existing Modrinth/AstralRinth databases. The account-type column is not cosmetic: it controls token refresh, skin capabilities, JVM arguments, and launch validation.

### Localization and user-facing text

- New AstralRinth message namespaces cover external authentication, offline accounts, unsupported skins, library management, update dialogs, update diagnostics, and settings.
- English and Russian contain the principal fork translations; the patch also updates or removes corresponding keys in the other generated locale files.
- Existing Modrinth App labels in settings, authentication errors, startup errors, and support text are replaced with AstralRinth wording where the desktop app is fork-owned.
- Support and release links are redirected to Xorison/AstralRinth endpoints in the relevant fork UI, although some upstream support links remain for generic Minecraft or Modrinth functionality.

### Repository, release, and CI/CD changes

The fork also changes repository operations rather than only application code:

- The root README is replaced with AstralRinth installation, feature, support, and Russian-language documentation. `readme/ru_ru/README.md` is added.
- `STRUCTURE.md` documents the repository architecture, fork-specific behavior, and integration contracts; `AGENTS.md` contains agent instructions and links to this guide. The fork also adds `mise.toml`, the AstralRinth issue form, and an AstralRinth desktop build workflow.
- The build workflow targets Linux x86_64/aarch64, Windows x86_64/aarch64, and macOS x86_64/aarch64, installs the required Rust/Node/pnpm/Java tooling, builds Tauri bundles, marks experimental packages, generates SHA-256 checksum files, and uploads GitHub Actions artifacts. It does not publish GitHub/Xorison releases. It runs on configured branch/tag pushes and manual dispatch, not pull requests; Linux ARM64, Windows ARM64, and macOS x86_64 packages receive the `nightly_expiremental_` filename prefix.
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

## GitHub repository metadata (`.github/`)

Treat `.github/` as configuration for this repository, not as a place to preserve every file from upstream Modrinth. Keep files that serve an active AstralRinth workflow, GitHub feature, or repository instruction; remove upstream automation that is not part of AstralRinth's CI/CD or release process.

### Keep while they are in use

- `.github/workflows/astralrinth-build.yml`: AstralRinth's desktop build pipeline and its platform artifacts/checksums. Keep and evolve this workflow as the fork's own CI/CD contract.
- `.github/ISSUE_TEMPLATE/astralrinth-bug.yml`: the sole bug-report form in the patch, scoped to AstralRinth-specific issues and collecting affected area, OS, launcher version, install source, account type, reproduction steps, logs, and system details. The upstream app, website, hosting, API bug forms, and generic feature-request form are removed; do not restore them without a product decision.
- `.github/ISSUE_TEMPLATE/config.yml`: disables blank issues and directs users to AstralRinth Telegram support, while retaining the Modrinth Support Portal link. Keep these links consistent with the actual support policy rather than assuming every link must point to AstralRinth.
- `.github/instructions/i18n-convert.instructions.md`: editor/AI instruction for Vue localization. Keep only if the team still uses this instruction; it is not required by GitHub Actions or the application build.

### Assets and upstream automation

- `.github/assets/` currently contains `api_cover.png`, `app_cover.png`, `monorepo_cover.png`, and `web_cover.png`. `apps/app/README.md` references `app_cover.png`, so do not delete that cover without first replacing/removing the reference. They are Modrinth-branded assets, and the repository's `COPYING.md` explicitly says forks must remove Modrinth branding. Replace referenced covers with AstralRinth artwork, update the READMEs, then remove unused upstream images.
- Do not retain Modrinth deployment, release, triage, merge-queue, or helper-script workflows/actions merely for parity with upstream. Keep a workflow/action only when AstralRinth's own process invokes it; verify callers, `uses:` references, and repository settings before removing any file.
- Root automation such as `scripts/` is separate from `.github/`: remove an upstream script only after checking for package scripts, Cargo scripts, workflow steps, or documentation that invokes it.

When importing upstream changes, review `.github/` as a fork-owned boundary: preserve AstralRinth build/release automation and issue policy, and do not reintroduce upstream CI jobs or helper actions that the fork does not use.

## Application entry points

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
- Provider icons: `packages/assets/icons/astralrinth/`. Account dialogs use theme-aware component styles; the optional glass override is in `apps/app-frontend/src/assets/stylesheets/liquid-glass.scss`.

Treat this as an end-to-end contract: UI provider IDs, Tauri command names and serialized data, Rust provider metadata, credential storage, and launcher argument construction must remain consistent. When changing one layer, trace the call through the others. Do not substitute Modrinth-account sign-in for Minecraft-account sign-in; these are separate flows.

### AstralRinth launcher self-updates (Xorison)

The fork's launcher update is separate from upstream Modrinth app updates:

1. `apps/app-frontend/src/helpers/astralrinth/update.ts` checks the Xorison-hosted AstralRinth release API, compares versions, selects platform installer assets, and starts installation.
2. `apps/app-frontend/src/App.vue` runs the check and presents the update notification/modal entry point.
3. `apps/app-frontend/src/components/ui/astralrinth/LauncherUpdateModal.vue` presents release information and installer selection.
4. `apps/app-frontend/src/components/ui/settings/astralrinth/UpdateSettings.vue` provides the related settings surface.
5. `apps/app/src/api/utils.rs` registers `plugin:utils|init_update_launcher`; `packages/app-lib/src/util/astralrinth/utils.rs` and `packages/app-lib/src/api/astralrinth/update.rs` download/open the package and handle process exit. The Tauri `api/astralrinth/` module handles authentication, not updates.

Do not conflate this with `apps/app-frontend/src/providers/app-update.ts` and `app-update-button/`, which represent the upstream app-update UI/state. The upstream provider's action wiring is not the Xorison update path, and its presence does not imply a second active self-update flow. Keep it disconnected unless explicitly re-enabled as a product decision.

### AstralRinth news service (Xorison)

- `apps/app-frontend/src/helpers/astralrinth/news.ts` exposes `fetchAstralRinthNews(signal?)` and `AstralRinthNewsArticle`. It sends an unauthenticated GET to `${XORISON_API_URL}public/product/astralrinth/news` through the Tauri HTTP plugin, with no query parameters or request body. It supplies the AstralRinth news modal separately from the existing Modrinth news feed.
- `apps/app-frontend/src/providers/astralrinth-news.ts` owns the shared TanStack Query state, injected by `App.vue`. `setupApp()` explicitly calls the provider's `load()` to start a non-blocking news request during frontend initialization; an API failure is presented in the news UI, not treated as a fatal startup failure. Automatic query execution, retries, and mount/focus/reconnect refetches are disabled. Modal open/close and page changes reuse this state without requesting news. The provider passes the query abort signal to the service; the App-owned observer remains while the modal is closed, so closing the modal does not cancel a request, but unmounting the app does.
- `apps/app-frontend/src/components/ui/astralrinth/news/index.vue` adds an independent sidebar button alongside the unchanged Modrinth news feed. A brand-colored accessible indicator appears when the latest article's age is between zero and four days inclusive; future-dated and older latest articles do not activate it. This is a recency indicator, not unread state: opening the modal does not clear it, and its age condition is reevaluated over time.
- The scrollable `NewModal` displays two articles per client-side page with shared pagination in its fixed action area. Opening the modal or receiving a changed dataset resets to page one; changing page scrolls the content to the top without an API request. There are no manual refresh or error retry buttons; news loads during startup only. The startup loader uses TanStack Query state to skip an in-flight or already completed request, including failed attempts; no manual refresh state or cooldown is retained.
- `astralrinth-news-card.vue` renders image/title/summary/date previews with locale-aware dates. A localized `New` badge beside each date uses the provider's shared reactive recency check (zero to four days inclusive), also used by the sidebar indicator; future-dated and older articles have no badge. Banner images retain their proportions without cropping, with maximum width 512px, maximum height 300px, and responsive width limits. Nullable or broken images do not hide the text; only HTTP(S) links and image sources or embedded PNG/JPEG data URIs are accepted. Use HTTPS for hosted images: the app CSP permits arbitrary HTTPS origins but restricts HTTP images to its explicit exceptions. Article links open through the existing app-wide external-link handler. Duplicate IDs are supported by including the list index in card keys. Loading, empty, error, and rate-limit states use the shared localization system; locale JSON files are maintained separately.
- Every response article must contain `id`, `title`, `summary`, `url`, and `published_at`; `image_url` may be omitted. The client validates string/null types and a finite `Date.parse(published_at)`, normalizes omitted images to `null` for UI callers, removes extra properties, and sorts newest first client-side. The API preserves source order without sorting or pagination. Equal timestamps retain response order; empty strings, duplicate IDs, and future publication dates are not filtered.
- The external API preserves supplied `image_url` strings and explicit `null` unchanged, omitting the field when absent. Neither the server nor the client helper resolves filenames, loads image files, or encodes/decodes image data. Locally hosted images use public URLs such as `https://xorison.dev/images/news.png`; existing external URLs and data URIs are also preserved. UI rendering applies its own safe-source checks.
- News JSON is read on every API request. Images are served independently; missing image files do not fail the news endpoint. Browser caching can affect replaced images, so a new filename provides a new image URL. The public image browser at `https://xorison.dev/images/` is a separate static website route, not an API endpoint.
- HTTP failures, including `429` and `500` from unreadable or invalid news JSON, throw `AstralRinthNewsError` with `httpStatus` before success-response parsing. Malformed successful responses also throw this error; transport/cancellation errors propagate. The AstralRinth feed remains separate from the Modrinth news filtering; opening its modal does not replace or merge the two feeds.

### Skins, Ears, and account-specific behavior

- Skin page and fork account UI: `apps/app-frontend/src/pages/Skins.vue` and `apps/app-frontend/src/components/ui/astralrinth/skin/UnsupportedSkinAccount.vue`.
- Ears controls and skin rendering: `apps/app-frontend/src/helpers/rendering/`, `apps/app-frontend/src/components/ui/skin/`, and the skin page.
- Account card/avatar rendering: `apps/app-frontend/src/components/ui/AccountsCard.vue` and `helpers/rendering/player-head.ts`.
- Backend authentication, provider metadata, and launcher integration: `packages/app-lib/src/models/astralrinth/` plus `packages/app-lib/src/launcher/`.

The current UI lazily acquires/releases baked previews through `BakedSkinButton` and `skin-previews.ts`. Verify exports at the actual import source: rendering helpers have been reorganized over time, and an import that used to exist in `batch-skin-renderer.ts` may no longer be exported there. Keep account-type checks, custom skin capabilities, Ears behavior, and URL/resource cleanup connected when changing the renderer.

### Liquid Glass and visual settings

- Settings registration: `apps/app-frontend/src/components/ui/modal/AppSettingsModal.vue`; the AstralRinth Visual tab is marked as beta.
- Controls and save/reset: `apps/app-frontend/src/components/ui/settings/astralrinth/VisualSettings.vue` edits `useTheme().advancedRendering` and `glassLevel` through the modal's unsaved-changes/save/reset flow. The three-position slider is disabled while Liquid Glass is off or a save is in progress; edits apply after saving, not as a live preview.
- Persistence: the toggle saves the existing SQLite `advanced_rendering` setting. `apps/app-frontend/src/composables/use-theme.ts` loads/saves the level separately in WebView local storage under `astralrinth-glass-level`. Missing, invalid, or unreadable stored values fall back to `standard`; storage-write failures are ignored, so the level may not survive a restart if storage is unavailable. The level is not synced with account appearance, and neither control adds a database setting or migration.
- Levels: `matte` uses near-opaque surfaces and 32px blur without glass-surface gradients; `standard` is the default translucent/blurred style with gradients and theme-specific blur; `transparent` lowers background opacity and removes glass backdrop filtering and glass-surface gradients. These settings do not remove unrelated component gradients or decorative app-background accents. Accessibility/unsupported-backdrop-filter fallbacks can still make surfaces opaque.
- Activation and styles: `apps/app-frontend/src/App.vue` watches `advancedRendering` and `glassLevel`, toggles `html.liquid-glass`, and sets `html[data-glass-level]`; `apps/app-frontend/src/assets/stylesheets/global.scss` imports `liquid-glass.scss`.
- Appearance UI: `apps/app-frontend/src/components/ui/settings/display/AppearanceSettings.vue` hides the upstream Advanced rendering control to avoid exposing the same setting twice. Keep the AstralRinth Visual control connected when adapting upstream appearance settings.
- Decoration: glass backgrounds and blur for panels, buttons, switches, and input wrappers use negative-z-index pseudo-elements with `pointer-events: none`; menus retain their own decorative `::before`. Native input backgrounds and slider decoration are handled directly where appropriate. Shadows stay on clipping panel hosts so `overflow-clip` does not cut off the outer glass shadow. The override includes opaque fallbacks, contrast/forced-color handling, and reduced-motion rules.
- Color contract: change background alpha and glass decoration only. Background colors come from the theme's surface tokens or the component's semantic background and are mixed with `transparent`, not another hue. Do not override foreground `color`, `fill`, `stroke`, font styles, theme foreground tokens, or whole-element `opacity`/`filter`; use `backdrop-filter` for glass blur. Existing component interaction and disabled-state styles remain authoritative.
- Button states: quiet/outlined buttons retain their native backgrounds paired with their native hover/focus colors. Their glass overlay is transparent; outlined buttons additionally disable backdrop filtering on `::after` and use `contain: paint` to confine descendant decoration to the button's own rounded bounds during native hover, pressed, and disabled states. Do not let a control's decorative layer paint over the surrounding panel. Do not fix a background-layer defect by recoloring the icon or text.
- CSS maintenance: normalize high-specificity branches in grouped `:is(...)` defaults with `:where(...)` where needed so later background variants and their opaque fallback colors can win. Keep opaque/accessibility fallbacks effective across all themes and glass levels. Audit selector usage against app-reachable markup, dynamic class/attribute producers, teleported UI, and browser pseudo-elements; stylesheet definitions or unused component exports alone do not prove a selector is used.

Treat Liquid Glass as a visual layer, not a replacement for component behavior. Preserve native modal transitions, focus indicators, disabled states, hit areas, and fixed-position menu anchoring. After changing the override or shared component markup, verify mouse, keyboard, scrolling, and nested-menu interactions in the running app; non-intercepting pseudo-elements alone do not prove all interaction paths are unaffected.

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

## Baseline and limits

`AR-0.19.202` is an existing local tag used as the AstralRinth release reference in the recent upstream comparison; it does not establish the latest published stable version. It is a comparison point, not proof that every future upstream behavior must remain unchanged. For each upstream update, state explicitly which behavior is preserved, migrated, intentionally replaced, or needs product-owner confirmation. Recheck this file when folder ownership, release/update architecture, or fork-specific integration points change.
