import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { runInNewContext } from 'node:vm'

import ts from 'typescript'

/** Exercises the component's actual functions without mounting Tauri or starting authentication. */
function componentFunction(path, name, context) {
	const source = readFileSync(new URL(`../src/${path}`, import.meta.url), 'utf8')
	const script = source.match(/<script\b[^>]*>([\s\S]*?)<\/script>/)?.[1]
	assert.ok(script, `${path}: script is required`)
	const ast = ts.createSourceFile(path, script, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)
	const declaration = ast.statements.find(
		(node) => ts.isFunctionDeclaration(node) && node.name?.text === name,
	)
	assert.ok(declaration, `${path}: ${name} is required`)
	const { outputText } = ts.transpileModule(declaration.getText(ast), {
		compilerOptions: { target: ts.ScriptTarget.ES2022 },
	})
	return runInNewContext(`${outputText}\n${name}`, context)
}

const skinsPage = 'pages/Skins.vue'

test('skin and required-account entry points use the shared chooser, not Microsoft-only login', () => {
	for (const path of [
		skinsPage,
		'components/ui/minecraft-required-modal/MinecraftRequiredModal.vue',
	]) {
		let opened = 0
		let hidden = 0
		const minecraftAccounts = {
			value: {
				loginDisabled: false,
				showAccountLoginModal: () => opened++,
			},
		}
		const show = componentFunction(path, 'showAccountLoginModal', {
			minecraftAccounts,
			modal: { value: { hide: () => hidden++ } },
		})
		show()
		assert.equal(opened, 1, path)
		assert.equal(hidden, path === skinsPage ? 0 : 1, path)
		minecraftAccounts.value.loginDisabled = true
		show()
		assert.equal(opened, 1, `${path}: disabled sign-in must not reopen the chooser`)
		minecraftAccounts.value = null
		show()
		assert.equal(opened, 1, `${path}: absent sidebar must not cause a direct login`)
	}
})

test('no-account, offline and external states do not request Microsoft skins or capes', async () => {
	for (const accountType of [undefined, 'offline', 'elyby']) {
		for (const name of ['loadCapes', 'loadSkins']) {
			let requests = 0
			const load = componentFunction(skinsPage, name, {
				isMicrosoftAccount: { value: accountType === 'microsoft' },
				get_available_capes: () => requests++,
				get_available_skins: () => requests++,
			})
			await load()
			assert.equal(requests, 0, `${accountType}: ${name}`)
		}
	}
})

test('Microsoft loaders discard responses after the selected account changes', async () => {
	for (const name of ['loadCapes', 'loadSkins']) {
		let complete
		const response = new Promise((resolve) => {
			complete = resolve
		})
		const profile = { value: 'first' }
		const originalItems = []
		const items = { value: originalItems }
		const load = componentFunction(skinsPage, name, {
			isMicrosoftAccount: { value: true },
			currentUserId: profile,
			get_available_capes: () => response,
			get_available_skins: () => response,
			capes: items,
			skins: items,
			handleError: (error) => {
				throw error
			},
		})
		const pending = load()
		profile.value = 'second'
		complete([{ id: 'old-account-item' }])
		await pending
		assert.equal(items.value, originalItems, name)
	}
})

test('Microsoft cape loading still applies a response for the current account', async () => {
	const capes = { value: [] }
	const loadedCapes = [{ id: 'cape' }]
	const load = componentFunction(skinsPage, 'loadCapes', {
		isMicrosoftAccount: { value: true },
		currentUserId: { value: 'current' },
		get_available_capes: async () => loadedCapes,
		capes,
	})
	await load()
	assert.equal(capes.value, loadedCapes)
})

test('delayed skin refresh cannot run after switching accounts or losing Microsoft capability', async () => {
	for (const switchAccount of [true, false]) {
		let callback
		let flushed = false
		const currentUserId = { value: 'microsoft-account' }
		const isMicrosoftAccount = { value: true }
		const schedule = componentFunction(skinsPage, 'schedulePendingSkinRefresh', {
			pendingSkinRefreshTimeout: null,
			isUnmounted: false,
			currentUserId,
			isMicrosoftAccount,
			PENDING_SKIN_REFRESH_DELAY_MS: 11_000,
			window: {
				setTimeout: (run) => {
					callback = run
					return 1
				},
			},
			flush_pending_skin_change_for_profile: () => {
				flushed = true
			},
		})
		schedule()
		if (switchAccount) currentUserId.value = 'another-account'
		else isMicrosoftAccount.value = false
		await callback()
		assert.equal(flushed, false)
	}
})
