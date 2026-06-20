<template>
	<div
		v-if="accounts.length === 0"
		class="flex flex-col gap-3 bg-button-bg border border-solid border-surface-5 rounded-xl p-3 mt-2"
	>
		<span>{{ formatMessage(messages.notSignedIn) }}</span>
		<ButtonStyled color="brand">
			<button color="primary" :disabled="loginDisabled" @click="login()">
				<MicrosoftIcon v-if="!loginDisabled" />
				<SpinnerIcon v-else class="animate-spin" />
				{{ formatMessage(messages.signInToMinecraft) }}
			</button>
		</ButtonStyled>
		<!-- BEGIN: This code block modified by AstralRinth -->
		<ButtonStyled class="w-full">
			<OverflowMenu class="w-full justify-between text-left" :options="additionalAccountOptions">
				<span class="inline-flex items-center gap-2">
					<PlusIcon />
					{{ formatMessage(messages.addAccount) }}
				</span>
				<DropdownIcon class="shrink-0" />
				<template #add_offline_account>
					<OfflineIcon />
					{{ formatMessage(messages.addOfflineAccount) }}
				</template>
				<template #add_elyby_account>
					<ElyByIcon v-if="!elyByLoginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
					{{ formatMessage(messages.addElyByAccount) }}
				</template>
			</OverflowMenu>
		</ButtonStyled>
		<!-- END: This code block modified by AstralRinth -->
	</div>
	<Accordion
		v-else
		class="w-full mt-2 bg-button-bg border border-solid border-surface-5 rounded-xl overflow-clip"
		button-class="button-base w-full bg-transparent px-3 py-2 border-0 cursor-pointer"
		:open-by-default="false"
	>
		<template #title>
			<div class="flex gap-2 w-full min-w-0">
				<Avatar
					size="36px"
					:src="
						selectedAccount
							? avatarUrl
							: 'https://launcher-files.modrinth.com/assets/steve_head.png'
					"
				/>
				<div class="flex flex-col items-start w-full min-w-0">
					<span class="truncate w-full text-left">
						<span class="inline-flex items-center gap-1 min-w-0">
							<component
								:is="getAccountType(selectedAccount)"
								v-if="selectedAccount && getAccountType(selectedAccount)"
								class="vector-icon shrink-0"
							/>
							<span class="truncate">
								{{
									selectedAccount
										? selectedAccount.profile.name
										: formatMessage(messages.selectAccount)
								}}
							</span>
						</span>
					</span>
					<span class="text-secondary text-xs">{{ formatMessage(messages.minecraftAccount) }}</span>
				</div>
			</div>
		</template>
		<div class="bg-button-bg pt-1 pb-2 border border-solid border-surface-5">
			<template v-if="accounts.length > 0">
				<div v-for="account in accounts" :key="account.profile.id" class="flex gap-1 items-center">
					<button
						class="flex items-center flex-shrink flex-grow overflow-clip gap-2 p-2 border-0 bg-transparent cursor-pointer button-base min-w-0"
						@click="setAccount(account)"
					>
						<RadioButtonCheckedIcon
							v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
							class="w-5 h-5 text-brand shrink-0"
						/>
						<RadioButtonIcon v-else class="w-5 h-5 text-secondary shrink-0" />
						<Avatar :src="getAccountAvatarUrl(account)" size="24px" />
						<p
							class="m-0 truncate min-w-0 inline-flex items-center gap-1"
							:class="
								selectedAccount && selectedAccount.profile.id === account.profile.id
									? 'text-contrast font-semibold'
									: 'text-primary'
							"
						>
							<component
								:is="getAccountType(account)"
								v-if="getAccountType(account)"
								class="vector-icon shrink-0"
							/>
							<span class="truncate">{{ account.profile.name }}</span>
						</p>
					</button>
					<ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
						<button
							v-tooltip="formatMessage(messages.removeAccount)"
							class="mr-2"
							@click="logout(account.profile.id)"
						>
							<TrashIcon />
						</button>
					</ButtonStyled>
				</div>
			</template>
			<div class="flex flex-col gap-2 px-2 pt-2">
				<ButtonStyled class="w-full" color="brand">
					<button :disabled="loginDisabled" @click="login()">
						<MicrosoftIcon v-if="!loginDisabled" />
						<SpinnerIcon v-else class="animate-spin" />
						{{ formatMessage(messages.addMicrosoftAccount) }}
					</button>
				</ButtonStyled>
				<ButtonStyled class="w-full">
					<OverflowMenu class="w-full justify-between text-left" :options="additionalAccountOptions">
						<span class="inline-flex items-center gap-2">
							<PlusIcon />
							{{ formatMessage(messages.addAccount) }}
						</span>
						<DropdownIcon class="shrink-0" />
						<template #add_offline_account>
							<OfflineIcon />
							{{ formatMessage(messages.addOfflineAccount) }}
						</template>
						<template #add_elyby_account>
							<ElyByIcon v-if="!elyByLoginDisabled" />
							<SpinnerIcon v-else class="animate-spin" />
							{{ formatMessage(messages.addElyByAccount) }}
						</template>
					</OverflowMenu>
				</ButtonStyled>
			</div>
		</div>
	</Accordion>
	<AccountsInputModals
		ref="accountsInputModals"
		:ely-by-login-disabled="elyByLoginDisabled"
		:ely-by-login-value="elyByLoginValue"
		:ely-by-password="elyByPassword"
		:ely-by-two-factor-code="elyByTwoFactorCode"
		:offline-login-disabled="offlineLoginDisabled"
		:offline-player-name="offlinePlayerName"
		@submit-elyby="addElyByProfile"
		@submit-offline="addOfflineProfile"
		@update:ely-by-login-value="elyByLoginValue = $event"
		@update:ely-by-password="elyByPassword = $event"
		@update:ely-by-two-factor-code="elyByTwoFactorCode = $event"
		@update:offline-player-name="offlinePlayerName = $event"
	/>
	<AccountsErrorModals
		ref="accountsErrorModals"
		:max-offline-player-name-length="maxOfflinePlayerNameLength"
		:min-offline-player-name-length="minOfflinePlayerNameLength"
		:name-exp="nameExp"
		@retry-elyby="retryAddElyByProfile"
		@retry-offline="retryAddOfflineProfile"
	/>
</template>

<script setup lang="ts">
import AccountsErrorModals from '@/components/ui/astralrinth/accounts/error/AccountsErrorModals.vue'
import AccountsInputModals from '@/components/ui/astralrinth/accounts/input/AccountsInputModals.vue'
import { trackEvent } from '@/helpers/analytics'
import {
	elyby_auth_authenticate,
	elyby_login,
	get_default_user,
	login as login_flow,
	offline_login,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { process_listener } from '@/helpers/events'
import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer.ts'
import type { Skin } from '@/helpers/skins'
import { get_available_skins } from '@/helpers/skins'
import { handleSevereError } from '@/store/error.js'
import {
	DropdownIcon,
	ElyByIcon,
	MicrosoftIcon,
	OfflineIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Accordion,
	Avatar,
	ButtonStyled,
	defineMessages,
	injectNotificationManager,
	OverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed, onUnmounted, ref } from 'vue'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const emit = defineEmits<{
	change: []
}>()

type MinecraftCredential = {
	account_type?: 'microsoft' | 'offline' | 'elyby' | string
	profile: {
		id: string
		name: string
	}
}

type AccountsInputModalsHandle = {
	hideElyBy: () => void
	hideElyByTwoFactor: () => void
	hideOffline: () => void
	showElyBy: () => void
	showElyByTwoFactor: () => void
	showOffline: () => void
}

type AccountsErrorModalsHandle = {
	hideAuthenticationElyByError: () => void
	hideInputElyByError: () => void
	hideInputOfflineError: () => void
	showAuthenticationElyByError: () => void
	showInputElyByError: () => void
	showInputOfflineError: () => void
	showUnexpectedError: () => void
}

const clientToken = 'astralrinth'
const offlineLoginCooldownMs = 1000
const minOfflinePlayerNameLength = 3
const maxOfflinePlayerNameLength = 20
const nameExp = 'a-zA-Z0-9_'
const nameRegex = new RegExp(`^[${nameExp}]+$`)

const accounts: Ref<MinecraftCredential[]> = ref([])
const loginDisabled = ref(false)
const elyByLoginDisabled = ref(false)
const offlineLoginDisabled = ref(false)
const defaultUser = ref<string | undefined>()
const equippedSkin = ref<Skin | null>(null)
const headUrlCache = ref(new Map<string, string>())

const accountsInputModals = ref<AccountsInputModalsHandle | null>(null)
const accountsErrorModals = ref<AccountsErrorModalsHandle | null>(null)

const offlinePlayerName = ref('')
const elyByLoginValue = ref('')
const elyByPassword = ref('')
const elyByTwoFactorCode = ref('')

function getAccountType(account?: MinecraftCredential) {
	switch (account?.account_type) {
		case 'microsoft':
			return MicrosoftIcon
		case 'offline':
			return OfflineIcon
		case 'elyby':
			return ElyByIcon
		default:
			return null
	}
}

function showOfflineLoginModal() {
	accountsInputModals.value?.showOffline()
}

function showElyByLoginModal() {
	accountsInputModals.value?.showElyBy()
}

const additionalAccountOptions = computed(() => [
	{
		id: 'add_offline_account',
		action: showOfflineLoginModal,
	},
	{
		id: 'add_elyby_account',
		action: showElyByLoginModal,
		disabled: elyByLoginDisabled.value,
	},
])

function retryAddOfflineProfile() {
	accountsErrorModals.value?.hideInputOfflineError()
	offlineLoginDisabled.value = false
	clearOfflineFields()
	showOfflineLoginModal()
}

function retryAddElyByProfile() {
	accountsErrorModals.value?.hideAuthenticationElyByError()
	accountsErrorModals.value?.hideInputElyByError()
	elyByLoginDisabled.value = false
	clearElyByFields()
	showElyByLoginModal()
}

function clearElyByFields() {
	elyByLoginValue.value = ''
	elyByPassword.value = ''
	elyByTwoFactorCode.value = ''
}

function clearOfflineFields() {
	offlinePlayerName.value = ''
}

async function addOfflineProfile() {
	if (offlineLoginDisabled.value) {
		return
	}

	offlineLoginDisabled.value = true

	const name = offlinePlayerName.value.trim()
	const isValidName =
		nameRegex.test(name) &&
		name.length >= minOfflinePlayerNameLength &&
		name.length <= maxOfflinePlayerNameLength

	if (!isValidName) {
		accountsInputModals.value?.hideOffline()
		accountsErrorModals.value?.showInputOfflineError()
		offlineLoginDisabled.value = false
		clearOfflineFields()
		return
	}

	try {
		const result = await offline_login(name)
		accountsInputModals.value?.hideOffline()

		if (result) {
			await setAccount(result)
			await refreshValues()
		} else {
			accountsErrorModals.value?.showUnexpectedError()
		}
	} catch (error) {
		handleError(error)
		accountsErrorModals.value?.showUnexpectedError()
	} finally {
		clearOfflineFields()
		window.setTimeout(() => {
			offlineLoginDisabled.value = false
		}, offlineLoginCooldownMs)
	}
}

async function addElyByProfile() {
	elyByLoginDisabled.value = true

	if (!elyByLoginValue.value || !elyByPassword.value) {
		accountsInputModals.value?.hideElyBy()
		accountsErrorModals.value?.showInputElyByError()
		clearElyByFields()
		elyByLoginDisabled.value = false
		return
	}

	const login = elyByLoginValue.value.trim()
	let password = elyByPassword.value.trim()
	const twoFactorCode = elyByTwoFactorCode.value.trim()

	if (password && twoFactorCode) {
		password = `${password}:${twoFactorCode}`
	}

	try {
		const rawResult = await elyby_auth_authenticate(login, password, clientToken)
		const jsonData = JSON.parse(rawResult)

		if (!jsonData.accessToken) {
			if (
				jsonData.error === 'ForbiddenOperationException' &&
				jsonData.errorMessage?.includes('two factor')
			) {
				accountsInputModals.value?.showElyByTwoFactor()
				return
			}

			accountsInputModals.value?.hideElyBy()
			accountsInputModals.value?.hideElyByTwoFactor()
			accountsErrorModals.value?.showAuthenticationElyByError()
			return
		}

		const accessToken = jsonData.accessToken
		const selectedProfileId = convertRawStringToUUIDv4(jsonData.selectedProfile.id)
		const selectedProfileName = jsonData.selectedProfile.name
		const result = await elyby_login(selectedProfileId, selectedProfileName, accessToken)

		accountsInputModals.value?.hideElyBy()
		accountsInputModals.value?.hideElyByTwoFactor()
		clearElyByFields()

		await setAccount(result)
		await refreshValues()
	} catch (error) {
		handleError(error)
		accountsErrorModals.value?.showUnexpectedError()
	} finally {
		elyByLoginDisabled.value = false
	}
}

function convertRawStringToUUIDv4(rawId: string) {
	if (rawId.length !== 32) {
		console.warn('Invalid UUID string:', rawId)
		return rawId
	}

	return `${rawId.slice(0, 8)}-${rawId.slice(8, 12)}-${rawId.slice(12, 16)}-${rawId.slice(16, 20)}-${rawId.slice(20)}`
}

async function refreshValues() {
	defaultUser.value = await get_default_user().catch(handleError)
	const userList = await users().catch(handleError)
	accounts.value = Array.isArray(userList) ? [...userList] : []
	accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))

	try {
		const skins = await get_available_skins()
		equippedSkin.value = skins.find((skin) => skin.is_equipped) ?? null

		if (equippedSkin.value) {
			try {
				const headUrl = await getPlayerHeadUrl(equippedSkin.value)
				headUrlCache.value = new Map(headUrlCache.value).set(
					equippedSkin.value.texture_key,
					headUrl,
				)
			} catch (error) {
				console.warn('Failed to get head render for equipped skin:', error)
			}
		}
	} catch {
		equippedSkin.value = null
	}
}

async function setEquippedSkin(skin: Skin) {
	equippedSkin.value = skin

	try {
		const headUrl = await getPlayerHeadUrl(skin)
		headUrlCache.value = new Map(headUrlCache.value).set(skin.texture_key, headUrl)
	} catch (error) {
		console.warn('Failed to get head render for equipped skin:', error)
	}
}

function setLoginDisabled(value: boolean) {
	loginDisabled.value = value
}

defineExpose({
	refreshValues,
	setEquippedSkin,
	setLoginDisabled,
	loginDisabled,
})

await refreshValues()

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)

const avatarUrl = computed(() => {
	if (equippedSkin.value?.texture_key) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}

		return `https://mc-heads.net/avatar/${equippedSkin.value.texture_key}/128`
	}

	if (selectedAccount.value?.profile?.id) {
		return `https://mc-heads.net/avatar/${selectedAccount.value.profile.id}/128`
	}

	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
})

function getAccountAvatarUrl(account: MinecraftCredential) {
	if (
		account.profile.id === selectedAccount.value?.profile?.id &&
		equippedSkin.value?.texture_key
	) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}
	}

	return `https://mc-heads.net/avatar/${account.profile.id}/128`
}

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
}

async function logout(id: string) {
	await remove_user(id).catch(handleError)
	await refreshValues()

	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
	} else {
		emit('change')
	}

	trackEvent('AccountLogOut')
}

const unlisten = await process_listener(async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

onUnmounted(() => {
	unlisten()
})

const messages = defineMessages({
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Not signed in',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
	},
	addMicrosoftAccount: {
		id: 'minecraft-account.add-microsoft-account',
		defaultMessage: 'Add Microsoft account',
	},
	addOfflineAccount: {
		id: 'astralrinth.app.minecraft-account.add-offline-account',
		defaultMessage: 'Add offline account',
	},
	addElyByAccount: {
		id: 'astralrinth.app.minecraft-account.add-elyby-account',
		defaultMessage: 'Add Ely.by account',
	},
	removeAccount: {
		id: 'minecraft-account.remove-account',
		defaultMessage: 'Remove account',
	},
	selectAccount: {
		id: 'minecraft-account.select-account',
		defaultMessage: 'Select account',
	},
	minecraftAccount: {
		id: 'minecraft-account.label',
		defaultMessage: 'Minecraft account',
	},
	signInToMinecraft: {
		id: 'minecraft-account.sign-in',
		defaultMessage: 'Sign in to Minecraft',
	},
})
</script>

<style scoped lang="scss">
.vector-icon {
	width: 0.875rem;
	height: 0.875rem;
}
</style>
