<script setup lang="ts">
import { NewspaperIcon } from '@modrinth/assets'
import { Button, defineMessages, NewModal, Pagination, useVIntl } from '@modrinth/ui'
import { computed, ref, useTemplateRef, watch } from 'vue'

import { injectAstralRinthNews } from '@/providers/astralrinth-news'

import AstralRinthNewsList from './astralrinth-news-list.vue'

const { formatMessage } = useVIntl()
const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const isOpen = ref(false)
const { articles, hasRecentNews } = injectAstralRinthNews()
const currentPage = ref(1)
const pageSize = 2
const pageCount = computed(() => Math.max(1, Math.ceil((articles.value?.length ?? 0) / pageSize)))
const list = useTemplateRef<InstanceType<typeof AstralRinthNewsList>>('list')

watch(articles, () => {
	currentPage.value = 1
})

function show() {
	currentPage.value = 1
	isOpen.value = true
}

function switchPage(page: number) {
	currentPage.value = Math.min(Math.max(page, 1), pageCount.value)
	void list.value?.scrollToTop()
}

const messages = defineMessages({
	title: {
		id: 'astralrinth.app.news.title',
		defaultMessage: 'AstralRinth news',
	},
	recentNews: {
		id: 'astralrinth.app.news.recent-available',
		defaultMessage: 'Recent news available',
	},
})
</script>

<template>
	<Button
		type="outlined"
		class="w-full"
		aria-haspopup="dialog"
		:aria-expanded="isOpen"
		@click="modal?.show($event)"
	>
		<NewspaperIcon aria-hidden="true" />
		{{ formatMessage(messages.title) }}
		<span
			v-if="hasRecentNews"
			role="img"
			:aria-label="formatMessage(messages.recentNews)"
			:title="formatMessage(messages.recentNews)"
			class="h-2 w-2 shrink-0 rounded-full bg-brand"
		/>
	</Button>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		width="36rem"
		max-width="calc(100vw - 2rem)"
		scrollable
		:on-show="show"
		:on-after-hide="() => (isOpen = false)"
	>
		<AstralRinthNewsList v-if="isOpen" ref="list" :page="currentPage" :page-size="pageSize" />
		<template v-if="pageCount > 1" #actions>
			<Pagination
				class="justify-center"
				:page="currentPage"
				:count="pageCount"
				@switch-page="switchPage"
			/>
		</template>
	</NewModal>
</template>
