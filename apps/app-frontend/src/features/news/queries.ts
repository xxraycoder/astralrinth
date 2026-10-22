import type { NewsArticle } from '@modrinth/ui'
import { queryOptions } from '@tanstack/vue-query'

type NewsFeedArticle = Omit<NewsArticle, 'path'> & {
	link: string
	description?: string
	excerpt?: string
}

const filteredNewsPhrases = [
	'LGBT',
	'LGBTQ',
	'LGBTQ+',
	'LGBTQIA+',
	'gay',
	'lesbian',
	'bisexual',
	'pansexual',
	'asexual',
	'aromantic',
	'transgender',
	'nonbinary',
	'intersex',
	'homosexual',
	'homosexuality',
	'pride',
]

function shouldHideNewsArticle(article: NewsFeedArticle): boolean {
	const haystack = [article.title, article.summary, article.description, article.excerpt]
		.filter(Boolean)
		.join(' ')
		.toLowerCase()
	return filteredNewsPhrases.some((phrase) => haystack.includes(phrase.toLowerCase()))
}

export const newsKeys = {
	articles: ['news', 'articles'] as const,
}

export function newsArticlesQueryOptions() {
	return queryOptions({
		queryKey: newsKeys.articles,
		queryFn: async ({ signal }): Promise<NewsArticle[]> => {
			const response = await fetch('https://modrinth.com/news/feed/articles.json', { signal })
			if (!response.ok) {
				throw new Error(`Failed to fetch news articles: HTTP ${response.status}`)
			}
			const feed: { articles?: NewsFeedArticle[] } | null = await response.json()
			return (feed?.articles ?? [])
				.filter((article) => !shouldHideNewsArticle(article))
				.map((article) => ({ ...article, path: article.link }))
		},
		staleTime: Infinity,
		retry: false,
		refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	})
}
