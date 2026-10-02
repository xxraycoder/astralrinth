<script setup lang="ts">
import { AstralRinthLogo } from '@modrinth/assets'

defineProps<{
	description: string
	title: string
}>()
</script>

<template>
	<div class="flex min-w-0 flex-col gap-5">
		<header
			class="glass-surface relative overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3 p-5"
		>
			<div
				class="pointer-events-none absolute -right-16 -top-20 size-56 rounded-full bg-brand-highlight blur-3xl"
			/>
			<div class="relative flex items-center gap-4">
				<div
					class="flex size-12 shrink-0 items-center justify-center rounded-2xl bg-brand-highlight text-brand"
				>
					<AstralRinthLogo class="size-7" />
				</div>
				<div class="min-w-0">
					<h1 class="m-0 text-xl font-bold leading-tight text-contrast">
						{{ title }}
					</h1>
					<p class="m-0 mt-1 max-w-2xl text-sm leading-relaxed text-secondary">
						{{ description }}
					</p>
				</div>
				<div v-if="$slots.actions" class="ml-auto shrink-0">
					<slot name="actions" />
				</div>
			</div>
		</header>

		<main>
			<slot />
		</main>

		<footer v-if="$slots.footer" class="px-1 text-sm text-secondary">
			<slot name="footer" />
		</footer>
	</div>
</template>
