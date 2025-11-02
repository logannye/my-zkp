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
		default: 'bg-primary text-white hover:bg-primary/90',
		secondary: 'bg-gray-200 text-gray-900 hover:bg-gray-300',
		outline: 'border-2 border-gray-300 bg-transparent hover:bg-gray-100',
		ghost: 'hover:bg-gray-100',
		success: 'bg-success text-white hover:bg-success/90',
		danger: 'bg-danger text-white hover:bg-danger/90'
	};
	
	const sizes = {
		default: 'h-10 px-4 py-2',
		sm: 'h-8 px-3 text-sm',
		lg: 'h-12 px-8 text-lg'
	};
</script>

<button
	{type}
	{disabled}
	onclick={onclick}
	class={cn(
		'inline-flex items-center justify-center rounded-md font-medium transition-colors',
		'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
		'disabled:pointer-events-none disabled:opacity-50',
		variants[variant],
		sizes[size],
		className
	)}
>
	{@render children?.()}
</button>

