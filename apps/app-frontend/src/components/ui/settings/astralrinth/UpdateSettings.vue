<script setup lang="ts">
import { ChartIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import AstralRinthSettingsPage from '@/components/ui/settings/astralrinth/AstralRinthSettingsPage.vue'
import {
	LAUNCHER_RELEASE_API,
	latestLauncherReleaseHttpStatus,
	latestLauncherReleases,
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

function formatReleaseValue(value: string | number): string {
	return latestLauncherReleases.value ? String(value) : formatMessage(messages.noUpdateInformation)
}

const latestReleaseTag = computed(() =>
	formatReleaseValue(latestLauncherReleases.value?.tag_name ?? ''),
)
const latestUpdateTitle = computed(() =>
	formatReleaseValue(latestLauncherReleases.value?.name ?? ''),
)
const latestReleaseAssets = computed(() =>
	Object.values(latestLauncherReleases.value?.assets ?? {}).flatMap((assets) =>
		Object.values(assets).flat(),
	),
)
const downloadableFiles = computed(() => formatReleaseValue(latestReleaseAssets.value.length))
const totalDownloads = computed(() =>
	formatReleaseValue(latestLauncherReleases.value?.total_downloads ?? 0),
)
const httpStatus = computed(() => {
	return (
		latestLauncherReleaseHttpStatus.value?.toString() ?? formatMessage(messages.noUpdateInformation)
	)
})
</script>

<template>
	<AstralRinthSettingsPage
		:title="formatMessage(messages.pageTitle)"
		:description="formatMessage(messages.pageDescription)"
	>
		<section
			class="rounded-xl border border-solid border-[rgba(62,140,222,0.3)] bg-[rgba(62,140,222,0.055)] p-1"
		>
			<div class="flex items-start gap-3">
				<div
					class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-[rgba(62,140,222,0.14)] text-brand"
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
					class="col-span-2 rounded-xl border border-solid border-[rgba(255,255,255,0.1)] bg-[rgba(0,0,0,0.14)] p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.latestReleaseTag) }}
					</dt>
					<dd class="m-0 mt-1 text-base font-semibold neon-text">{{ latestReleaseTag }}</dd>
				</div>

				<div
					class="col-span-2 rounded-xl border border-solid border-[rgba(255,255,255,0.1)] bg-[rgba(0,0,0,0.14)] p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.latestUpdateTitle) }}
					</dt>
					<dd class="m-0 mt-1 break-words text-base font-semibold neon-text">
						{{ latestUpdateTitle }}
					</dd>
				</div>

				<div
					class="rounded-xl border border-solid border-[rgba(255,255,255,0.1)] bg-[rgba(0,0,0,0.14)] p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.downloadableFiles) }}
					</dt>
					<dd class="m-0 mt-1 text-xl font-bold neon-text">{{ downloadableFiles }}</dd>
				</div>

				<div
					class="rounded-xl border border-solid border-[rgba(255,255,255,0.1)] bg-[rgba(0,0,0,0.14)] p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.totalDownloads) }}
					</dt>
					<dd class="m-0 mt-1 text-xl font-bold neon-text">{{ totalDownloads }}</dd>
				</div>

				<div
					class="col-span-2 rounded-xl border border-solid border-[rgba(255,255,255,0.1)] bg-[rgba(0,0,0,0.14)] p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.httpStatus) }}
					</dt>
					<dd class="m-0 mt-1 text-xl font-bold neon-text">{{ httpStatus }}</dd>
				</div>

				<div
					class="col-span-2 rounded-xl border border-solid border-[rgba(255,255,255,0.1)] bg-[rgba(0,0,0,0.14)] p-3"
				>
					<dt class="text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.apiUrl) }}
					</dt>
					<dd class="m-0 mt-1">
						<a
							class="break-all neon-text"
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

<style lang="scss" scoped>
@import '../../../../../../../packages/assets/styles/astralrinth/neon-text.scss';
</style>
