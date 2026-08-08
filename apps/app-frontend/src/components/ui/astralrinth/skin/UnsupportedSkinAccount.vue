<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { getExternalAuthProvider } from '@/models/astralrinth/authentication'

const props = defineProps<{
	accountType?: string
}>()

const messages = defineMessages({
	title: {
		id: 'astralrinth.app.skins.unsupported-account.title',
		defaultMessage: 'Microsoft account required',
	},
	description: {
		id: 'astralrinth.app.skins.unsupported-account.description',
		defaultMessage:
			'Skins can only be managed with a Microsoft account. Return home and select or add a Microsoft account to continue.',
	},
	returnHome: {
		id: 'astralrinth.app.skins.unsupported-account.return-home',
		defaultMessage: 'Return home',
	},
	externalTitle: {
		id: 'astralrinth.app.skins.unsupported-account.external.title',
		defaultMessage: 'Manage your {providerName} skin',
	},
	externalDescription: {
		id: 'astralrinth.app.skins.unsupported-account.external.description',
		defaultMessage:
			"You're signed in with a {providerName} account. You can upload, select, and manage your skin directly on the provider's website.",
	},
	externalAction: {
		id: 'astralrinth.app.skins.unsupported-account.external.action',
		defaultMessage: 'Manage skin on {providerName}',
	},
})

const router = useRouter()
const { formatMessage } = useVIntl()
const externalAuthProvider = computed(() => getExternalAuthProvider(props.accountType))

/** Returns from the unavailable skin flow to account selection. */
function returnHome() {
	void router.push({ path: '/' })
}

/** Opens the selected provider's own skin-management page. */
function openExternalProviderSkins() {
	if (externalAuthProvider.value) {
		void openUrl(externalAuthProvider.value.skinManagementUrl)
	}
}
</script>

<template>
	<div
		class="box-border flex min-h-full cursor-pointer items-center justify-center p-4 sm:p-8"
		role="alert"
		@click="returnHome"
	>
		<div
			class="grid w-full max-w-3xl overflow-hidden rounded-2xl border border-solid border-surface-5 bg-bg-raised shadow-xl"
			:class="{ 'md:grid-cols-[minmax(220px,0.8fr)_minmax(0,1.5fr)]': externalAuthProvider }"
		>
			<div
				v-if="externalAuthProvider"
				class="relative flex min-h-48 items-center justify-center overflow-hidden bg-surface-2 p-8 md:min-h-full"
			>
				<div class="absolute inset-0 bg-gradient-to-br from-bg-blue via-surface-2 to-bg-raised" />
				<div
					class="relative flex size-28 items-center justify-center rounded-3xl border border-solid border-surface-5 bg-bg-raised shadow-lg"
				>
					<component :is="externalAuthProvider.icon" class="size-20" aria-hidden="true" />
				</div>
			</div>

			<div class="flex min-w-0 flex-col gap-5 p-7 sm:p-9">
				<h1 class="m-0 text-3xl font-extrabold leading-tight">
					{{
						formatMessage(externalAuthProvider ? messages.externalTitle : messages.title, {
							providerName: externalAuthProvider?.name,
						})
					}}
				</h1>
				<p class="m-0 text-lg leading-relaxed text-secondary">
					{{
						formatMessage(
							externalAuthProvider ? messages.externalDescription : messages.description,
							{ providerName: externalAuthProvider?.name },
						)
					}}
				</p>
				<div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap">
					<Button
						v-if="externalAuthProvider"
						type="colored"
						color="brand"
						@click.stop="openExternalProviderSkins"
					>
						{{ formatMessage(messages.externalAction, { providerName: externalAuthProvider.name }) }}
						<ExternalIcon aria-hidden="true" />
					</Button>
					<Button
						:type="externalAuthProvider ? 'outlined' : 'colored'"
						color="brand"
						@click.stop="returnHome"
					>
						{{ formatMessage(messages.returnHome) }}
					</Button>
				</div>
			</div>
		</div>
	</div>
</template>
