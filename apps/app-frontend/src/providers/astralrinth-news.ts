import { createContext } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { useTimestamp } from '@vueuse/core'
import { computed } from 'vue'

import { ASTRALRINTH_NEWS_API, fetchAstralRinthNews } from '@/helpers/astralrinth/news'

const RECENT_NEWS_MAX_AGE = 4 * 24 * 60 * 60 * 1000

export function useAstralRinthNews() {
	const now = useTimestamp({ interval: 1000 })

	const query = useQuery({
		queryKey: ['astralrinth', 'news', ASTRALRINTH_NEWS_API] as const,
		queryFn: ({ signal }) => fetchAstralRinthNews(signal),
		enabled: false,
		staleTime: Infinity,
		retry: false,
		refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	})

	function isRecentNews(publishedAt: string): boolean {
		const age = now.value - Date.parse(publishedAt)
		return age >= 0 && age <= RECENT_NEWS_MAX_AGE
	}

	const hasRecentNews = computed(() => {
		const latest = query.data.value?.[0]
		if (!latest) {
			return false
		}
		return isRecentNews(latest.published_at)
	})

	function load() {
		if (query.isFetching.value || query.isFetched.value) {
			return
		}
		return query.refetch()
	}

	return {
		articles: query.data,
		error: query.error,
		isPending: query.isPending,
		isError: query.isError,

		hasRecentNews,
		isRecentNews,
		load,
	}
}

export const [injectAstralRinthNews, provideAstralRinthNews] = createContext<
	ReturnType<typeof useAstralRinthNews>
>('root', 'astralrinthNews')
