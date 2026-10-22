<script setup lang="ts">
import { TagItem } from '#ui/components'
import { externalProjectLicenseStatusMessages } from '#ui/utils'

import { useVIntl } from '../../composables/i18n'
import type { ExternalLicenseStatus } from './external-project-utils'

const props = defineProps<{
	state: ExternalLicenseStatus
}>()

const { formatMessage } = useVIntl()
</script>

<template>
	<TagItem>
		{{ formatMessage(externalProjectLicenseStatusMessages[props.state]) }}
	</TagItem>
</template>
