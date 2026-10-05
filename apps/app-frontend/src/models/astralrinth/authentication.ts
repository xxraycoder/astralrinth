import { ElyByIcon, ExternalIcon } from '@modrinth/assets'
import { invoke } from '@tauri-apps/api/core'
import { fetch } from '@tauri-apps/plugin-http'
import type { Component } from 'vue'
import { ref } from 'vue'

export type ExternalAuthProvider = {
	id: string
	accountOptionId: string
	icon: Component
	libraryReleaseUrl: string
	name: string
	skinManagementUrl?: string
}

type ExternalAuthLibraryState = {
	providerId: string
	selectedAssetName: string | null
	localAssetNames: string[]
}

type ExternalAuthLibraryCatalogEntry = {
	provider: ExternalAuthProvider
	assetNames: string[] | null
}

export type MinecraftCredential = {
	account_type?: string
	profile: {
		id: string
		name: string
	}
}

type ExternalAuthenticationOptions = {
	onAuthenticated: (credentials: MinecraftCredential) => Promise<void>
	onError: (error: unknown) => void
}

type ExternalAuthProviderMetadata = {
	id: string
	displayName: string
	icon: string
	libraryReleaseUrl: string
	skinManagementUrl?: string
}

const externalAuthProviderIcons: Record<string, Component> = {
	elyby: ElyByIcon,
}

export const externalAuthProviders = ref<ExternalAuthProvider[]>([])

let providerLoad: Promise<ExternalAuthProvider[]> | undefined
let externalAuthLibraryCatalog: ExternalAuthLibraryCatalogEntry[] | undefined
let externalAuthLibraryCatalogNextRefreshAt = 0

const externalAuthLibraryCatalogRefreshCooldownMs = 30_000
const externalAuthLibraryRequestTimeoutMs = 15_000

/** Loads provider metadata once and adapts it for account-selection controls. */
export async function loadExternalAuthProviders() {
	if (externalAuthProviders.value.length > 0) {
		return externalAuthProviders.value
	}

	const load =
		providerLoad ??
		invoke<ExternalAuthProviderMetadata[]>('plugin:auth|get_external_auth_providers').then(
			(providers) => {
				externalAuthProviders.value = providers.map((provider) => ({
					id: provider.id,
					accountOptionId: `add_external_${provider.id}_account`,
					icon: externalAuthProviderIcons[provider.icon] ?? ExternalIcon,
					libraryReleaseUrl: provider.libraryReleaseUrl,
					name: provider.displayName,
					skinManagementUrl: provider.skinManagementUrl,
				}))

				return externalAuthProviders.value
			},
		)
	providerLoad = load

	try {
		return await load
	} finally {
		if (providerLoad === load) {
			providerLoad = undefined
		}
	}
}

function parseExternalAuthLibraryAssets(value: unknown): string[] {
	if (
		!value ||
		typeof value !== 'object' ||
		Array.isArray(value) ||
		!('assets' in value) ||
		!Array.isArray(value.assets)
	) {
		throw new Error('The release response does not contain an authlib-injector assets array')
	}

	return value.assets
		.flatMap((asset) =>
			typeof asset === 'object' &&
			asset !== null &&
			'name' in asset &&
			typeof asset.name === 'string'
				? [asset.name]
				: [],
		)
		.filter((assetName) => {
			const name = assetName.toLowerCase()
			return name.startsWith('authlib-injector-')
		})
}

async function fetchExternalAuthLibraryCatalogEntry(
	provider: ExternalAuthProvider,
): Promise<ExternalAuthLibraryCatalogEntry> {
	const controller = new AbortController()
	const timeout = window.setTimeout(() => controller.abort(), externalAuthLibraryRequestTimeoutMs)
	try {
		const response = await fetch(provider.libraryReleaseUrl, { signal: controller.signal })
		if (!response.ok) {
			throw new Error(`HTTP ${response.status}`)
		}

		return {
			provider,
			assetNames: parseExternalAuthLibraryAssets(await response.json()),
		}
	} catch {
		return {
			provider,
			assetNames: null,
		}
	} finally {
		window.clearTimeout(timeout)
	}
}

/** Loads the remote provider-library catalog once per app runtime unless refresh is requested. */
export async function loadExternalAuthLibraryCatalog(
	forceRefresh = false,
): Promise<ExternalAuthLibraryCatalogEntry[]> {
	if (!forceRefresh && externalAuthLibraryCatalog) {
		return externalAuthLibraryCatalog
	}
	if (
		forceRefresh &&
		externalAuthLibraryCatalog &&
		getExternalAuthLibraryCatalogRefreshCooldown() > 0
	) {
		return externalAuthLibraryCatalog
	}

	const providers = await loadExternalAuthProviders()
	externalAuthLibraryCatalog = await Promise.all(
		providers.map(fetchExternalAuthLibraryCatalogEntry),
	)
	externalAuthLibraryCatalogNextRefreshAt = Date.now() + externalAuthLibraryCatalogRefreshCooldownMs

	return externalAuthLibraryCatalog
}

/** Returns the delay before the remote provider-library catalog can be refreshed again. */
export function getExternalAuthLibraryCatalogRefreshCooldown(): number {
	return Math.max(0, externalAuthLibraryCatalogNextRefreshAt - Date.now())
}

/** Returns persisted selections and locally available provider libraries. */
export async function getExternalAuthLibraryStates(): Promise<ExternalAuthLibraryState[]> {
	return await invoke('plugin:auth|get_external_auth_library_states')
}

/** Installs and selects an exact provider library asset. */
export async function installExternalAuthLibrary(
	provider: string,
	assetName: string,
): Promise<void> {
	await invoke('plugin:auth|install_external_auth_library', { provider, assetName })
}

/** Selects an already-downloaded provider library asset. */
export async function selectExternalAuthLibrary(
	provider: string,
	assetName: string,
): Promise<boolean> {
	return await invoke('plugin:auth|select_external_auth_library', { provider, assetName })
}

/** Finds the UI metadata for a stored external account type. */
export function getExternalAuthProvider(accountType?: string) {
	return externalAuthProviders.value.find((provider) => provider.id === accountType)
}

/** Starts the native OAuth flow for one external authentication provider. */
async function authenticateExternalProvider(provider: string): Promise<MinecraftCredential | null> {
	return await invoke('plugin:auth|authenticate_external_provider', { provider })
}

/** Coordinates an external sign-in attempt and exposes its shared loading state. */
export function useExternalAuthentication(options: ExternalAuthenticationOptions) {
	const disabled = ref(false)

	/** Authenticates the selected provider while preventing parallel sign-in windows. */
	async function authenticate(selectedProvider: ExternalAuthProvider) {
		if (disabled.value) {
			return
		}

		disabled.value = true

		try {
			const credentials = await authenticateExternalProvider(selectedProvider.id)
			if (credentials) {
				await options.onAuthenticated(credentials)
			}
		} catch (error) {
			options.onError(error)
		} finally {
			disabled.value = false
		}
	}

	return {
		authenticate,
		disabled,
	}
}
