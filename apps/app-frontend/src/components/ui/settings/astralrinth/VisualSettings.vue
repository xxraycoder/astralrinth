<script setup lang="ts">
import { PaletteIcon } from '@modrinth/assets'
import {
	commonMessages,
	defineMessages,
	injectNotificationManager,
	Slider,
	Toggle,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { computed, inject, onBeforeUnmount, onMounted } from 'vue'

import AstralRinthSettingsPage from '@/components/ui/settings/astralrinth/AstralRinthSettingsPage.vue'
import { GLASS_LEVEL_OPTIONS, type GlassLevel, useTheme } from '@/composables/use-theme.ts'
import { get, set } from '@/helpers/settings.ts'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const theme = useTheme()
const settingsModal = inject(appSettingsModalContextKey, null)
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const messages = defineMessages({
	pageTitle: {
		id: 'astralrinth.app.settings.visual.title',
		defaultMessage: 'Visual',
	},
	pageDescription: {
		id: 'astralrinth.app.settings.visual.description',
		defaultMessage:
			'Customize the visual effects used by AstralRinth without changing the selected theme.',
	},
	liquidGlassTitle: {
		id: 'astralrinth.app.settings.visual.liquid-glass.title',
		defaultMessage: 'Liquid Glass',
	},
	liquidGlassDescription: {
		id: 'astralrinth.app.settings.visual.liquid-glass.description',
		defaultMessage:
			'Use translucent surfaces, blur, and layered depth across the app. Turn this off if the effects impact performance or readability.',
	},
	glassLevelTitle: {
		id: 'astralrinth.app.settings.visual.glass-level.title',
		defaultMessage: 'Glass level',
	},
	matte: {
		id: 'astralrinth.app.settings.visual.glass-level.matte',
		defaultMessage: 'Matte',
	},
	standard: {
		id: 'astralrinth.app.settings.visual.glass-level.standard',
		defaultMessage: 'Standard',
	},
	transparent: {
		id: 'astralrinth.app.settings.visual.glass-level.transparent',
		defaultMessage: 'Transparent',
	},
	note: {
		id: 'astralrinth.app.settings.visual.note',
		defaultMessage:
			'Liquid Glass is an AstralRinth visual override. Your selected color theme and all app interactions remain unchanged.',
	},
})

type VisualSettingsState = {
	liquidGlass: boolean
	glassLevel: GlassLevel
}

function getVisualSettingsState(): VisualSettingsState {
	return { liquidGlass: theme.advancedRendering, glassLevel: theme.glassLevel }
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	getVisualSettingsState,
	async () => {
		const { liquidGlass, glassLevel } = current.value
		const settings = await get()
		settings.advanced_rendering = liquidGlass
		await set(settings)
		theme.advancedRendering = liquidGlass
		theme.glassLevel = glassLevel
	},
)

const glassLevelPosition = computed({
	get: () => GLASS_LEVEL_OPTIONS.indexOf(current.value.glassLevel),
	set: (position: number) => {
		current.value.glassLevel = GLASS_LEVEL_OPTIONS[position] ?? 'standard'
	},
})

const glassLevelLabel = computed(
	() =>
		`${formatMessage(messages.glassLevelTitle)}: ${formatMessage(messages[current.value.glassLevel])}`,
)

async function saveVisualSettings(): Promise<void> {
	try {
		await save()
	} catch (error) {
		handleError(error)
	}
}

onMounted(() => {
	settingsModal?.registerUnsavedChangesController({
		hasChanges: () => hasChanges.value,
		getOriginal: () => saved.value,
		getModified: () => changes.value,
		isSaving: () => saving.value,
		reset,
		save: saveVisualSettings,
	})
})

onBeforeUnmount(() => {
	settingsModal?.registerUnsavedChangesController(null)
})
</script>

<template>
	<AstralRinthSettingsPage
		:title="formatMessage(messages.pageTitle)"
		:description="formatMessage(messages.pageDescription)"
	>
		<section
			class="glass-surface glass-surface--strong rounded-2xl border border-solid border-surface-5 bg-surface-3 p-5"
		>
			<div class="flex items-start gap-3">
				<div
					class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-brand-highlight text-brand"
				>
					<PaletteIcon class="size-5" />
				</div>
				<div>
					<div class="flex flex-wrap items-center gap-2">
						<h2 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.liquidGlassTitle) }}
						</h2>
						<span
							class="shrink-0 rounded-full bg-brand-highlight px-1.5 py-0.5 text-xs font-bold text-brand-green"
						>
							{{ formatMessage(commonMessages.beta) }}
						</span>
					</div>
					<p class="m-0 mt-1 text-sm text-secondary">
						{{ formatMessage(messages.liquidGlassDescription) }}
					</p>
				</div>
			</div>

			<div class="mt-5 flex items-center justify-between gap-4">
				<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.note) }}</p>
				<Toggle
					id="astralrinth-liquid-glass"
					:model-value="current.liquidGlass"
					:disabled="saving"
					:aria-label="formatMessage(messages.liquidGlassTitle)"
					@update:model-value="current.liquidGlass = $event ?? false"
				/>
			</div>
			<div class="mt-5 flex flex-col gap-2">
				<span class="text-sm font-semibold text-contrast">
					{{ formatMessage(messages.glassLevelTitle) }}
				</span>
				<Slider
					v-model="glassLevelPosition"
					class="glass-level-slider"
					:min="0"
					:max="GLASS_LEVEL_OPTIONS.length - 1"
					:step="1"
					:disabled="!current.liquidGlass || saving"
					:aria-label="glassLevelLabel"
				/>
				<div class="grid grid-cols-3 gap-2 text-sm text-secondary">
					<span
						v-for="(level, index) in GLASS_LEVEL_OPTIONS"
						:key="level"
						:class="{
							'font-semibold text-contrast': current.glassLevel === level,
							'text-center': index === 1,
							'text-right': index === 2,
						}"
					>
						{{ formatMessage(messages[level]) }}
					</span>
				</div>
			</div>
		</section>
	</AstralRinthSettingsPage>
</template>

<style lang="scss" scoped>
.glass-level-slider.glass-level-slider {
	:deep(> span),
	:deep(.slider-value) {
		display: none;
	}

	:deep(.slider) {
		background: transparent !important;
		border: 0;
		box-shadow: none;
		-webkit-backdrop-filter: none;
		backdrop-filter: none;

		&::-webkit-slider-thumb {
			background: transparent;
			border: 0;
			box-shadow: none;
			-webkit-backdrop-filter: none;
			backdrop-filter: none;
		}

		&::-moz-range-thumb {
			background: transparent;
			border: 0;
			box-shadow: none;
			-webkit-backdrop-filter: none;
			backdrop-filter: none;
		}
	}

	:deep(.slider-track) {
		height: 4px;
		border: 0;
		background: var(--surface-5);
		box-shadow: none;
		-webkit-backdrop-filter: none;
		backdrop-filter: none;
	}

	:deep(.filled-slider-track) {
		background: var(--color-brand);
		box-shadow: none;
	}

	:deep(.slider-thumb) {
		top: -6px;
		right: -8px;
		width: 16px;
		height: 16px;
		border: 0;
		background: var(--color-brand);
		box-shadow: none;
		-webkit-backdrop-filter: none;
		backdrop-filter: none;
	}
}
</style>
