<script setup lang="ts">
import {
	AstralRinthLogo,
	CoffeeIcon,
	DownloadIcon,
	GameIcon,
	GaugeIcon,
	LanguagesIcon,
	PaintbrushIcon,
	SettingsIcon,
	ShieldIcon,
	SpinnerIcon,
	ToggleRightIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	ProgressBar,
	TabbedModal,
	useVIntl,
} from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { ref, watch } from 'vue'

import LauncherUpdateModal from '@/components/ui/astralrinth/LauncherUpdateModal.vue'
import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/DefaultInstanceSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/FeatureFlagSettings.vue'
import JavaSettings from '@/components/ui/settings/JavaSettings.vue'
import LanguageSettings from '@/components/ui/settings/LanguageSettings.vue'
import PrivacySettings from '@/components/ui/settings/PrivacySettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/ResourceManagementSettings.vue'
import { get, set } from '@/helpers/settings.ts'
import { isUpdateInstalling, isUpdateAvailable } from '@/helpers/astralrinth/update'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'
import { useTheming } from '@/store/state'

const themeStore = useTheming()
const { formatMessage } = useVIntl()

const devModeCounter = ref(0)
const modal = ref<InstanceType<typeof TabbedModal> | null>(null)
const launcherUpdateModal = ref<InstanceType<typeof LauncherUpdateModal> | null>(null)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.language',
			defaultMessage: 'Language',
		}),
		icon: LanguagesIcon,
		content: LanguageSettings,
		badge: commonMessages.beta,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Privacy',
		}),
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Java installations',
		}),
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.default-instance-options',
			defaultMessage: 'Default instance options',
		}),
		icon: GameIcon,
		content: DefaultInstanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resource-management',
			defaultMessage: 'Resource management',
		}),
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		name: commonSettingsMessages.featureFlags,
		icon: ToggleRightIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
]

function show() {
	modal.value?.show()
}

function showUpdateModal() {
	modal.value?.show()
	void launcherUpdateModal.value?.show()
}

defineExpose({ show, showUpdateModal })

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

function devModeCount() {
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		themeStore.devMode = !themeStore.devMode
		settings.value.developer_mode = !!themeStore.devMode
		devModeCounter.value = 0

		if (!themeStore.devMode && tabs[modal.value!.selectedTab].developerOnly) {
			modal.value!.setTab(0)
		}
	}
}

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
	updateInstalling: {
		id: 'astralrinth.app.settings.update-installing',
		defaultMessage: 'Installing update...',
	},
	viewUpdateInfo: {
		id: 'astralrinth.app.settings.view-update-info',
		defaultMessage: 'View update info',
	},
})
</script>

<template>
	<TabbedModal ref="modal" :tabs="tabs.filter((t) => !t.developerOnly || themeStore.devMode)">
		<template #title>
			<span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
				<SettingsIcon /> Settings
			</span>
		</template>
		<template #footer>
			<div class="mt-auto text-secondary text-sm">
				<div class="mb-3">
					<template v-if="progress > 0 && progress < 1">
						<p class="m-0 mb-2">
							{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
						</p>
						<ProgressBar :progress="progress" />
					</template>
				</div>
				<p v-if="themeStore.devMode" class="text-brand font-semibold m-0 mb-2">
					{{ formatMessage(developerModeEnabled) }}
				</p>
				<div class="flex items-center gap-3">
					<button
						class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
						:class="{
							'text-brand': themeStore.devMode,
							'text-secondary': !themeStore.devMode,
						}"
						@click="devModeCount"
					>
						<AstralRinthLogo class="w-6 h-6" />
					</button>
					<div class="max-w-[200px]">
						<p class="m-0">AstralRinth App {{ version }}</p>
						<p class="m-0">
							<span v-if="osPlatform === 'macos'">macOS</span>
							<span v-else class="capitalize">{{ osPlatform }}</span>
							{{ osVersion }}
						</p>
					</div>
					<div
						v-if="isUpdateAvailable"
						class="w-8 h-8 cursor-pointer hover:brightness-75 neon-icon pulse shrink-0"
					>
						<template v-if="isUpdateInstalling">
							<SpinnerIcon
								class="size-6 animate-spin"
								v-tooltip.bottom="formatMessage(messages.updateInstalling)"
							/>
						</template>
						<template v-else>
							<DownloadIcon
								class="size-6"
								v-tooltip.bottom="formatMessage(messages.viewUpdateInfo)"
								@click="showUpdateModal()"
							/>
						</template>
					</div>
				</div>
			</div>
		</template>
	</TabbedModal>

	<LauncherUpdateModal ref="launcherUpdateModal" :version="version" />
</template>

<style lang="scss" scoped>
@import '../../../../../../packages/assets/styles/astralrinth/neon-icon.scss';
</style>
