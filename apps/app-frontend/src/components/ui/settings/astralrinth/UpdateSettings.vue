<script setup lang="ts">
import { ChartIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { isTauri } from '@tauri-apps/api/core'
import { arch, platform } from '@tauri-apps/plugin-os'
import { computed } from 'vue'

import AstralRinthSettingsPage from '@/components/ui/settings/astralrinth/AstralRinthSettingsPage.vue'
import {
	latestLauncherRelease,
	LAUNCHER_RELEASE_API,
} from '@/helpers/astralrinth/update'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	pageTitle: {
		id: 'astralrinth.app.settings.updates.title',
		defaultMessage: 'Updates',
	},
	pageDescription: {
		id: 'astralrinth.app.settings.updates.description',
		defaultMessage:
			'Inspect the AstralRinth release channel, distribution status, and update-provider diagnostics.',
	},
	analyticsTitle: {
		id: 'astralrinth.app.settings.updates.analytics.title',
		defaultMessage: 'Analytics',
	},
	analyticsDescription: {
		id: 'astralrinth.app.settings.updates.analytics.description',
		defaultMessage: 'Release and distribution statistics from the update provider.',
	},
	operatingSystem: {
		id: 'astralrinth.app.settings.updates.os',
		defaultMessage: 'Operating system',
	},
	architecture: {
		id: 'astralrinth.app.settings.updates.arch',
		defaultMessage: 'Architecture',
	},
	systemInformationUnavailable: {
		id: 'astralrinth.app.settings.updates.system-information-unavailable',
		defaultMessage: 'System information is unavailable.',
	},
	latestReleaseTag: {
		id: 'astralrinth.app.settings.updates.latest-release-tag',
		defaultMessage: 'Latest release tag',
	},
	latestUpdateTitle: {
		id: 'astralrinth.app.settings.updates.latest-update-title',
		defaultMessage: 'Latest update title',
	},
	downloadableFiles: {
		id: 'astralrinth.app.settings.updates.downloadable-files',
		defaultMessage: 'Downloadable files',
	},
	totalDownloads: {
		id: 'astralrinth.app.settings.updates.total-downloads',
		defaultMessage: 'Total product downloads',
	},
	httpStatus: {
		id: 'astralrinth.app.settings.updates.http-status',
		defaultMessage: 'HTTP status',
	},
	apiUrl: {
		id: 'astralrinth.app.settings.updates.api-url',
		defaultMessage: 'Update provider API URL',
	},
	noUpdateInformation: {
		id: 'astralrinth.app.settings.updates.no-update-information',
		defaultMessage: 'No update information is available.',
	},
})

const operatingSystem = isTauri() ? platform() : null
const architecture = isTauri() ? arch() : null

function formatReleaseValue(value: string | number): string {
	return latestLauncherRelease.value.data
		? String(value)
		: formatMessage(messages.noUpdateInformation)
}

const latestReleaseTag = computed(() =>
	formatReleaseValue(latestLauncherRelease.value.data?.tag_name ?? ''),
)
const latestUpdateTitle = computed(() =>
	formatReleaseValue(latestLauncherRelease.value.data?.name ?? ''),
)
const latestReleaseAssets = computed(() =>
	Object.values(latestLauncherRelease.value.data?.assets ?? {}).flatMap((assets) =>
		Object.values(assets).flat(),
	),
)
const downloadableFiles = computed(() => formatReleaseValue(latestReleaseAssets.value.length))
const totalDownloads = computed(() =>
	formatReleaseValue(latestLauncherRelease.value.data?.total_downloads ?? 0),
)
const httpStatus = computed(() => {
	return (
		latestLauncherRelease.value.httpStatus?.toString() ?? formatMessage(messages.noUpdateInformation)
	)
})
</script>

<template>
	<AstralRinthSettingsPage
		:title="formatMessage(messages.pageTitle)"
		:description="formatMessage(messages.pageDescription)"
	>
		<section
			class="glass-surface rounded-2xl border border-solid border-surface-5 bg-surface-3 p-5"
		>
			<div class="flex items-start gap-3">
				<div
					class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-brand-highlight text-brand"
				>
					<ChartIcon class="size-5" />
				</div>
				<div>
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.analyticsTitle) }}
					</h2>
					<p class="m-0 mt-1 text-sm text-secondary">
						{{ formatMessage(messages.analyticsDescription) }}
					</p>
				</div>
			</div>

			<dl class="m-0 mt-4 grid grid-cols-2 gap-3">
				<div
					class="glass-surface glass-surface--subtle rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.operatingSystem) }}
					</dt>
					<dd class="m-0 mt-1 break-words text-base font-semibold text-contrast">
						{{ operatingSystem ?? formatMessage(messages.systemInformationUnavailable) }}
					</dd>
				</div>

				<div
					class="glass-surface glass-surface--subtle rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.architecture) }}
					</dt>
					<dd class="m-0 mt-1 break-words text-base font-semibold text-contrast">
						{{ architecture ?? formatMessage(messages.systemInformationUnavailable) }}
					</dd>
				</div>

				<div
					class="glass-surface glass-surface--subtle col-span-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.latestReleaseTag) }}
					</dt>
					<dd class="m-0 mt-1 text-base font-semibold text-contrast">{{ latestReleaseTag }}</dd>
				</div>

				<div
					class="glass-surface glass-surface--subtle col-span-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.latestUpdateTitle) }}
					</dt>
					<dd class="m-0 mt-1 break-words text-base font-semibold text-contrast">
						{{ latestUpdateTitle }}
					</dd>
				</div>

				<div
					class="glass-surface glass-surface--subtle rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.downloadableFiles) }}
					</dt>
					<dd class="m-0 mt-1 text-xl font-bold text-contrast">{{ downloadableFiles }}</dd>
				</div>

				<div
					class="glass-surface glass-surface--subtle rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.totalDownloads) }}
					</dt>
					<dd class="m-0 mt-1 text-xl font-bold text-contrast">{{ totalDownloads }}</dd>
				</div>

				<div
					class="glass-surface glass-surface--subtle col-span-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.httpStatus) }}
					</dt>
					<dd class="m-0 mt-1 text-xl font-bold text-contrast">{{ httpStatus }}</dd>
				</div>

				<div
					class="glass-surface glass-surface--subtle col-span-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.apiUrl) }}
					</dt>
					<dd class="m-0 mt-1">
						<a
							class="break-all text-link hover:underline focus-visible:underline"
							:href="LAUNCHER_RELEASE_API"
							target="_blank"
							rel="noopener noreferrer"
						>
							{{ LAUNCHER_RELEASE_API }}
						</a>
					</dd>
				</div>
			</dl>
		</section>
	</AstralRinthSettingsPage>
</template>
