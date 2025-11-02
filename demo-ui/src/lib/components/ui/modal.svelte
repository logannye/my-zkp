<script lang="ts">
	import { CheckCircle, X } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';
	import Button from './button.svelte';

	interface Props {
		show: boolean;
		onClose: () => void;
		title?: string;
		showIcon?: boolean;
	}

	let { show, onClose, title = '', showIcon = true }: Props = $props();
</script>

{#if show}
	<!-- Backdrop -->
	<div
		class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
		transition:fade={{ duration: 200 }}
		onclick={onClose}
		role="presentation"
	>
		<!-- Modal -->
		<div
			class="bg-white rounded-2xl shadow-2xl max-w-md w-full"
			transition:scale={{ duration: 200, start: 0.95 }}
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
		>
			<!-- Close button -->
			<button
				onclick={onClose}
				class="absolute top-4 right-4 text-gray-400 hover:text-gray-600 transition-colors"
				aria-label="Close modal"
			>
				<X class="w-5 h-5" />
			</button>

			<!-- Content -->
			<div class="p-8">
				{#if showIcon}
					<div class="flex justify-center mb-6">
						<div class="relative">
							<div class="absolute inset-0 bg-green-400 rounded-full blur-xl opacity-30 animate-pulse"></div>
							<div class="relative w-20 h-20 rounded-full bg-gradient-to-br from-green-500 to-emerald-600 flex items-center justify-center shadow-lg">
								<CheckCircle class="w-12 h-12 text-white" />
							</div>
						</div>
					</div>
				{/if}

				{#if title}
					<h2 class="text-2xl font-bold text-gray-900 text-center mb-4">
						{title}
					</h2>
				{/if}

				<div class="space-y-4">
					<slot />
				</div>

				<!-- Action buttons -->
				<div class="mt-6 flex justify-center">
					<Button
						variant="default"
						size="lg"
						onclick={onClose}
						class="min-w-[120px]"
					>
						Got It
					</Button>
				</div>
			</div>
		</div>
	</div>
{/if}

