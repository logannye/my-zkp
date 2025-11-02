<script lang="ts">
	import { cn } from '$lib/utils/cn';
	
	interface Props {
		class?: string;
		children?: any;
	}
	
	let { class: className, children }: Props = $props();
</script>

<div class={cn('rounded-lg border bg-white shadow-md p-6', className)}>
	{@render children?.()}
</div>

