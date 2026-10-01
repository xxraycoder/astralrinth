import { getVersion } from '@tauri-apps/api/app'
import { isTauri } from '@tauri-apps/api/core'
import { arch } from '@tauri-apps/plugin-os'
import { fetch } from '@tauri-apps/plugin-http'
import { ref } from 'vue'

import { getOS, initUpdateLauncher, isDev } from '@/helpers/utils.js'

const systems = ['macos', 'windows', 'linux'] as const

type LauncherOperatingSystem = (typeof systems)[number]
type LauncherArchitecture = 'arm64' | 'amd64'

type LauncherReleaseAsset = {
	name: string
	browser_download_url: string
	download_count: number
}

type LauncherRelease = {
	tag_name: string
	name: string
	total_downloads: number
	assets: Record<LauncherArchitecture, Record<LauncherOperatingSystem, LauncherReleaseAsset[]>>
}

// import.meta.env uses `vite.config.ts`
// Environments can be configured in `packages/app-lib/` directory.
export const LAUNCHER_REPOSITORY_URL = `${import.meta.env.XORISON_REPO_URL}didirus/AstralRinth/`
export const LAUNCHER_RELEASE_API = `${import.meta.env.XORISON_API_URL}public/product/astralrinth`


export const isUpdateInstalling = ref(false)
export const isUpdateAvailable = ref(false)
export const latestLauncherReleases = ref<LauncherRelease | null>(null)
export const latestLauncherReleaseHttpStatus = ref<number | null>(null)

const currentOS = ref('')

const isDeveloper = isTauri() && (await isDev())

const blacklistBeginPrefixes = ['dev', 'nightly']

export async function fetchRemote(): Promise<void> {
	currentOS.value = (await getOS()).toLowerCase()
	latestLauncherReleaseHttpStatus.value = null
	try {
		if (!currentOS.value) {
			throw new Error(String('Current OS is undefined'))
		}
		// Get latest AstralRinth release from API.
		const response = await fetch(LAUNCHER_RELEASE_API + '?version=latest')
		latestLauncherReleaseHttpStatus.value = response.status
		if (!response.ok) {
			throw new Error(String(response.status))
		}

		const remoteData = (await response.json()) as LauncherRelease
		latestLauncherReleases.value = remoteData

		if (systems.includes(currentOS.value as (typeof systems)[number])) {
			const rawLocalVersion = await getVersion()
			const localVersion = normalizeVersion(rawLocalVersion)
			const remoteVersion = normalizeVersion(remoteData.tag_name)
			const versionComparison = compareVersions(remoteVersion, localVersion)
			isUpdateAvailable.value = versionComparison > 0

			if (isDeveloper) {
				console.debug('Raw local version is', rawLocalVersion)
				console.debug('Normalized local version is', localVersion)
				console.debug('Raw remote version is', remoteData.tag_name)
				console.debug('Normalized remote version is', remoteVersion)
				console.debug('Local version parts are', parseVersionParts(localVersion))
				console.debug('Remote version parts are', parseVersionParts(remoteVersion))
				console.debug('Version comparison result is', versionComparison)
			}
		} else {
			isUpdateAvailable.value = false

			if (isDeveloper) {
				console.debug('Skipped update comparison for unsupported OS', currentOS.value)
			}
		}

		if (isDeveloper) {
			console.debug('Update available state is', isUpdateAvailable.value)
			console.debug('Remote version is', remoteData.tag_name)
			console.debug('Remote title is', remoteData.name)
			console.debug('Operating System is', currentOS.value)
		}
	} catch (error) {
		console.error('Failed to fetch remote releases:', error)
		latestLauncherReleases.value = null
		isUpdateAvailable.value = false
		isUpdateInstalling.value = false
	}
}

export async function downloadLatestRelease(
	selectedInstaller?: LauncherReleaseAsset | null,
): Promise<boolean> {
	if (!latestLauncherReleases.value) {
		return false
	}

	if (!currentOS.value) {
		currentOS.value = (await getOS()).toLowerCase()
	}

	const installer = selectedInstaller ?? null
	if (isDeveloper) {
		console.debug(installer)
	}
	if (!installer) {
		isUpdateInstalling.value = false
		return false
	}

	try {
		isUpdateInstalling.value = true
		return await initUpdateLauncher(installer.browser_download_url, installer.name, currentOS.value)
	} finally {
		isUpdateInstalling.value = false
	}
}

export function getAvailableInstallers(): LauncherReleaseAsset[] {
	if (!latestLauncherReleases.value) {
		return []
	}

	const architecture = resolveArchitecture(arch())
	const operatingSystem = currentOS.value as LauncherOperatingSystem
	if (!architecture || !systems.includes(operatingSystem)) {
		return []
	}

	const builds = latestLauncherReleases.value.assets[architecture]?.[operatingSystem]
	return getInstallers(builds ?? [])
}

function getInstallers(builds: LauncherReleaseAsset[]): LauncherReleaseAsset[] {
	return builds.filter((build) => {
		if (blacklistBeginPrefixes.some((prefix) => build.name.toLowerCase().startsWith(prefix))) {
			return false
		}

		if (isDeveloper) {
			console.debug(build.name, build.browser_download_url)
		}

		return true
	})
}

function resolveArchitecture(architecture: string): LauncherArchitecture | null {
	switch (architecture.toLowerCase()) {
		case 'x86_64':
		case 'x64':
		case 'amd64':
			return 'amd64'
		case 'aarch64':
		case 'arm64':
			return 'arm64'
		default:
			return null
	}
}

function normalizeVersion(version: string): string {
	return version.trim().replace(/^v/i, '')
}

function compareVersions(left: string, right: string): number {
	const leftParts = parseVersionParts(left)
	const rightParts = parseVersionParts(right)
	const maxLength = Math.max(leftParts.length, rightParts.length)

	for (let index = 0; index < maxLength; index += 1) {
		const leftPart = leftParts[index] ?? 0
		const rightPart = rightParts[index] ?? 0

		if (leftPart !== rightPart) {
			if (isDeveloper) {
				console.debug('Version parts differ at index', index, leftPart, rightPart)
			}
			return leftPart - rightPart
		}
	}

	if (isDeveloper) {
		console.debug('Version parts are equal', leftParts, rightParts)
	}

	return 0
}

function parseVersionParts(version: string): number[] {
	return normalizeVersion(version)
		.split(/[.-]/)
		.map((part) => Number.parseInt(part, 10))
		.filter((part) => !Number.isNaN(part))
}
