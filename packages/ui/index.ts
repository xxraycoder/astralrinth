import TooltipDirectiveComponent from './src/components/floating/TooltipDirective.vue'
import ServerModpackContentComponent from './src/components/project/server/ServerModpackContent.vue'

export * from './src/components'
export * from './src/composables'
export * from './src/layouts'
export * from './src/locales'
export * from './src/providers'
export * from './src/utils'

export const ServerModpackContent = ServerModpackContentComponent
export type ServerModpackContent = import('./src/utils/v3-projects').ServerModpackContent
export const TooltipDirective = TooltipDirectiveComponent
export type TooltipDirective = import('./src/providers/tooltip').TooltipDirective
export { getCompatibleLoaderAliases, normalizeLoaderAlias } from './src/utils/version-compatibility'
