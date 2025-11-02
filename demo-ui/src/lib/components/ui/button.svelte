<script lang="ts">
	import { cn } from '$lib/utils/cn';
	
	interface Props {
		variant?: 'default' | 'secondary' | 'outline' | 'ghost' | 'success' | 'danger';
		size?: 'default' | 'sm' | 'lg';
		class?: string;
		disabled?: boolean;
		type?: 'button' | 'submit' | 'reset';
		onclick?: (e: MouseEvent) => void;
		children?: any;
	}
	
	let {
		variant = 'default',
		size = 'default',
		class: className,
		disabled = false,
		type = 'button',
		onclick,
		children
	}: Props = $props();
	
	const variants = {
		default: 'bg-gradient-to-r from-blue-600 to-blue-500 text-white hover:from-blue-700 hover:to-blue-600 shadow-md hover:shadow-lg active:scale-[0.98]',
		secondary: 'bg-gray-200 text-gray-900 hover:bg-gray-300 hover:shadow-md active:scale-[0.98]',
		outline: 'border-2 border-gray-300 bg-transparent hover:bg-gray-50 hover:border-gray-400 hover:shadow-sm active:scale-[0.98]',
		ghost: 'hover:bg-gray-100 hover:shadow-sm active:scale-[0.98]',
		success: 'bg-gradient-to-r from-green-600 to-emerald-500 text-white hover:from-green-700 hover:to-emerald-600 shadow-md hover:shadow-lg active:scale-[0.98]',
		danger: 'bg-gradient-to-r from-red-600 to-red-500 text-white hover:from-red-700 hover:to-red-600 shadow-md hover:shadow-lg active:scale-[0.98]'
	};
	
	const sizes = {
		default: 'min-h-[44px] px-6 py-3 text-base',
		sm: 'min-h-[36px] px-4 py-2 text-sm',
		lg: 'min-h-[52px] px-8 py-4 text-lg'
	};
</script>

<button
	{type}
	{disabled}
	onclick={onclick}
	class={cn(
		'inline-flex items-center justify-center rounded-lg font-semibold',
		'transition-all duration-150 ease-in-out',
		'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-blue-500',
		'disabled:pointer-events-none disabled:opacity-50 disabled:cursor-not-allowed',
		variants[variant],
		sizes[size],
		className
	)}
>
	{@render children?.()}
</button>

