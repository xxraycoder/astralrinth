<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { ButtonLink, defineMessages, useFormatDateTime, useVIntl } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import type { AstralRinthNewsArticle } from '@/helpers/astralrinth/news'
import { injectAstralRinthNews } from '@/providers/astralrinth-news'

const props = defineProps<{
	article: AstralRinthNewsArticle
}>()

const formatDate = useFormatDateTime({ dateStyle: 'long' })
const { formatMessage } = useVIntl()
const { isRecentNews } = injectAstralRinthNews()
const isNew = computed(() => isRecentNews(props.article.published_at))
const messages = defineMessages({
	new: {
		id: 'astralrinth.app.news.new',
		defaultMessage: 'New',
	},
	openArticle: {
		id: 'astralrinth.app.news.open-article',
		defaultMessage: 'Go to',
	},
})
const imageFailed = ref(false)
const articleUrl = computed(() => getHttpUrl(props.article.url))
const imageUrl = computed(() => {
	const source = props.article.image_url
	if (source && /^data:image\/(?:png|jpeg);base64,/i.test(source)) {
		return source
	}
	return getHttpUrl(source)
})
const publishedAt = computed(() => new Date(props.article.published_at).toISOString())

function getHttpUrl(value: string | null): string | null {
	if (!value) {
		return null
	}
	try {
		const url = new URL(value)
		return url.protocol === 'https:' || url.protocol === 'http:' ? url.href : null
	} catch {
		return null
	}
}

watch(imageUrl, () => {
	imageFailed.value = false
})
</script>

<template>
	<article class="flex min-w-0 flex-col gap-4">
		<img
			v-if="imageUrl && !imageFailed"
			:src="imageUrl"
			:alt="article.title"
			loading="lazy"
			decoding="async"
			class="h-auto w-auto max-h-[300px] max-w-[min(100%,512px)] self-center rounded-xl border border-solid border-surface-5 object-contain"
			@error="imageFailed = true"
		/>
		<div class="flex min-w-0 flex-col gap-2">
			<h3 class="m-0 break-words text-base font-semibold leading-tight text-contrast">
				<a
					v-if="articleUrl"
					:href="articleUrl"
					target="_blank"
					rel="noopener noreferrer"
					class="text-inherit hover:underline focus-visible:underline"
				>
					{{ article.title }}
				</a>
				<template v-else>{{ article.title }}</template>
			</h3>
			<p
				v-if="article.summary"
				class="m-0 whitespace-pre-line break-words text-sm leading-tight text-primary"
			>
				{{ article.summary }}
			</p>
			<div class="flex flex-wrap items-center gap-2">
				<time :datetime="publishedAt" class="text-sm text-secondary">
					{{ formatDate(article.published_at) }}
				</time>
				<span
					v-if="isNew"
					class="shrink-0 rounded-full bg-brand-highlight px-1.5 py-0.5 text-xs font-bold text-brand-green"
				>
					{{ formatMessage(messages.new) }}
				</span>
			</div>
			<ButtonLink
				v-if="articleUrl"
				:href="articleUrl"
				type="outlined"
				target="_blank"
				rel="noopener noreferrer"
			>
				{{ formatMessage(messages.openArticle) }}
				<ExternalIcon aria-hidden="true" />
			</ButtonLink>
		</div>
	</article>
</template>
