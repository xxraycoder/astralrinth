import { fetch } from '@tauri-apps/plugin-http'
import { arch } from '@tauri-apps/plugin-os'
import { ref } from 'vue'

import { getOS, initUpdateLauncher } from '@/helpers/utils.js'

declare const __ASTRALRINTH_APP_BUILD__: string
declare const __ASTRALRINTH_APP_COMMIT__: string

const systems = ['macos', 'windows', 'linux'] as const

type LauncherOperatingSystem = (typeof systems)[number]
type LauncherArchitecture = 'arm64' | 'amd64'

type LauncherReleaseAsset = {
	name: string
	browser_download_url: string
	download_count: number
}

type LauncherReleaseTag = {
	name: string
	message: string
	id: string
	commit: {
		url: string
		sha: string
		created: string
	}
	zipball_url: string
	tarball_url: string
	archive_download_count: {
		zip: number
		tar_gz: number
	}
}

type LauncherRelease = {
	tag_name?: string
	name?: string
	total_downloads: number
	tag: LauncherReleaseTag
	assets: Record<LauncherArchitecture, Record<LauncherOperatingSystem, LauncherReleaseAsset[]>>
}

// import.meta.env uses `vite.config.ts`
// Environments can be configured in `packages/app-lib/` directory.
export const LAUNCHER_REPOSITORY_URL = `${import.meta.env.XORISON_GIT_URL}didirus/AstralRinth/`
export const LAUNCHER_RELEASE_API = `${import.meta.env.XORISON_API_URL}public/product/astralrinth`

export const isUpdateInstalling = ref(false)
export const isUpdateAvailable = ref(false)
export const latestLauncherRelease = ref<{
	data: LauncherRelease | null
	httpStatus: number | null
}>({
	data: null,
	httpStatus: null,
})
export const APP_BUILD = __ASTRALRINTH_APP_BUILD__
export const APP_COMMIT = __ASTRALRINTH_APP_COMMIT__

const currentOS = ref('')

const blacklistBeginPrefixes = ['dev', 'nightly']

export async function fetchRemote(): Promise<void> {
	currentOS.value = (await getOS()).toLowerCase()
	latestLauncherRelease.value.httpStatus = null
	try {
		if (!currentOS.value) {
			throw new Error(String('Current OS is undefined'))
		}
		// Get latest AstralRinth release from API.
		const response = await fetch(LAUNCHER_RELEASE_API + '?version=latest')
		latestLauncherRelease.value.httpStatus = response.status
		if (!response.ok) {
			throw new Error(String(response.status))
		}

		const remoteData = (await response.json()) as LauncherRelease
		latestLauncherRelease.value.data = remoteData

		if (systems.includes(currentOS.value as (typeof systems)[number])) {
			const remoteCommit = remoteData.tag.commit.sha.toLowerCase()
			const localCommitHash = APP_COMMIT.toLowerCase()
			isUpdateAvailable.value =
				localCommitHash !== 'unknown' &&
				remoteCommit.length > 0 &&
				!remoteCommit.startsWith(localCommitHash)
		} else {
			isUpdateAvailable.value = false
		}
	} catch (error) {
		console.error('Failed to fetch remote releases:', error)
		latestLauncherRelease.value.data = null
		isUpdateAvailable.value = false
		isUpdateInstalling.value = false
	}
}

export async function downloadLatestRelease(
	selectedInstaller?: LauncherReleaseAsset | null,
): Promise<boolean> {
	if (!latestLauncherRelease.value.data) {
		return false
	}

	if (!currentOS.value) {
		currentOS.value = (await getOS()).toLowerCase()
	}

	const installer = selectedInstaller ?? null
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
	if (!latestLauncherRelease.value.data) {
		return []
	}

	const architecture = resolveArchitecture(arch())
	const operatingSystem = currentOS.value as LauncherOperatingSystem
	if (!architecture || !systems.includes(operatingSystem)) {
		return []
	}

	const builds = latestLauncherRelease.value.data.assets[architecture]?.[operatingSystem]
	return getInstallers(builds ?? [])
}

function getInstallers(builds: LauncherReleaseAsset[]): LauncherReleaseAsset[] {
	return builds.filter((build) => {
		if (blacklistBeginPrefixes.some((prefix) => build.name.toLowerCase().startsWith(prefix))) {
			return false
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
