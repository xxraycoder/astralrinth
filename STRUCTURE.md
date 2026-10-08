# AstralRinth technical reference

## Purpose

This document describes the repository architecture, package responsibilities, development tooling, application entry points, fork-specific behavior, and shared integration contracts. AI working rules, upstream merge procedures, and maintenance instructions are in [`AGENTS.md`](AGENTS.md). User-facing installation and usage information is in [`README.md`](README.md) and its [Russian translation](readme/ru_ru/README.md).

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
| `packages/ui/` | Shared Modrinth Vue component library | The app frontend composes these components through shared props, exports, and layout contracts. |
| `packages/api-client/` | Shared API client | Provides current API types/contracts; the utility package also contains older types. |
| `packages/` (other) | Shared utilities, config, protocol, assets, analytics, and related libraries | Includes dependencies used by the desktop app and its frontend. |
| `scripts/`, `.github/`, `standards/` | Repository automation, fork-owned CI, and engineering standards | Contains automation and policies for the AstralRinth checkout, distinct from upstream deployment infrastructure. |

The working tree intentionally excludes upstream projects not needed to build the desktop launcher: `apps/frontend/`, `apps/labrinth/`, `apps/docs/`, `apps/daedalus_client/`, and `apps/app-playground/`. The root `docker-compose.yml` for backend-service development is also excluded. These deletions are a fork-maintenance policy, not evidence that upstream has removed those projects.

### Additional shared packages (`packages/`)

| Package            | Description                                           |
| ------------------ | ----------------------------------------------------- |
| `blog`             | Blog system and changelog data                        |
| `utils`            | Shared utility functions (mostly deprecated)          |
| `moderation`       | Moderation utilities                                  |
| `daedalus`         | Minecraft metadata querying and parsing utilities     |
| `tooling-config`   | ESLint, Prettier, TypeScript configs                  |
| `ariadne`          | Shared IDs, networking/presence messages, users, and version helpers |
| `modrinth-log`     | Logging utilities                                     |
| `modrinth-maxmind` | MaxMind GeoIP                                         |
| `modrinth-util`    | General utilities                                     |
| `muralpay`         | Payment processing                                    |
| `path-util`        | Path utilities                                        |
| `sqlx-tracing`     | SQLx query tracing                                    |
| `async-minecraft-ping` | Minecraft ServerListPing client                   |
| `component-derive` | Currently unused component procedural macro for Labrinth's experimental API |
| `modrinth-content-management` | Content diffing and dependency/install resolution |
| `neverbounce`      | NeverBounce API client                                |
| `serde-binhum`     | Different Serde behavior for human-readable and binary formats |
| `xredis`          | Redis support library                                 |

Retained packages include backend-oriented libraries, not only desktop launcher dependencies. Excluding Labrinth does not remove all of its supporting packages.

## Development tooling and commands

- The desktop development entry point is `pnpm app:dev --cache=local:r` from the repository root. `local:r` uses the local Turborepo cache in read-only mode, but the `dev` task itself is uncached. Install the tools specified by `mise.toml` and make `mise` available: Tauri's frontend hooks invoke `mise exec`. The app library also invokes Gradle during compilation; the build workflow uses Java 21.
- Before starting local app development, prepare `packages/app-lib/.env` from the appropriate checked-in environment variant (`.env.local` or `.env.prod`). Its `build.rs` reads the active `.env` through `dotenvy`; the development command does not create it automatically. The build workflow copies `.env.prod` to `.env`.
- Frontend type checking without a build is `pnpm --filter @modrinth/app-frontend tsc:check` from the repository root, or `pnpm tsc:check` from `apps/app-frontend/`. The package script runs `vue-tsc --noEmit`.
- Frontend ESLint and Prettier are available through the app frontend's package tooling. Its `lint` script checks the whole frontend; `pnpm exec eslint <file...>` and `pnpm exec prettier --check <file...>` from `apps/app-frontend/` support file-scoped checks.
- Rust crate membership and features are defined in the root and package `Cargo.toml` files. Frontend tool versions and workspace dependencies are defined in `package.json`, `pnpm-workspace.yaml`, and `mise.toml`. Root tooling and mise specify pnpm 12.8.1, Turbo 2.11.5, and Node 24.15.0 (the root engine permits newer Node versions). The frontend inherits the root package-manager version. Rust is pinned to 1.95.0 in both mise and `rust-toolchain.toml`; CMake uses the latest mise release.

## AstralRinth changes over upstream Modrinth

This section describes fork-specific behavior in the current checkout and relevant removals from the Modrinth App baseline. The feature map below lists implementation entry points for those behaviors.

### Product identity, branding, and visual presentation

AstralRinth replaces the main desktop product identity with its own identity; some upstream wording and metadata remain (for example in privacy and survey text, Cargo metadata, and the API user agent):

- The Tauri product name, executable name, application identifier, window titles, HTML title, settings label, startup messages, and user-facing error text are changed from Modrinth App to AstralRinth.
- Modrinth launcher artwork is replaced with AstralRinth artwork in the splash screen, application/bundled icons, settings header, and account-provider controls. The frontend HTML favicon is not correctly connected: `apps/app-frontend/index.html` still references the absent `/vite.svg`. The configured `apps/app/dmg/dmg-background.png` retains the Modrinth logo and “modrinth app” text, so packaging branding replacement is incomplete.
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
- Account records gain `account_type`. SQL migrations classify existing rows; Microsoft accounts are identified as `microsoft`, and offline accounts as `offline`. This does not guarantee compatibility with upstream SQLite migration checksums or legacy JSON imports; see the persistence section.
- Credential refresh now distinguishes Microsoft, external-provider, offline, and unknown account types. External refresh tokens use the provider OAuth flow; offline accounts are not sent through online token refresh. External launches still validate the access token remotely on every launch, so an installed provider library does not enable disconnected launch. Provider validation failures block launch, and external refresh failures do not inherit the Microsoft-specific connectivity fallback.
- `AccountsCard.vue`, `MinecraftRequiredModal.vue`, `Skins.vue`, and the new AstralRinth account components share the same account-type-aware flow. The shared chooser exposes Microsoft, offline, and registered external-provider methods.

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
- Ely.by release metadata comes from an unauthenticated GET to `https://api.xorison.dev/v1/public/lib/elyby/injector`, without query parameters. The frontend catalog and exact Rust installer consume the flat `assets` array, accepting only case-insensitive `authlib-injector-` names; remote `authlib-` and `old_` assets are excluded. Existing local libraries and selections, including legacy `old_` files, remain usable. Automatic installation uses the API's `latest_injector` object, whose nullable `name` and `browser_download_url` identify the highest numeric version independently of asset order. A null name or missing/blank URL fails automatic installation without falling back to an older asset. A missing download URL does not prevent catalog listing, but explicit installation reports an error. Release labels, download counts, and unrelated upstream asset fields are not required by the launcher. HTTP failures, including 404, 502, and non-JSON 429 responses, do not enter success-response parsing; the frontend retains its local-library fallback and refresh cooldown.
- `apps/app-frontend/src/components/ui/settings/astralrinth/ExternalAuthLibrarySettings.vue` exposes per-provider version discovery, refresh, installation, reinstall, local-only fallback, and selection. Its dropdown combines remote assets with installed local versions, preserving legacy selections even after a successful catalog refresh. Install/reinstall is available only for assets still listed remotely; local-only versions remain selectable without downloading.
- `packages/app-lib/migrations/20260802201752_external-auth-libraries.sql` creates the provider-to-asset selection table.
- When an external account launches, the selected JAR's asset name and local file existence are checked before it is passed as `-javaagent:<path>=<server>`. Missing selections/files or invalid asset names produce the serializable `external_auth_library_not_installed` error; the file-existence check does not validate JAR contents. The frontend provides a dedicated recovery message pointing to AstralRinth settings.
- If no local library exists, the launcher attempts to install the latest compatible provider library. If local libraries exist but the selected one is missing, it reports the error rather than silently changing the user's selection.
- Asset names are restricted to safe JAR file names, preventing path traversal through remote release metadata or persisted selections. Provider-library downloads check HTTP success but do not verify cryptographic signatures or checksums.

### Offline Minecraft launch behavior

The offline flow changes both authentication and launch arguments:

- `packages/app-lib/src/state/minecraft_auth.rs` creates a local profile with a random UUID, `null` placeholder tokens, and an expiry 99 years in the future. Offline accounts bypass online token refresh, but backend profile lookup/upsert can still attempt Mojang requests using the placeholder token. Player-name validation is enforced by the frontend, not the native offline-login function.
- For Minecraft 1.16.4 and 1.16.5, `packages/app-lib/src/models/astralrinth/authentication.rs` applies the vanilla multiplayer compatibility workaround by setting the Minecraft API hosts to an invalid endpoint and enabling the custom API environment.
- The launcher emits informational events when applying the compatibility workaround or loading an external provider library. These events are surfaced as frontend notifications through the new `info` event path.

### AstralRinth launcher updates through Xorison

The upstream Tauri updater is replaced for the fork's own launcher distribution:

- `apps/app-frontend/src/helpers/astralrinth/update.ts` queries `${XORISON_API_URL}public/product/astralrinth?version=latest`, normalizes and compares version number parts, selects installers from the release's architecture/OS asset groups, and excludes asset names starting with `dev` or `nightly` (not `dirty`). It does not filter release tags/titles by these prefixes; it exposes update status and HTTP diagnostics. The shared `latestLauncherRelease` ref groups the release payload (`data`) and response status (`httpStatus`); the update modal and settings consumers use this contract. The repository and release-recovery links use `XORISON_GIT_URL` from the app-library environment, matching the checked-in `.env.prod` configuration.
- `apps/app-frontend/src/components/ui/astralrinth/LauncherUpdateModal.vue` shows the installed version, latest release tag/title, backup warnings, repository link, installer selection, download state, and failure recovery link.
- `apps/app-frontend/src/components/ui/settings/astralrinth/UpdateSettings.vue` is exposed only when developer mode is enabled and displays release and distribution diagnostics: operating system, architecture, latest tag, release title, asset count, download count, HTTP status, and API URL. OS and architecture come from the Tauri OS plugin; outside Tauri, the UI shows that system information is unavailable.
- `apps/app-frontend/src/App.vue` checks for updates during startup, posts an update notification, marks the settings icon with the theme's brand color, and mounts the fork update modal.
- `apps/app/src/api/utils.rs`, `packages/app-lib/src/util/astralrinth/utils.rs`, and `packages/app-lib/src/api/astralrinth/update.rs` download the selected package into the platform Downloads directory. Windows executes `.exe`/`.msi` installers (with a folder fallback); macOS opens the package; Linux opens the containing folder for manual installation. The helper exits after `get_resource` returns success, but open failures are logged rather than propagated, and the Tauri command discards download errors. UI success therefore does not prove installation or handoff succeeded.
- The native update download is not Tauri's signed-updater contract. `packages/app-lib/src/api/astralrinth/update.rs` accepts a caller-supplied URL, filename, and OS type, joins the filename directly onto Downloads, and writes response bytes without checking HTTP success. It performs no signature/checksum verification or explicit URL-origin/basename validation; frontend HTTP allowlists and CSP do not constrain its Rust `reqwest` download.
- `packages/app-lib/.env.prod`, `apps/app-frontend/vite.config.ts`, Tauri capabilities, CSP, and HTTP permissions add the Xorison endpoints and Ely.by endpoints required by this path.
- Upstream updater code remains, including `app-update.ts`, `app-update-button/`, Rust updater implementations/dependencies, and `apps/app/capabilities/updater.json`, but it is not wired into the Xorison update path. The default Cargo features and current build workflow do not enable `updater`, and the Tauri configs do not select the updater capability or supply upstream signing configuration. The updater plugin is also no longer registered in `apps/app/src/main.rs`; enabling its retained feature/capability alone would not reconnect it. Its presence does not represent a second active self-update flow.

### Skin and account-type restrictions

`apps/app-frontend/src/pages/Skins.vue` becomes account-type aware:

- Microsoft accounts retain in-app skin and cape loading, editing, applying, and deletion.
- Offline and external accounts are treated as read-only for Mojang skin operations.
- External providers can expose their own skin-management URL; `UnsupportedSkinAccount.vue` offers a provider-specific link instead of trying to call Mojang endpoints.
- The edit modal, file input, delete confirmation, initial cape/skin loading, and account-change loading are gated to Microsoft accounts. This is not enforced on every callback path: a delayed refresh scheduled after applying a skin calls both loaders without rechecking the selected account type, and account switching does not cancel that timer. The frontend gates are not a general native account-type enforcement boundary.
- Without a selected account, the skin page shows a sign-in prompt with the shared account-method chooser (Microsoft, offline, and external providers). There is no demo-editing mode or demo-account type; `EditSkinModal.vue` has no demo prop.
- Ears and custom skin rendering remain part of the shared launcher behavior alongside these account checks.

### Advertising, consent, promotion, and campaign removals

AstralRinth removes the desktop advertising integration and selected promotional surfaces, but not every upstream campaign or survey feature:

- The desktop ad WebView, ads Tauri plugin, ads capability, ads bridge/controller/CMP scripts, ad event, ad helpers, consent notification, consent settings, ad window visibility holds, and ad-related modal hooks are removed.
- `PromotionWrapper.vue` and the desktop ad placement are deleted. Download-manager sizing no longer reserves 250 pixels for an ad.
- `apps/frontend/`, including its ad placements and `src/public/ads.txt`, is absent from this desktop-only checkout. This is not a claim about a maintained website build.
- Advertising consent text and localization entries are removed from the app locales and privacy settings. The telemetry toggle is disabled when telemetry has already been forced off by the fork startup path.
- Earlier campaign/banner removals include the Pride Fundraiser Supporter profile-badge definition and its icon, not merely artwork. These changes are not a blanket removal of campaign support. The current tree still contains `packages/ui/src/components/content/PrideCollectionWidget.vue`, campaign blog articles/media, and campaign-gated Pride skins in `apps/app-frontend/src/pages/Skins.vue`.
- The app filters news articles whose title, summary, description, or excerpt contains entries from the fork's filtered phrase list. This affects the news feed, not only a single campaign component.

These removals are intentional differences from the upstream desktop product.

### Privacy and analytics behavior

- `apps/app-frontend/src/App.vue` writes `telemetry = false` during startup and does not initialize the upstream analytics launch path in the patched code.
- The fork retains analytics helper contracts used by shared code; the analytics package is not entirely removed.
- The privacy settings UI no longer offers Modrinth advertising consent management. The telemetry control is presented as disabled when the fork has forced telemetry off.
- This disables the normal PostHog usage-analytics path, not all reporting or external requests. `apps/app-frontend/src/main.js` still calls `setupErrorReporting`; its production-only Sentry integration initializes on user interaction or errors, enables browser tracing, and does not check `telemetry`. `SurveyPopup.vue` remains mounted and fetches Modrinth surveys on Windows; opening a survey passes the Modrinth user ID to Tally when available. The HTML entry point unconditionally loads `https://tally.so/widgets/embed.js`, independently of telemetry, the Windows survey check, or opening a survey. Hosting Intercom integration also remains.

### Startup, notifications, and onboarding changes

- The upstream onboarding checklist UI component is deleted and sidebar visibility is no longer gated by onboarding progress. Its provider/state, native commands, events, and migration remain; `App.vue` still initializes the checklist.
- A new icon editor notification, shown once when its local-storage marker can be persisted, is added under `apps/app-frontend/src/components/ui/new-icon-editor-notification/`. It can open a modal that finds iconless instances and applies randomized custom icons through the existing icon editor.
- The app subscribes to a native `info` event and turns backend informational messages into user notifications. Rust emits this directly through Tauri with a `{ message }` payload; it is separate from the generated `AppEvent`/postcard transport.
- The upstream `ads_consent_required` event is removed from Rust, generated TypeScript event types, postcard decoding, and frontend event handling.
- Some upstream promotional and hosting-update UI is removed from `App.vue`, while ordinary instance, friend, and launcher functionality remains.

### Discord Rich Presence

AstralRinth replaces the upstream Discord Rich Presence presentation:

- A different Discord application ID is used.
- The asset changes from the Modrinth logo to `astralrinth_logo`.
- Presence includes the AstralRinth version, download and support buttons, and a timestamp reset on each activity update, rather than a persistent launcher/session start.
- Active and inactive status messages use randomized AstralRinth-specific phrases rather than only `Playing <instance>` or `Idling...`. Game exit resets presence to an inactive phrase without checking whether another instance remains running.
- The launcher embeds its version from the frontend package metadata at compile time, so package-version changes affect Rich Presence after rebuilding.

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

The native invocation boundary consists of frontend `invoke` wrappers and return types, registered Rust commands, permission generation in `apps/app/build.rs`, Tauri capabilities, and allowed origins where applicable. Enabled capabilities are local and scoped to the `main` window; the separate `astralrinth-external-signin` OAuth verification window is not granted the main window's native-command capabilities.

### Persistence and migration changes

AstralRinth changes initial defaults and adds account/library persistence:

- New installations default to an OLED theme rather than the upstream dark theme.
- Telemetry defaults to disabled.
- `minecraft_users.account_type` is added and existing rows are classified during migration. A follow-up migration renames the legacy `pirate` classification to `offline`.
- `external_auth_libraries` stores the selected provider library asset.
- SQLx offline query snapshots are regenerated to include `account_type` and the changed account upsert/select queries.
- The backup directory changes from a Modrinth-branded path to `AstralRinthApp/Backups/app-db`.
- Liquid Glass enablement reuses SQLite's `advanced_rendering` setting (default `true`); the Glass level is frontend-local under `astralrinth-glass-level`, defaults to `standard`, and is not part of database migrations or account appearance sync.

Fresh installations, existing SQLite databases, and legacy JSON imports are distinct compatibility paths. The fork modifies the historical initial migration to change theme/telemetry defaults, while startup uses SQLx's checksum-validating migrator. A reused upstream Modrinth database carrying the original migration checksum can fail before later migrations run; compatibility is not unconditional. Existing migration files match the `AR-0.19.202` fork reference, so that checksum mismatch does not by itself establish a failure for upgrades from that release.

SQL migrations classify existing account rows, but the legacy JSON importer requires `LegacyCredentials.account_type` without a Serde default. Legacy account JSON lacking this field fails deserialization and is silently skipped by the importer. The account-type column controls token refresh, skin capabilities, JVM arguments, and launch validation.

### Localization and user-facing text

- New AstralRinth message namespaces cover external authentication, offline accounts, unsupported skins, library management, update dialogs, update diagnostics, and settings.
- English and Russian contain the principal fork translations; the patch also updates or removes corresponding keys in the other generated locale files. Coverage is incomplete: all 14 new-icon-editor notification/modal message IDs are absent from both app locales and rely on default-message fallback; the modal description also retains “Modrinth App”.
- Existing Modrinth App labels in settings, authentication errors, startup errors, and support text are replaced with AstralRinth wording where the desktop app is fork-owned.
- Support and release links are redirected to Xorison/AstralRinth endpoints in the relevant fork UI, although some upstream support links remain for generic Minecraft or Modrinth functionality.

### Repository, release, and CI/CD changes

The fork also changes repository operations rather than only application code:

- The root README is replaced with AstralRinth installation, feature, support, and Russian-language documentation. `readme/ru_ru/README.md` is added.
- `STRUCTURE.md` documents the repository architecture, fork-specific behavior, and integration contracts; `AGENTS.md` contains agent instructions and links to this guide. The fork also adds `mise.toml`, the AstralRinth issue form, and an AstralRinth desktop build workflow.
- The build workflow targets Linux x86_64/aarch64, Windows x86_64/aarch64, and macOS x86_64/aarch64, installs the required Rust/Node/pnpm/Java tooling, builds Tauri bundles, marks experimental packages, generates SHA-256 checksum files, and uploads GitHub Actions artifacts. It does not publish GitHub/Xorison releases. It runs on configured branch/tag pushes and manual dispatch, not pull requests; Linux ARM64, Windows ARM64, and macOS x86_64 packages receive the `nightly_expiremental_` filename prefix.
- The patch removes or replaces many upstream Modrinth workflows for website deployment, Labrinth deployment, app build/release, Crowdin automation, generic CI, PR cancellation, changelog comments, slash commands, and API-client publishing. The resulting CI/CD configuration is fork-owned rather than a mirror of upstream automation.
- Push builds are path-filtered; changes only to omitted paths such as root `Cargo.toml`/`Cargo.lock`, `packages/api-client/`, `packages/tooling-config/`, or some retained Rust dependencies do not trigger branch builds. Tag triggers are `release-*` and `beta-*`, not the existing `AR-*` convention. There are no explicit Rust test, Clippy, formatting, or ESLint steps; frontend type checking runs as part of its build script. Checksums are generated for every regular file recursively in the bundle directory, excluding existing checksum files.
- `.gitignore` ignores `cmp_*.patch`, allowing local upstream-comparison patches to remain untracked.

### Cross-layer integration boundaries

Fork behavior spans the following shared files and subsystems:

- `App.vue`, `AccountsCard.vue`, `Skins.vue`, settings registration, generated app events, and locale extraction.
- `apps/app/src/api/auth.rs`, `apps/app/src/api/utils.rs`, `apps/app/src/api/mod.rs`, `apps/app/src/main.rs`, capabilities, and Tauri configuration.
- `packages/app-lib/src/state/minecraft_auth.rs`, launcher construction, Discord state, event emission, error serialization, and migrations.
- Any upstream updater, advertising, campaign, onboarding, or promotional changes.
- Any database query or model change involving `account_type`, external library selections, telemetry, theme defaults, or backup paths.

These features depend on matching commands, events, migrations, capabilities, and generated bindings across layers. Git conflict status alone does not establish compatibility.

## GitHub repository metadata (`.github/`)

`.github/` contains this repository's build automation, issue configuration, editor instructions, and documentation assets. It is not a complete copy of upstream Modrinth automation.

### Workflow and issue configuration

- `.github/workflows/astralrinth-build.yml`: AstralRinth's desktop build pipeline and its platform artifacts/checksums.
- `.github/ISSUE_TEMPLATE/astralrinth-bug.yml`: the sole bug-report form in the patch, scoped to AstralRinth-specific issues and collecting affected area, OS, launcher version, install source, account type, reproduction steps, logs, and system details. The upstream app, website, hosting, API bug forms, and generic feature-request form are removed.
- `.github/ISSUE_TEMPLATE/config.yml`: disables blank issues and directs users to AstralRinth Telegram support, while retaining the Modrinth Support Portal link. The configuration therefore includes both fork and upstream support destinations.
- `.github/instructions/i18n-convert.instructions.md`: editor/AI instruction for Vue localization, separate from GitHub Actions and the application build.

### Assets and upstream automation

- `.github/assets/` currently contains `api_cover.png`, `app_cover.png`, `monorepo_cover.png`, and `web_cover.png`. `apps/app/README.md` references `app_cover.png`. These assets retain Modrinth branding; `COPYING.md` contains the fork branding requirements.
- Workflow dependencies include `uses:` references, helper actions, and repository settings; root automation under `scripts/` is separate from `.github/` and can also be invoked by package scripts, Cargo scripts, and documentation.
- Some retained scripts are legacy/backend utilities, not ready-to-use desktop-development automation. `clone-labrinth-projects.mjs`, `seed-db.sh`, and `reset-db.sh` assume deleted Labrinth fixtures/environment files; `import-projects.py` assumes a running `labrinth-postgres` Podman container. Some website-specific tooling references remain despite project pruning.

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

UI provider IDs, Tauri command names and serialized data, Rust provider metadata, credential storage, and launcher argument construction form an end-to-end contract. Minecraft-account sign-in and Modrinth-account sign-in are separate flows.

### AstralRinth launcher self-updates (Xorison)

The fork's launcher update is separate from upstream Modrinth app updates:

1. `apps/app-frontend/src/helpers/astralrinth/update.ts` checks the Xorison-hosted AstralRinth release API, compares versions, selects platform installer assets, and starts installation.
2. `apps/app-frontend/src/App.vue` runs the check and presents the update notification/modal entry point.
3. `apps/app-frontend/src/components/ui/astralrinth/LauncherUpdateModal.vue` presents release information and installer selection.
4. `apps/app-frontend/src/components/ui/settings/astralrinth/UpdateSettings.vue` provides the related developer-only diagnostics settings surface.
5. `apps/app/src/api/utils.rs` registers `plugin:utils|init_update_launcher`; `packages/app-lib/src/util/astralrinth/utils.rs` and `packages/app-lib/src/api/astralrinth/update.rs` download/open the package and handle process exit. The Tauri `api/astralrinth/` module handles authentication, not updates.

`apps/app-frontend/src/providers/app-update.ts` and `app-update-button/` represent the upstream app-update UI/state. Their action wiring is separate from the active Xorison update path.

### AstralRinth news service (Xorison)

- `apps/app-frontend/src/helpers/astralrinth/news.ts` exposes `fetchAstralRinthNews(signal?)` and `AstralRinthNewsArticle`. It sends an unauthenticated GET to `${XORISON_API_URL}public/product/astralrinth/news` through the Tauri HTTP plugin, with no query parameters or request body. It supplies the AstralRinth news modal separately from the existing Modrinth news feed.
- `apps/app-frontend/src/providers/astralrinth-news.ts` owns the shared TanStack Query state, injected by `App.vue`. `setupApp()` explicitly calls the provider's `load()` to start a non-blocking news request during frontend initialization; an API failure is presented in the news UI, not treated as a fatal startup failure. Automatic query execution, retries, and mount/focus/reconnect refetches are disabled. Modal open/close and page changes reuse this state without requesting news. The provider passes the query abort signal to the service; the App-owned observer remains while the modal is closed, so closing the modal does not cancel a request, but unmounting the app does.
- `apps/app-frontend/src/components/ui/astralrinth/news/index.vue` adds an independent sidebar button alongside the unchanged Modrinth news feed. A brand-colored accessible indicator appears when the latest article's age is between zero and four days inclusive; future-dated and older latest articles do not activate it. This is a recency indicator, not unread state: opening the modal does not clear it, and its age condition is reevaluated over time.
- The scrollable `NewModal` displays two articles per client-side page with shared pagination in its fixed action area. Opening the modal or receiving a changed dataset resets to page one; changing page scrolls the content to the top without an API request. There are no manual refresh or error retry buttons; news loads during startup only. The startup loader uses TanStack Query state to skip an in-flight or already completed request, including failed attempts; no manual refresh state or cooldown is retained.
- `astralrinth-news-card.vue` renders image/title/summary/date previews with locale-aware dates. A localized `New` badge beside each date uses the provider's shared reactive recency check (zero to four days inclusive), also used by the sidebar indicator; future-dated and older articles have no badge. Banner images retain their proportions without cropping, with maximum width 512px, maximum height 300px, and responsive width limits. Nullable or broken images do not hide the text; only HTTP(S) links and image sources or embedded PNG/JPEG data URIs are accepted. The app CSP permits arbitrary HTTPS image origins but restricts HTTP images to its explicit exceptions. Article links open through the existing app-wide external-link handler. Duplicate IDs are supported by including the list index in card keys. Loading, empty, error, and rate-limit states use the shared localization system; English and Russian message entries are in `apps/app-frontend/src/locales/en-US/index.json` and `apps/app-frontend/src/locales/ru-RU/index.json`.
- Every response article must contain `id`, `title`, `summary`, `url`, and `published_at`; `image_url` may be omitted. The client validates string/null types and a finite `Date.parse(published_at)`, normalizes omitted images to `null` for UI callers, removes extra properties, and sorts newest first client-side. Server-side ordering/pagination is an externally supplied contract, not verified by this repository. Equal timestamps retain response order; empty strings, duplicate IDs, and future publication dates are not filtered.
- The client helper does not resolve filenames, load image files, or encode/decode image data; it normalizes omitted images to `null` and otherwise preserves accepted image strings/null. UI rendering applies its own safe-source checks. The external service's preservation/omission of image fields and image-hosting behavior are externally supplied contracts, not repository-verified implementation details.
- The externally supplied service contract describes news JSON being read per request, independently served images, and a separate static image browser at `https://xorison.dev/images/`. This checkout does not contain that server implementation and cannot verify those behaviors. Browser caching can affect replaced images; changing the filename provides a new image URL.
- HTTP failures, including `429` and `500`, throw `AstralRinthNewsError` with `httpStatus` before success-response parsing. The client cannot establish the server-side cause of a failure, such as unreadable or invalid news JSON. Malformed successful responses also throw this error; transport/cancellation errors propagate. The AstralRinth feed remains separate from the Modrinth news filtering; opening its modal does not replace or merge the two feeds.

### Skins, Ears, and account-specific behavior

- Skin page and fork account UI: `apps/app-frontend/src/pages/Skins.vue` and `apps/app-frontend/src/components/ui/astralrinth/skin/UnsupportedSkinAccount.vue`.
- Ears controls and skin rendering: `apps/app-frontend/src/helpers/rendering/`, `apps/app-frontend/src/components/ui/skin/`, and the skin page.
- Account card/avatar rendering: `apps/app-frontend/src/components/ui/AccountsCard.vue` and `helpers/rendering/player-head.ts`.
- Backend authentication, provider metadata, and launcher integration: `packages/app-lib/src/models/astralrinth/` plus `packages/app-lib/src/launcher/`.

The current UI lazily acquires/releases baked previews through `BakedSkinButton` and `skin-previews.ts`. Rendering helpers have been reorganized over time; older imports from `batch-skin-renderer.ts` may no longer match current exports. Account-type checks, custom skin capabilities, Ears behavior, and URL/resource cleanup are coupled to this rendering lifecycle.

### Liquid Glass and visual settings

- Settings registration: `apps/app-frontend/src/components/ui/modal/AppSettingsModal.vue`; the AstralRinth Visual tab is marked as beta.
- Controls and save/reset: `apps/app-frontend/src/components/ui/settings/astralrinth/VisualSettings.vue` edits `useTheme().advancedRendering` and `glassLevel` through the modal's unsaved-changes/save/reset flow. The three-position slider is disabled while Liquid Glass is off or a save is in progress; edits apply after saving, not as a live preview.
- Persistence: the toggle saves the existing SQLite `advanced_rendering` setting. `apps/app-frontend/src/composables/use-theme.ts` loads/saves the level separately in WebView local storage under `astralrinth-glass-level`. Missing, invalid, or unreadable stored values fall back to `standard`; storage-write failures are ignored, so the level may not survive a restart if storage is unavailable. The level is not synced with account appearance, and neither control adds a database setting or migration.
- Levels: `matte` uses near-opaque surfaces and 32px blur without glass-surface gradients; `standard` is the default translucent/blurred style with gradients and theme-specific blur; `transparent` lowers background opacity and removes glass backdrop filtering and glass-surface gradients. These settings do not remove unrelated component gradients or decorative app-background accents. Accessibility/unsupported-backdrop-filter fallbacks can still make surfaces opaque.
- Activation and styles: `apps/app-frontend/src/App.vue` watches `advancedRendering` and `glassLevel`, toggles `html.liquid-glass`, and sets `html[data-glass-level]`; `apps/app-frontend/src/assets/stylesheets/global.scss` imports `liquid-glass.scss`.
- Appearance UI: `apps/app-frontend/src/components/ui/settings/display/AppearanceSettings.vue` hides the upstream Advanced rendering control to avoid exposing the same setting twice. The AstralRinth Visual tab is the app's control surface for this shared setting.
- Decoration: glass backgrounds and blur for panels, buttons, switches, and input wrappers use negative-z-index pseudo-elements with `pointer-events: none`; menus retain their own decorative `::before`. Native input backgrounds and slider decoration are handled directly where appropriate. Shadows stay on clipping panel hosts so `overflow-clip` does not cut off the outer glass shadow. The override includes opaque fallbacks, contrast/forced-color handling, and reduced-motion rules.
- Color contract: glass decoration changes background alpha while foreground colors, font styles, and component interaction/disabled states remain owned by the theme and components. Background colors come from theme surface tokens or semantic component backgrounds and are mixed with `transparent`. Blur uses `backdrop-filter`, not whole-element `opacity`/`filter`.
- Button states: quiet/outlined buttons retain their native backgrounds paired with their native hover/focus colors. Their glass overlay is transparent; outlined buttons additionally disable backdrop filtering on `::after` and use `contain: paint` to confine descendant decoration to the button's own rounded bounds during native hover, pressed, and disabled states. This containment separates button decoration from the surrounding panel without changing icon or text colors.
- CSS specificity: grouped `:is(...)` defaults can inherit high specificity; `:where(...)` provides zero-specificity branches so background variants and opaque fallbacks can win. Selector reachability includes app markup, dynamic classes/attributes, teleported UI, and browser pseudo-elements.

Liquid Glass is a visual layer over native component behavior, including modal transitions, focus indicators, disabled states, hit areas, and fixed-position menu anchoring. Decorative pseudo-elements are non-intercepting; interaction behavior also depends on stacking, containment, and positioning.

### Other fork behavior

- Startup, launcher branding, and privacy override: `apps/app-frontend/src/App.vue`; the current fork sets telemetry off during app startup.
- New-icon-editor notification: `apps/app-frontend/src/components/ui/new-icon-editor-notification/`, invoked from the app startup path. It is a user-facing icon-management notification, separate from removed advertising surfaces.
- AstralRinth-specific startup/random text and launcher changes may live in shared files such as `packages/app-lib/src/launcher/mod.rs`, not just fork-named modules.
- Translations are maintained under `apps/app-frontend/src/locales/`, including the English and Russian fork messages.
- Fork workflows and release automation also live in `.github/`, including the AstralRinth build workflow.

## Shared upstream contracts and data migrations

The frontend and Rust library mirror several contracts across these data paths:

1. **Settings:** Rust model/defaults and SQLite migration under `packages/app-lib/src/state/` and `packages/app-lib/migrations/` → Tauri settings commands → frontend settings helpers/types → app startup initialization → consumers/settings UI → sync behavior where applicable.
2. **Events:** Rust event enum and emit/serialization code → generated TypeScript event types/codec → frontend listener/consumer. The frontend codec depends on matching the Rust event schema. Direct Tauri events such as `info` use a separate payload/listener contract rather than this generated codec.
3. **Commands:** Rust Tauri command/plugin registration → generated permissions in `apps/app/build.rs` and capability definitions → frontend `invoke` wrapper and argument/return types. Commands are not available merely because a Rust function exists.
4. **Minecraft launch/auth:** account credentials and selected external provider → Rust launch context and JVM/auth arguments → native process lifecycle → UI notifications and account state.
5. **Content and downloads:** frontend install/download manager → Tauri command/event contracts → app-lib job state, cancellation/pause, install, and recovery.

Fresh-install defaults and upgrades from stable releases are distinct persistence paths. Settings migrations map existing state to new representations; affected preferences include window refocus-on-game-close, tab visibility (including legacy per-instance `visible_tabs`), telemetry, and account/skin-related settings. Compared with `AR-0.19.202`'s unconditional launcher refocus after game exit, current process handling requires `refocus_on_game_close`. Its migration preserves an explicit feature flag but defaults missing flags to `false`, so upgrades without that flag no longer automatically refocus the launcher.

## Baseline and limits

`patches/cmp_v0.21.6.patch` uses upstream `v0.21.6` (`23ab5cdc331e40712f12b9026979359aec1e404d`) as its old-side baseline; all 2,352 patch section paths/blob identities matched that tag during the static review. This is distinct from the fork behavior reference.

`AR-0.19.202` (`65add242443c22d15031c0a447412db1b0bd3a2a`) is an existing local tag used as the AstralRinth release reference in the recent upstream comparison; it does not establish the latest published stable version. It is a comparison point, not proof that every future upstream behavior must remain unchanged. The reference describes the current checkout's architecture and contracts, not an independent audit of the published release or external services. The patch review inspected branch `beta` at `4304bf764b95b3e6c076d9c06922389a6e42459c`. Validation used static patch/blob comparisons, targeted code/configuration inspection, and focused in-memory migration checks; it did not run builds, platform installers, runtime glass interactions, or external API requests. Environment-template contents were unavailable under the editor's private-file policy.
