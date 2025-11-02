<script lang="ts">
	import { cn } from '$lib/utils/cn';
	
	interface Props {
		value?: string;
		onchange?: (value: string) => void;
		class?: string;
		placeholder?: string;
		children?: any;
	}
	
	let {
		value = $bindable(''),
		onchange,
		class: className,
		placeholder = 'Select an option',
		children
	}: Props = $props();
	
	function handleChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		value = target.value;
		onchange?.(target.value);
	}
</script>

<select
	{value}
	onchange={handleChange}
	class={cn(
		'flex h-10 w-full rounded-md border border-input bg-white px-3 py-2',
		'text-sm ring-offset-background',
		'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
		'disabled:cursor-not-allowed disabled:opacity-50',
		className
	)}
>
	{#if placeholder}
		<option value="" disabled selected>{placeholder}</option>
	{/if}
	{@render children?.()}
</select>

