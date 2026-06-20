import { getVersion } from '@tauri-apps/api/app'
import { ref } from 'vue'

import { getOS, initUpdateLauncher, isDev } from '@/helpers/utils.js'

export type LauncherReleaseAsset = {
	name: string
	browser_download_url: string
}

export type LauncherRelease = {
	tag_name: string
	name: string
	assets: LauncherReleaseAsset[]
}

// import.meta.env uses `vite.config.ts`
// Environments can be configured in `packages/app-lib/` directory.
export const LAUNCHER_REPOSITORY_URL = `${import.meta.env.GIT_ASTRALIUM_URL}didirus/AstralRinth/`
export const LAUNCHER_RELEASES_URL = `${LAUNCHER_REPOSITORY_URL}releases`
const LAUNCHER_LATEST_RELEASE_API = `${import.meta.env.GIT_ASTRALIUM_API_URL}repos/didirus/AstralRinth/releases/latest`

export const isUpdateInstalling = ref(false)
export const isUpdateAvailable = ref(false)
export const latestLauncherRelease = ref<LauncherRelease | null>(null)

const currentOS = ref('')

const systems = ['macos', 'windows', 'linux'] as const
const osExtensions = {
	"linux": ['.deb'],
	"macos": ['.dmg', '.pkg', '.app'],
	"windows": ['.exe', '.msi']
}

const isDeveloper = await isDev()

const blacklistBeginPrefixes = [
	'dev',
	'nightly',
	'dirty',
	'dirty-dev',
	'dirty-nightly',
	'dirty_dev',
	'dirty_nightly',
]

export async function fetchRemote(): Promise<void> {
	currentOS.value = (await getOS()).toLowerCase()

	try {
		const response = await fetch(LAUNCHER_LATEST_RELEASE_API)
		if (!response.ok) {
			throw new Error(String(response.status))
		}

		const remoteData = (await response.json()) as LauncherRelease
		latestLauncherRelease.value = remoteData

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
		latestLauncherRelease.value = null
		isUpdateAvailable.value = false
		isUpdateInstalling.value = false
	}
}

export async function downloadLatestRelease(): Promise<boolean> {
	if (!latestLauncherRelease.value) {
		return false
	}

	if (!currentOS.value) {
		currentOS.value = (await getOS()).toLowerCase()
	}

	const installer = getInstaller(resolveOperationalSystemExtension(), latestLauncherRelease.value.assets)
	if (isDeveloper) {
		console.debug(installer)
	}
	if (!installer) {
		isUpdateInstalling.value = false
		return false
	}

	try {
		isUpdateInstalling.value = true
		return await initUpdateLauncher(
			installer.browser_download_url,
			installer.name,
			currentOS.value,
			true,
		)
	} finally {
		isUpdateInstalling.value = false
	}
}

function getInstaller(
	osExtensions: string[],
	builds: LauncherReleaseAsset[],
): LauncherReleaseAsset | null {
	for (const build of builds) {
		if (blacklistBeginPrefixes.some((prefix) => build.name.startsWith(prefix))) {
			continue
		}

		if (osExtensions.some((extension) => build.name.endsWith(extension))) {
			if (isDeveloper) {
				console.debug(build.name, build.browser_download_url)
			}
			return build
		}
	}

	return null
}

function resolveOperationalSystemExtension(): string[] {
	if (currentOS.value === 'macos') {
		return osExtensions["macos"]
	}

	if (currentOS.value === 'linux') {
		return osExtensions["linux"]
	}

	return osExtensions["windows"]
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
