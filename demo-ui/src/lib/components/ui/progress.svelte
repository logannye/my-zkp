<script lang="ts">
	import { cn } from '$lib/utils/cn';
	
	interface Props {
		value: number;
		max?: number;
		class?: string;
	}
	
	let {
		value = 0,
		max = 100,
		class: className
	}: Props = $props();
	
	const percentage = $derived(Math.min(100, Math.max(0, (value / max) * 100)));
</script>

<div
	class={cn('relative h-4 w-full overflow-hidden rounded-full bg-gray-200', className)}
	role="progressbar"
	aria-valuemin="0"
	aria-valuemax={max}
	aria-valuenow={value}
>
	<div
		class="h-full bg-primary transition-all duration-300 ease-in-out"
		style="width: {percentage}%"
	></div>
</div>

