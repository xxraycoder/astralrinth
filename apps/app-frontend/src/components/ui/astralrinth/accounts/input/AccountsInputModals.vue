<script setup lang="ts">
import { MicrosoftIcon, OfflineIcon, SpinnerIcon } from '@modrinth/assets'
import { Button, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import type { ExternalAuthProvider } from '@/models/astralrinth/authentication'

type ModalHandle = {
	hide: () => void
	show: () => void
}

const props = defineProps<{
	offlineLoginDisabled: boolean
	offlinePlayerName: string
	loginDisabled: boolean
	externalAuthDisabled: boolean
	externalAuthProviders: ExternalAuthProvider[]
}>()

const emit = defineEmits<{
	(event: 'submit-offline'): void
	(event: 'update:offlinePlayerName', value: string): void
	(event: 'login-microsoft'): void
	(event: 'login-external', provider: ExternalAuthProvider): void
}>()

const { formatMessage } = useVIntl()
const addOfflineModal = ref<ModalHandle | null>(null)
const authMethodsModal = ref<ModalHandle | null>(null)

const messages = defineMessages({
	loginAction: {
		id: 'astralrinth.app.minecraft-account.input.login-action',
		defaultMessage: 'Login',
	},
	addOfflineHeader: {
		id: 'astralrinth.app.minecraft-account.input.offline.header',
		defaultMessage: 'Add new offline account',
	},
	addAccountHeader: {
		id: 'astralrinth.app.minecraft-account.input.auth.header',
		defaultMessage: 'Add a Minecraft account',
	},
	chooseSignInMethod: {
		id: 'astralrinth.app.minecraft-account.input.auth.description',
		defaultMessage: 'Choose how you want to sign in or play.',
	},
	microsoftAccountDescription: {
		id: 'astralrinth.app.minecraft-account.input.auth.microsoft-description',
		defaultMessage: 'Official account with Minecraft: Java Edition.',
	},
	externalAccountDescription: {
		id: 'astralrinth.app.minecraft-account.input.auth.external-description',
		defaultMessage: 'Account for servers that support {providerName}.',
	},
	offlineAccount: {
		id: 'astralrinth.app.minecraft-account.input.auth.offline-label',
		defaultMessage: 'Offline',
	},
	offlineAccountDescription: {
		id: 'astralrinth.app.minecraft-account.input.auth.offline-description',
		defaultMessage: 'Local profile without authentication tokens.',
	},
	offlineNameLabel: {
		id: 'astralrinth.app.minecraft-account.input.offline.name.label',
		defaultMessage: 'Enter your player name',
	},
	offlineNamePlaceholder: {
		id: 'astralrinth.app.minecraft-account.input.offline.name.placeholder',
		defaultMessage: 'Your player name here...',
	},
})

function showOfflineFromAuth() {
	authMethodsModal.value?.hide()
	addOfflineModal.value?.show()
}

defineExpose({
	hideAuth: () => authMethodsModal.value?.hide(),
	hideOffline: () => addOfflineModal.value?.hide(),
	showAuth: () => authMethodsModal.value?.show(),
	showOffline: () => addOfflineModal.value?.show(),
})
</script>

<template>
	<NewModal
		ref="authMethodsModal"
		:header="formatMessage(messages.addAccountHeader)"
		width="560px"
		max-width="560px"
		no-padding
	>
		<div class="flex flex-col gap-3 p-6">
			<p class="m-0 mb-1 text-base text-secondary">
				{{ formatMessage(messages.chooseSignInMethod) }}
			</p>

			<button
				type="button"
				class="auth-method"
				:disabled="props.loginDisabled"
				@click="emit('login-microsoft')"
			>
				<span class="auth-method-icon">
					<MicrosoftIcon v-if="!props.loginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
				</span>
				<span class="auth-method-copy">
					<strong>Microsoft</strong>
					<span>{{ formatMessage(messages.microsoftAccountDescription) }}</span>
				</span>
			</button>

			<button
				v-for="provider in props.externalAuthProviders"
				:key="provider.id"
				type="button"
				class="auth-method"
				:disabled="props.externalAuthDisabled"
				@click="emit('login-external', provider)"
			>
				<span class="auth-method-icon">
					<component :is="provider.icon" v-if="!props.externalAuthDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
				</span>
				<span class="auth-method-copy">
					<strong>{{ provider.name }}</strong>
					<span>
						{{
							formatMessage(messages.externalAccountDescription, {
								providerName: provider.name,
							})
						}}
					</span>
				</span>
			</button>

			<button
				type="button"
				class="auth-method"
				:disabled="props.offlineLoginDisabled"
				@click="showOfflineFromAuth"
			>
				<span class="auth-method-icon auth-method-icon-offline">
					<OfflineIcon v-if="!props.offlineLoginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
				</span>
				<span class="auth-method-copy">
					<strong>{{ formatMessage(messages.offlineAccount) }}</strong>
					<span>{{ formatMessage(messages.offlineAccountDescription) }}</span>
				</span>
			</button>
		</div>
	</NewModal>

	<NewModal
		ref="addOfflineModal"
		:header="formatMessage(messages.addOfflineHeader)"
		max-width="500px"
	>
		<div class="flex flex-col gap-4 px-6 py-5">
			<label class="label form-label">{{ formatMessage(messages.offlineNameLabel) }}</label>
			<input
				:value="props.offlinePlayerName"
				type="text"
				:placeholder="formatMessage(messages.offlineNamePlaceholder)"
				class="input soft-input"
				@input="emit('update:offlinePlayerName', ($event.target as HTMLInputElement).value)"
			/>
			<div class="mt-6 ml-auto">
				<Button
					type="colored"
					color="brand"
					:disabled="props.offlineLoginDisabled"
					@click="emit('submit-offline')"
				>
					{{ formatMessage(messages.loginAction) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<style scoped lang="scss">
@import '../../../../../../../../packages/assets/styles/astralrinth/soft-inputs.scss';

.auth-method {
	@apply flex w-full cursor-pointer items-center gap-4 rounded-xl border border-solid border-surface-5 bg-button-bg px-4 py-3 text-left text-primary transition-colors;

	&:hover:not(:disabled) {
		@apply border-surface-4 bg-button-bgHover;
	}

	&:focus-visible {
		@apply outline-none ring-4 ring-brand-shadow;
	}

	&:disabled {
		@apply cursor-not-allowed opacity-60;
	}
}

.auth-method-icon {
	@apply flex size-10 shrink-0 items-center justify-center;

	:deep(svg) {
		@apply size-8;
	}
}

.auth-method-icon-offline {
	@apply text-secondary;

	:deep(path) {
		fill: currentColor;
	}
}

.auth-method-copy {
	@apply flex min-w-0 flex-col gap-1;

	strong {
		@apply text-base font-semibold leading-tight text-contrast;
	}

	span {
		@apply text-sm leading-snug text-secondary;
	}
}
</style>
