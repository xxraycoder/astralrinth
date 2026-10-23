import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { runInNewContext } from 'node:vm'

import ts from 'typescript'

const source = readFileSync(
	new URL(
		'../src/components/ui/settings/astralrinth/ExternalAuthLibrarySettings.vue',
		import.meta.url,
	),
	'utf8',
)
const script = source.match(/<script\b[^>]*>([\s\S]*?)<\/script>/)[1]
const ast = ts.createSourceFile(
	'settings.ts',
	script,
	ts.ScriptTarget.Latest,
	true,
	ts.ScriptKind.TS,
)

function componentFunction(name, context) {
	const declaration = ast.statements.find(
		(node) => ts.isFunctionDeclaration(node) && node.name?.text === name,
	)
	assert.ok(declaration, `${name} is required`)
	const { outputText } = ts.transpileModule(declaration.getText(ast), {
		compilerOptions: { target: ts.ScriptTarget.ES2022 },
	})
	return runInNewContext(`${outputText}\n${name}`, context)
}

const installed = 'authlib-injector-1.2.0.jar'
const remote = 'authlib-injector-1.3.0.jar'
const legacy = 'old_authlib-injector-1.1.0.jar'

function setup(overrides = {}) {
	const calls = { selections: [], installs: [], notifications: [], errors: [] }
	const context = {
		selectExternalAuthLibrary: async (...args) => {
			calls.selections.push(args)
			return true
		},
		installExternalAuthLibrary: async (...args) => calls.installs.push(args),
		addNotification: (notification) => calls.notifications.push(notification),
		handleError: (error) => calls.errors.push(error.message),
		getErrorMessage: String,
		formatMessage: (_message, values) => values.version,
		messages: { installed: {} },
		...overrides,
	}
	return {
		calls,
		select: componentFunction('selectLibrary', context),
		install: componentFunction('installLibrary', context),
		library: {
			provider: { id: 'elyby', name: 'Ely.by' },
			assetNames: [remote, installed, legacy],
			remoteAssetNames: [remote, installed],
			localAssetNames: [installed, legacy],
			selectedAssetName: installed,
			savedAssetName: installed,
			busy: null,
		},
	}
}

test('choosing a remote version installs that exact version, including on a fresh setup', async () => {
	for (const saved of [installed, null]) {
		const { library, calls, select, install } = setup()
		library.selectedAssetName = saved
		library.savedAssetName = saved
		await select(library, remote)
		assert.equal(library.selectedAssetName, remote)
		assert.equal(library.savedAssetName, saved)
		assert.equal(calls.selections.length, 0)
		await install(library)
		assert.deepEqual(calls.installs, [['elyby', remote]])
		assert.equal(library.savedAssetName, remote)
		assert.ok(library.localAssetNames.includes(remote))
		assert.equal(calls.notifications.length, 1)
		assert.equal(library.busy, null)
	}
})

test('local and legacy versions can be selected without downloading', async () => {
	for (const assetName of [installed, legacy]) {
		const { library, calls, select } = setup()
		library.selectedAssetName = remote
		library.savedAssetName = null
		await select(library, assetName)
		assert.equal(library.selectedAssetName, assetName)
		assert.equal(library.savedAssetName, assetName)
		assert.deepEqual(calls.selections, [['elyby', assetName]])
		assert.equal(calls.installs.length, 0)
		assert.equal(library.busy, null)
	}
})

test('returning to the saved version resets the install target without persisting it again', async () => {
	const { library, calls, select, install } = setup()
	await select(library, remote)
	await select(library, installed)
	assert.equal(library.selectedAssetName, installed)
	assert.equal(calls.selections.length, 0)
	await install(library)
	assert.deepEqual(calls.installs, [['elyby', installed]])
	assert.equal(library.localAssetNames.filter((name) => name === installed).length, 1)
})

test('failed or missing local selections do not overwrite the persisted selection', async () => {
	for (const selectExternalAuthLibrary of [
		async () => false,
		async () => {
			throw new Error('selection failed')
		},
	]) {
		const { library, select } = setup({ selectExternalAuthLibrary })
		await select(library, legacy)
		assert.equal(library.savedAssetName, installed)
		assert.equal(library.busy, null)
	}
})

test('failed installation preserves saved and local versions and clears the busy state', async () => {
	const { library, calls, select, install } = setup({
		installExternalAuthLibrary: async () => {
			throw new Error('download failed')
		},
	})
	await select(library, remote)
	await install(library)
	assert.equal(library.selectedAssetName, remote)
	assert.equal(library.savedAssetName, installed)
	assert.ok(!library.localAssetNames.includes(remote))
	assert.equal(calls.notifications.length, 0)
	assert.equal(calls.errors.length, 1)
	assert.equal(library.busy, null)
})

test('busy providers cannot change selection or install, and local-only versions cannot download', async () => {
	const { library, calls, select, install } = setup()
	library.busy = 'installing'
	await select(library, remote)
	await install(library)
	assert.equal(library.selectedAssetName, installed)
	assert.equal(calls.selections.length, 0)
	assert.equal(calls.installs.length, 0)
	library.busy = null
	await select(library, legacy)
	await install(library)
	assert.equal(calls.installs.length, 0)
})
