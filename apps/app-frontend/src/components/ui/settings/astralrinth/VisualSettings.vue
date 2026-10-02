<script setup lang="ts">
import { PaletteIcon } from '@modrinth/assets'
import { defineMessages, Toggle, useSavable, useVIntl } from '@modrinth/ui'
import { inject, onBeforeUnmount, onMounted } from 'vue'

import AstralRinthSettingsPage from '@/components/ui/settings/astralrinth/AstralRinthSettingsPage.vue'
import { useTheme } from '@/composables/use-theme.ts'
import { get, set } from '@/helpers/settings.ts'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const theme = useTheme()
const settingsModal = inject(appSettingsModalContextKey, null)
const { formatMessage } = useVIntl()

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
	note: {
		id: 'astralrinth.app.settings.visual.note',
		defaultMessage:
			'Liquid Glass is an AstralRinth visual override. Your selected color theme and all app interactions remain unchanged.',
	},
})

type VisualSettingsState = {
	liquidGlass: boolean
}

function getVisualSettingsState(): VisualSettingsState {
	return { liquidGlass: theme.advancedRendering }
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	getVisualSettingsState,
	async () => {
		const settings = await get()
		settings.advanced_rendering = current.value.liquidGlass
		await set(settings)
		theme.advancedRendering = current.value.liquidGlass
	},
)

async function saveVisualSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
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
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.liquidGlassTitle) }}
					</h2>
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
					:aria-label="formatMessage(messages.liquidGlassTitle)"
					@update:model-value="current.liquidGlass = $event ?? false"
				/>
			</div>
		</section>
	</AstralRinthSettingsPage>
</template>
