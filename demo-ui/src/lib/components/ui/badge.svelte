<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import { Check, Clock } from 'lucide-svelte';
	
	interface Props {
		variant?: 'default' | 'success' | 'warning' | 'danger' | 'secondary';
		class?: string;
		showIcon?: boolean;
		children?: any;
	}
	
	let {
		variant = 'default',
		class: className,
		showIcon = true,
		children
	}: Props = $props();
	
	const variants = {
		default: 'bg-blue-500 text-white shadow-sm',
		success: 'bg-green-500 text-white shadow-sm shadow-green-200',
		warning: 'bg-amber-500 text-white shadow-sm shadow-amber-200',
		danger: 'bg-red-500 text-white shadow-sm shadow-red-200',
		secondary: 'bg-gray-500 text-white shadow-sm'
	};
	
	const icons = {
		default: null,
		success: Check,
		warning: Clock,
		danger: null,
		secondary: null
	};
	
	const Icon = icons[variant];
</script>

<span
	class={cn(
		'inline-flex items-center gap-1 rounded-full px-3 py-1.5 text-xs font-semibold',
		'transition-all duration-150',
		variants[variant],
		className
	)}
>
	{#if showIcon && Icon}
		<Icon class="w-3 h-3" />
	{/if}
	{@render children?.()}
</span>

