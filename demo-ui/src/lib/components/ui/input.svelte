<script lang="ts">
	import { cn } from '$lib/utils/cn';
	
	interface Props {
		type?: string;
		value?: string;
		placeholder?: string;
		class?: string;
		disabled?: boolean;
		onchange?: (e: Event) => void;
	}
	
	let {
		type = 'text',
		value = $bindable(''),
		placeholder,
		class: className,
		disabled = false,
		onchange
	}: Props = $props();
</script>

<input
	{type}
	bind:value
	{placeholder}
	{disabled}
	onchange={onchange}
	class={cn(
		'flex h-10 w-full rounded-md border border-input bg-white px-3 py-2',
		'text-sm ring-offset-background',
		'placeholder:text-muted-foreground',
		'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
		'disabled:cursor-not-allowed disabled:opacity-50',
		className
	)}
/>

