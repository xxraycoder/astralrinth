<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, nextTick, useTemplateRef } from 'vue'

import { AstralRinthNewsError } from '@/helpers/astralrinth/news'
import { injectAstralRinthNews } from '@/providers/astralrinth-news'

import AstralRinthNewsCard from './astralrinth-news-card.vue'

const props = defineProps<{
	page: number
	pageSize: number
}>()

const { formatMessage } = useVIntl()
const { articles, error, isPending, isError } = injectAstralRinthNews()
const content = useTemplateRef<HTMLElement>('content')
const pageArticles = computed(() =>
	articles.value?.slice((props.page - 1) * props.pageSize, props.page * props.pageSize),
)

async function scrollToTop() {
	await nextTick()
	content.value?.closest('[data-modal-content]')?.scrollTo({ top: 0 })
}

defineExpose({ scrollToTop })

const isRateLimited = computed(
	() => error.value instanceof AstralRinthNewsError && error.value.httpStatus === 429,
)

const messages = defineMessages({
	loading: {
		id: 'astralrinth.app.news.loading',
		defaultMessage: 'Loading AstralRinth news…',
	},
	empty: {
		id: 'astralrinth.app.news.empty',
		defaultMessage: 'No AstralRinth news yet.',
	},
	error: {
		id: 'astralrinth.app.news.error',
		defaultMessage: 'Could not load AstralRinth news.',
	},
	rateLimited: {
		id: 'astralrinth.app.news.rate-limited',
		defaultMessage: 'Too many requests.',
	},
})
</script>

<template>
	<div ref="content" class="flex min-w-0 flex-col gap-6 pb-5">
		<div
			v-if="isPending"
			role="status"
			class="flex items-center justify-center gap-2 py-8 text-secondary"
		>
			<SpinnerIcon aria-hidden="true" class="h-5 w-5 shrink-0 animate-spin" />
			{{ formatMessage(messages.loading) }}
		</div>
		<template v-else>
			<p v-if="isError" role="alert" class="m-0 text-secondary">
				{{ formatMessage(isRateLimited ? messages.rateLimited : messages.error) }}
			</p>

			<template v-if="articles?.length">
				<AstralRinthNewsCard
					v-for="(article, index) in pageArticles"
					:key="`${article.id}-${(page - 1) * pageSize + index}`"
					:article="article"
				/>
			</template>
			<p v-else-if="!isError" role="status" class="m-0 py-8 text-center text-secondary">
				{{ formatMessage(messages.empty) }}
			</p>
		</template>
	</div>
</template>
