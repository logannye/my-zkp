<script lang="ts">
	import { cn } from '$lib/utils/cn';
	
	interface Props {
		class?: string;
		hover?: boolean;
		children?: any;
	}
	
	let { class: className, hover = false, children }: Props = $props();
</script>

<div
	class={cn(
		'rounded-xl border border-gray-200 bg-white p-6',
		'shadow-sm transition-all duration-200 ease-in-out',
		hover && 'hover:shadow-lg hover:-translate-y-0.5 cursor-pointer',
		className
	)}
>
	{@render children?.()}
</div>

