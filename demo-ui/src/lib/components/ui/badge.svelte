<script lang="ts">
	import { cn } from '$lib/utils/cn';
	
	interface Props {
		variant?: 'default' | 'success' | 'warning' | 'danger' | 'secondary';
		class?: string;
		children?: any;
	}
	
	let {
		variant = 'default',
		class: className,
		children
	}: Props = $props();
	
	const variants = {
		default: 'bg-blue-100 text-blue-800 border-blue-200',
		success: 'bg-green-100 text-green-800 border-green-200',
		warning: 'bg-yellow-100 text-yellow-800 border-yellow-200',
		danger: 'bg-red-100 text-red-800 border-red-200',
		secondary: 'bg-gray-100 text-gray-800 border-gray-200'
	};
</script>

<span
	class={cn(
		'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors',
		variants[variant],
		className
	)}
>
	{@render children?.()}
</span>

