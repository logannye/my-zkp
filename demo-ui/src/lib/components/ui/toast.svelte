<script lang="ts">
	import { CheckCircle } from 'lucide-svelte';
	
	interface Props {
		show: boolean;
		message: string;
		onClose: () => void;
	}
	
	let { show, message, onClose }: Props = $props();
</script>

{#if show}
	<div class="fixed top-4 right-4 z-50 animate-in slide-in-from-top-2 duration-300">
		<div class="bg-gradient-to-r from-green-500 to-emerald-600 text-white px-6 py-4 rounded-xl shadow-lg flex items-center space-x-3 min-w-[320px]">
			<CheckCircle class="w-6 h-6 flex-shrink-0" />
			<p class="font-semibold flex-1">{message}</p>
		</div>
	</div>
{/if}

