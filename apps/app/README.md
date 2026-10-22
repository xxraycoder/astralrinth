![AstralRinth](../../packages/assets/icons/astralrinth/astralrinth-logo.svg)

## AstralRinth desktop host

AstralRinth is a Modrinth-based desktop launcher built with [Tauri](https://tauri.app/) and [Vue](https://vuejs.org/).

For downloads and everyday use, see the [AstralRinth guide](../../README.md).

## Development

### Pre-requisites

Before you begin, ensure you have the following installed on your machine:

- [Node.js](https://nodejs.org/en/)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri](https://v2.tauri.app/start/prerequisites/)
- Java 21 for the app library's Gradle build
- [mise](https://mise.jdx.dev/) for the pinned development tools

### Setup

Use the pinned tools and prepare the app library environment as described in [STRUCTURE.md](../../STRUCTURE.md#development-tooling-and-commands), then:

```bash
pnpm install
pnpm app:dev
```

You should now have a development build of the app running with hot-reloading enabled. Any changes you make to the code will automatically refresh the app.
