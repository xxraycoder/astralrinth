import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { runInNewContext } from 'node:vm'

import { onlineManager, QueryClient, queryOptions } from '@tanstack/vue-query'
import ts from 'typescript'

/** Load option factories without importing the Tauri runtime or executing native commands. */
function loadFactories(path) {
	const source = readFileSync(new URL(`../src/${path}`, import.meta.url), 'utf8')
	const { outputText } = ts.transpileModule(source, {
		compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
	})
	const exports = {}
	runInNewContext(outputText, {
		exports,
		require: (name) =>
			name === '@tanstack/vue-query'
				? { queryOptions }
				: new Proxy({}, { get: () => () => undefined }),
	})
	return exports
}

/** Read component option contracts without mounting Vue or invoking native operations. */
function componentOptions(path, variableName) {
	const source = readFileSync(new URL(`../src/${path}`, import.meta.url), 'utf8')
	const script = source.match(/<script\b[^>]*>([\s\S]*?)<\/script>/)?.[1] ?? source
	const ast = ts.createSourceFile(path, script, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)
	let options
	function visit(node) {
		if (
			ts.isVariableDeclaration(node) &&
			(node.name.getText(ast) === variableName ||
				(ts.isObjectBindingPattern(node.name) &&
					node.name.elements.some((element) => element.name.getText(ast) === variableName)))
		) {
			function findOptions(child) {
				if (ts.isObjectLiteralExpression(child)) {
					const mode = child.properties.find(
						(property) =>
							ts.isPropertyAssignment(property) && property.name.getText(ast) === 'networkMode',
					)
					if (mode && ts.isStringLiteral(mode.initializer)) {
						options = { networkMode: mode.initializer.text }
					}
				}
				ts.forEachChild(child, findOptions)
			}
			findOptions(node)
		}
		ts.forEachChild(node, visit)
	}
	visit(ast)
	assert.ok(options, `${path}: ${variableName} needs an explicit network mode`)
	return options
}

const instances = loadFactories('pages/instance/query-options.ts')
const settings = loadFactories('helpers/settings.ts')
const memory = loadFactories('helpers/jre.js')
const game = loadFactories('helpers/game-options.ts')
const sync = loadFactories('helpers/synced-options.ts')
const worlds = loadFactories('components/ui/world/queries.ts')

const localOptions = [
	instances.instanceListQueryOptions(),
	instances.instanceDetailQueryOptions('instance'),
	instances.instanceProcessesQueryOptions('instance'),
	instances.instanceContentQueryOptions('instance'),
	instances.instanceWorldsQueryOptions('instance'),
	instances.syncedScreenshotsQueryOptions(),
	instances.instanceScreenshotsQueryOptions('instance'),
	instances.screenshotGroupsQueryOptions(),
	settings.appSettingsQueryOptions(),
	memory.maxMemoryQueryOptions(),
	game.gameSettingsQueryOptions(),
	game.gameSettingsQueryOptions('instance'),
	sync.globalSyncedOptionsQueryOptions(),
	sync.initializedSyncedOptionsQueryOptions(),
	sync.gameOptionsSyncSourcesQueryOptions(),
	sync.syncedServersQueryOptions(),
	sync.commandHistoryQueryOptions(),
	worlds.recentWorldsQueryOptions(5),
]

const componentQueries = [
	['pages/instance/files/index.vue', 'instanceRootQuery'],
	['pages/instance/files/index.vue', 'directoryQuery'],
	['pages/instance/logs/index.vue', 'consoleHydrationQuery'],
	['components/ui/world/RecentWorldsList.vue', 'processesQuery'],
	['features/settings/app-settings-modal/index.vue', 'appInfo'],
	['features/settings/system/resource-management-settings/content-storage.vue', 'storeUsage'],
	['features/settings/system/resource-management-settings/content-storage.vue', 'runningProcesses'],
	['features/instances/settings-modal/behavior-settings.vue', 'globalSyncedOptionsQuery'],
]
const componentMutations = [
	['features/settings/app-settings-modal/index.vue', 'developerModeMutation'],
	['components/ui/screenshots-page/index.vue', 'deleteMutation'],
	['components/ui/screenshots-page/index.vue', 'exportMutation'],
	['components/ui/screenshots-page/index.vue', 'moveMutation'],
	['components/ui/screenshots-page/index.vue', 'saveEditMutation'],
	[
		'features/settings/system/resource-management-settings/content-storage.vue',
		'cacheLimitMutation',
	],
	['features/settings/syncing/game-settings-modal/use-editor.ts', 'previewMutation'],
	['features/settings/syncing/game-settings-modal/use-editor.ts', 'saveMutation'],
	['features/settings/instance-defaults/use-default-instance-settings.ts', 'mutation'],
	['features/settings/syncing/command-history-modal.vue', 'saveMutation'],
	['features/settings/syncing/servers-modal.vue', 'saveMutation'],
	['features/settings/syncing/servers-modal.vue', 'removeMutation'],
	['features/settings/syncing/syncing-settings.vue', 'globalOptionMutation'],
	['features/instances/settings-modal/synced-options-settings.vue', 'mutation'],
	['features/updates/new-update-modal/use-sync.ts', 'syncMutation'],
	['features/settings/system/troubleshooting-settings.vue', 'copyDetailsMutation'],
]

test(
	'local queries and mutations execute offline; remote queries wait for reconnection',
	{ timeout: 10_000 },
	async () => {
		const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
		const wasOnline = onlineManager.isOnline()
		client.mount()
		try {
			onlineManager.setOnline(false)
			let calls = 0
			for (const [index, options] of [
				...localOptions,
				...componentQueries.map(([path, name]) => componentOptions(path, name)),
			].entries()) {
				assert.equal(options.networkMode, 'always')
				await client.fetchQuery({
					...options,
					queryKey: ['offline-local-query', index],
					queryFn: async () => ++calls,
				})
				assert.equal(client.getQueryState(['offline-local-query', index]).fetchStatus, 'idle')
			}
			assert.equal(calls, localOptions.length + componentQueries.length)

			for (const [path, name] of componentMutations) {
				const options = componentOptions(path, name)
				assert.equal(options.networkMode, 'always')
				const mutation = client.getMutationCache().build(client, {
					...options,
					mutationFn: async () => 'saved locally',
				})
				assert.equal(await mutation.execute(undefined), 'saved locally')
				assert.equal(mutation.state.isPaused, false)
			}

			let remoteCalls = 0
			const remoteOptions = instances.instanceLinkedProjectQueryOptions('project')
			assert.notEqual(remoteOptions.networkMode, 'always')
			assert.notEqual(worlds.serverStatusQueryOptions('server', null).networkMode, 'always')
			const remote = client.fetchQuery({
				...remoteOptions,
				queryFn: async () => ++remoteCalls,
			})
			assert.equal(remoteCalls, 0)
			assert.equal(client.getQueryState(remoteOptions.queryKey).fetchStatus, 'paused')
			onlineManager.setOnline(true)
			assert.equal(await remote, 1)
		} finally {
			await client.cancelQueries()
			client.unmount()
			client.clear()
			onlineManager.setOnline(wasOnline)
		}
	},
)
