# AstralRinth

AstralRinth is a Minecraft launcher with its own visual identity, built for a clean and flexible game experience. It offers a Modrinth-style interface with optional Liquid Glass effects, no desktop ad placements, flexible account support, and updates delivered through Xorison services.

## Languages

[Русский](readme/ru_ru/README.md)

## Download and install

1. Open the [AstralRinth releases page](https://xorison.dev/product/astralrinth/).
2. Download the package for your operating system.
3. Run the installer or open the downloaded package.

| File type | Operating system | Notes |
| --- | --- | --- |
| `.msi, .exe` | Windows 10/11 | Run the installer and follow its prompts. |
| `.dmg, .app` | macOS | Open the image and move AstralRinth to Applications. |
| `.deb, .rpm` | Linux | Install it using your distribution's package installer. |
| `.AppImage` | Linux | Make the file executable and run it directly. |

Avoid test releases or package filenames beginning with `dev`, `nightly` for normal use. The build workflow marks experimental platform packages with a `nightly_expiremental_` filename prefix.

## What AstralRinth offers

- AstralRinth branding for the main launcher interface, with custom icons, splash screen, window titles, and a Modrinth-style interface.
- Optional Liquid Glass effects with Matte, Standard, and Transparent levels, controlled from the dedicated AstralRinth visual settings without changing the selected color theme.
- Desktop ad placements and advertising consent UI are removed; usage analytics are forced off at startup. Production Sentry error reporting/browser tracing and Windows survey prompts remain.
- Russian and English translations for AstralRinth-specific screens and messages.
- Minecraft account sign-in with Microsoft, Ely.by, or an offline account. Ely.by sign-in uses OAuth device authorization.
- Account-aware skin controls: Mojang skin editing is available for Microsoft accounts, while unsupported account types are directed to their provider when one is available.
- `authlib-injector` management for external authentication: browse local JARs, install and select a version, or let AstralRinth install a compatible version when needed.
- Offline account support, including a compatibility workaround for Minecraft 1.16.4 and 1.16.5.
- A dedicated AstralRinth settings section and a one-time notification offering to add randomized custom icons to instances that do not have one.
- Launcher updates through Xorison: check for releases, choose the installer for Windows, macOS, or Linux, and view update diagnostics, including your operating system and architecture, in settings.
- Discord Rich Presence with AstralRinth-specific status messages and a launcher download link.

## Getting started

1. Download and install the latest stable release.
2. In **Accounts**, sign in with Microsoft or Ely.by, complete the device-code flow for a supported external account, or create an offline account.
3. Create or choose a Minecraft instance.
4. For Ely.by, AstralRinth attempts to install a compatible `authlib-injector` library automatically if none is installed. To install or select a version manually, use **Settings → AstralRinth → Authentication libraries**.
5. Launch the game. AstralRinth will use the recommended Java version when possible; you can choose Java manually in settings if needed.

## Visual settings

Open **Settings → AstralRinth → Visual** to enable or disable **Liquid Glass** and choose a **Glass level**, then save your changes:

- **Matte:** near-opaque surfaces with stronger blur and no decorative gradients.
- **Standard** (default): translucent surfaces with blur and layered gradients.
- **Transparent:** more transparent surfaces without glass backdrop blur or decorative gradients.

The level slider is disabled when Liquid Glass is off. Changes apply after saving; resetting unsaved changes restores the saved values. Enablement uses the launcher's existing setting, while the level is stored locally for this app installation and is not synced with your account. If local storage is unavailable, the level may revert to Standard after restarting.

This visual option is marked as beta and is separate from the color theme selected in Appearance settings. Accessibility settings or unavailable blur support may use opaque fallback surfaces. Disable Liquid Glass if the effects reduce performance or readability.

## Updates and support

AstralRinth checks Xorison for updates at startup and notifies you when a newer version is reported. In the update window, choose a package from those offered for your operating system and architecture. Back up your launcher data before continuing. The package is downloaded to Downloads: Windows runs the installer, macOS opens the package, and Linux opens the containing folder so you can install it manually. You may need to finish installation outside the launcher.

If an update does not complete, check Downloads and the launcher logs, then open update diagnostics in AstralRinth settings before contacting support. Those diagnostics show release/API status, not confirmation that installation succeeded.

Support: [AstralRinth Telegram channel](https://xorison.dev/product/astralrinth/support)

## Notice

AstralRinth is an independent Modrinth-based project. Please comply with Minecraft's terms and the terms of any account or authentication service you use. We encourage players to own a legitimate Minecraft license.

# Support Our Project (Crypto Wallets)

If you'd like to support development, you can donate via the following crypto wallets:

- Toncoin (TON): UQA5pGOJhIz9UAVEOh5t2ur1QVbNr_FC1eq9bOb3GwTgaiqk
- USDT (TON): UQA5pGOJhIz9UAVEOh5t2ur1QVbNr_FC1eq9bOb3GwTgaiqk
