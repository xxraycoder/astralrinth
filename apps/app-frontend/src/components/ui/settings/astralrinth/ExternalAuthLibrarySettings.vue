<script setup lang="ts">
import { CheckIcon, DownloadIcon, RefreshCwIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	DropdownSelect,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import AstralRinthSettingsPage from '@/components/ui/settings/astralrinth/AstralRinthSettingsPage.vue'
import {
	type ExternalAuthProvider,
	getExternalAuthLibraryCatalogRefreshCooldown,
	getExternalAuthLibraryStates,
	installExternalAuthLibrary,
	loadExternalAuthLibraryCatalog,
	selectExternalAuthLibrary,
} from '@/models/astralrinth/authentication'

type Library = {
	provider: ExternalAuthProvider
	assetNames: string[]
	localAssetNames: string[]
	selectedAssetName: string | null
	savedAssetName: string | null
	localOnly: boolean
	busy: 'selecting' | 'installing' | null
}

const messages = defineMessages({
	pageTitle: {
		id: 'astralrinth.app.settings.external-auth-libraries.title',
		defaultMessage: 'Authentication libraries',
	},
	pageDescription: {
		id: 'astralrinth.app.settings.external-auth-libraries.description',
		defaultMessage:
			'Choose and install the authentication library used when Minecraft starts with an external player profile.',
	},
	loading: {
		id: 'astralrinth.app.settings.external-auth-libraries.loading',
		defaultMessage: 'Loading available library versions...',
	},
	loadFailed: {
		id: 'astralrinth.app.settings.external-auth-libraries.load-failed',
		defaultMessage: 'Could not load authentication libraries: {error}',
	},
	retry: {
		id: 'astralrinth.app.settings.external-auth-libraries.retry',
		defaultMessage: 'Retry',
	},
	requestServer: {
		id: 'astralrinth.app.settings.external-auth-libraries.request-server',
		defaultMessage: 'Request from server',
	},
	requestingServer: {
		id: 'astralrinth.app.settings.external-auth-libraries.requesting-server',
		defaultMessage: 'Requesting...',
	},
	selectedVersion: {
		id: 'astralrinth.app.settings.external-auth-libraries.selected-version',
		defaultMessage: 'Selected version',
	},
	notSelected: {
		id: 'astralrinth.app.settings.external-auth-libraries.not-selected',
		defaultMessage: 'Not selected',
	},
	version: {
		id: 'astralrinth.app.settings.external-auth-libraries.version',
		defaultMessage: 'Version',
	},
	install: {
		id: 'astralrinth.app.settings.external-auth-libraries.install-selected',
		defaultMessage: 'Install',
	},
	reinstall: {
		id: 'astralrinth.app.settings.external-auth-libraries.reinstall',
		defaultMessage: 'Reinstall',
	},
	installing: {
		id: 'astralrinth.app.settings.external-auth-libraries.installing',
		defaultMessage: 'Installing...',
	},
	installed: {
		id: 'astralrinth.app.settings.external-auth-libraries.installed',
		defaultMessage: '{providerName} library {version} installed',
	},
	noVersions: {
		id: 'astralrinth.app.settings.external-auth-libraries.no-versions',
		defaultMessage: 'No compatible library versions were found.',
	},
	localOnly: {
		id: 'astralrinth.app.settings.external-auth-libraries.local-only',
		defaultMessage: 'Only locally installed versions are shown because the server is unavailable.',
	},
})

const { addNotification, handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const libraries = ref<Library[]>([])
const loading = ref(true)
const refreshing = ref(false)
const refreshLocked = ref(false)
const loadError = ref<string | null>(null)
const busy = computed(() => libraries.value.some((library) => library.busy !== null))
let refreshTimer: number | undefined

function getErrorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error)
}

function sortLibraryVersions(assetNames: string[]): string[] {
	return [...assetNames].sort(compareLibraryVersions)
}

function compareLibraryVersions(left: string, right: string): number {
	const leftVersion = left.match(/\d+(?:\.\d+)+/)?.[0].split('.').map(Number) ?? []
	const rightVersion = right.match(/\d+(?:\.\d+)+/)?.[0].split('.').map(Number) ?? []
	const length = Math.max(leftVersion.length, rightVersion.length)

	for (let index = 0; index < length; index++) {
		const difference = (rightVersion[index] ?? 0) - (leftVersion[index] ?? 0)
		if (difference !== 0) {
			return difference
		}
	}

	return right.localeCompare(left)
}

function updateRefreshLock(): void {
	if (refreshTimer !== undefined) {
		window.clearTimeout(refreshTimer)
	}

	const delay = getExternalAuthLibraryCatalogRefreshCooldown()
	refreshLocked.value = delay > 0
	refreshTimer = delay > 0 ? window.setTimeout(() => (refreshLocked.value = false), delay) : undefined
}

async function loadLibraries(forceRefresh = false): Promise<void> {
	if (!forceRefresh) {
		loading.value = true
	}
	refreshing.value = forceRefresh
	loadError.value = null

	try {
		const [catalog, states] = await Promise.all([
			loadExternalAuthLibraryCatalog(forceRefresh),
			getExternalAuthLibraryStates(),
		])
		const statesByProvider = new Map(states.map((state) => [state.providerId, state]))

		libraries.value = catalog.map(({ provider, assetNames }) => {
			const state = statesByProvider.get(provider.id)
			const localAssetNames = sortLibraryVersions(state?.localAssetNames ?? [])
			const availableAssetNames = assetNames ? sortLibraryVersions(assetNames) : localAssetNames
			const savedAssetName = state?.selectedAssetName ?? null

			return {
				provider,
				assetNames: availableAssetNames,
				localAssetNames,
				selectedAssetName: availableAssetNames.includes(savedAssetName ?? '')
					? savedAssetName
					: null,
				savedAssetName,
				localOnly: assetNames === null,
				busy: null,
			}
		})
	} catch (error) {
		loadError.value = getErrorMessage(error)
	} finally {
		loading.value = false
		refreshing.value = false
		updateRefreshLock()
	}
}

async function refreshLibraries(): Promise<void> {
	if (refreshLocked.value || refreshing.value || busy.value) {
		return
	}

	await loadLibraries(true)
}

async function selectLibrary(library: Library, assetName: string): Promise<void> {
	if (library.busy || assetName === library.savedAssetName) {
		return
	}

	library.busy = 'selecting'
	try {
		if (await selectExternalAuthLibrary(library.provider.id, assetName)) {
			library.savedAssetName = assetName
		}
	} catch (error) {
		handleError(new Error(getErrorMessage(error)))
	} finally {
		library.busy = null
	}
}

async function installLibrary(library: Library): Promise<void> {
	const assetName = library.selectedAssetName
	if (!assetName || library.busy) {
		return
	}

	library.busy = 'installing'
	try {
		await installExternalAuthLibrary(library.provider.id, assetName)
		if (!library.localAssetNames.includes(assetName)) {
			library.localAssetNames.push(assetName)
		}
		library.savedAssetName = assetName
		addNotification({
			type: 'success',
			title: formatMessage(messages.installed, {
				providerName: library.provider.name,
				version: assetName,
			}),
		})
	} catch (error) {
		handleError(new Error(getErrorMessage(error)))
	} finally {
		library.busy = null
	}
}

onMounted(() => void loadLibraries())
onUnmounted(() => {
	if (refreshTimer !== undefined) {
		window.clearTimeout(refreshTimer)
	}
})
</script>

<template>
	<AstralRinthSettingsPage
		:title="formatMessage(messages.pageTitle)"
		:description="formatMessage(messages.pageDescription)"
	>
		<template #actions>
			<Button
				type="outlined"
				:disabled="loading || refreshing || refreshLocked || busy"
				@click="refreshLibraries"
			>
				<SpinnerIcon v-if="refreshing" class="animate-spin" />
				<RefreshCwIcon v-else />
				{{ formatMessage(refreshing ? messages.requestingServer : messages.requestServer) }}
			</Button>
		</template>

		<div v-if="loading" class="flex items-center gap-2 text-secondary">
			<SpinnerIcon class="size-5 animate-spin" />
			{{ formatMessage(messages.loading) }}
		</div>

		<div v-else-if="loadError" class="flex flex-col items-start gap-3">
			<p class="m-0 text-red">
				{{ formatMessage(messages.loadFailed, { error: loadError }) }}
			</p>
			<Button type="outlined" @click="loadLibraries()">
				<RefreshCwIcon />
				{{ formatMessage(messages.retry) }}
			</Button>
		</div>

		<div v-else class="flex flex-col gap-4">
			<section
				v-for="library in libraries"
				:key="library.provider.id"
				class="rounded-xl border border-solid border-[rgba(62,140,222,0.3)] bg-[rgba(62,140,222,0.055)] p-4"
			>
				<div class="flex items-center gap-3">
					<div
						class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-[rgba(62,140,222,0.14)] text-brand"
					>
						<component :is="library.provider.icon" class="size-6" />
					</div>
					<div class="min-w-0">
						<h2 class="m-0 text-lg font-semibold text-contrast">
							{{ library.provider.name }}
						</h2>
						<p class="m-0 mt-1 text-sm text-secondary">
							{{ formatMessage(messages.selectedVersion) }}:
							<span v-if="library.savedAssetName" class="font-semibold text-contrast">
								{{ library.savedAssetName }}
							</span>
							<span v-else>{{ formatMessage(messages.notSelected) }}</span>
						</p>
					</div>
					<CheckIcon
						v-if="
							library.savedAssetName && library.localAssetNames.includes(library.savedAssetName)
						"
						class="ml-auto size-6 shrink-0 text-green"
					/>
				</div>

				<p v-if="library.localOnly" class="m-0 mt-4 text-secondary">
					{{ formatMessage(messages.localOnly) }}
				</p>

				<p v-if="library.assetNames.length === 0" class="m-0 mt-4 text-secondary">
					{{ formatMessage(messages.noVersions) }}
				</p>

				<div v-else class="mt-4 flex flex-wrap items-end gap-3">
					<div class="flex min-w-0 flex-1 flex-col gap-2 text-sm font-semibold text-contrast">
						<span>{{ formatMessage(messages.version) }}</span>
						<DropdownSelect
							v-model="library.selectedAssetName"
							:name="`external-auth-library-${library.provider.id}`"
							:options="library.assetNames"
							:disabled="library.busy || refreshing"
							@change="selectLibrary(library, $event.option)"
						/>
					</div>
					<Button
						type="colored"
						color="brand"
						:disabled="library.localOnly || !library.selectedAssetName || library.busy || refreshing"
						@click="installLibrary(library)"
					>
						<SpinnerIcon v-if="library.busy === 'installing'" class="animate-spin" />
						<DownloadIcon v-else />
						{{
							formatMessage(
								library.busy === 'installing'
									? messages.installing
									: library.localAssetNames.includes(library.selectedAssetName ?? '')
										? messages.reinstall
										: messages.install,
							)
						}}
					</Button>
				</div>
			</section>
		</div>
	</AstralRinthSettingsPage>
</template>
