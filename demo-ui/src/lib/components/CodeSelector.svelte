<script lang="ts">
	import { ArrowLeft } from 'lucide-svelte';
	import Card from './ui/card.svelte';
	import Button from './ui/button.svelte';
	import Badge from './ui/badge.svelte';
	import type { Code } from '$lib/types';
	
	interface Props {
		availableCodes: Code[];
		onCodeSelect: (code: Code) => void;
		onBack: () => void;
	}
	
	let {
		availableCodes,
		onCodeSelect,
		onBack
	}: Props = $props();
	
	let selectedCodeValue = $state('');
	let searchQuery = $state('');
	
	const filteredCodes = $derived(
		availableCodes.filter(
			(code) =>
				code.code.toLowerCase().includes(searchQuery.toLowerCase()) ||
				code.description.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);
	
	function handleSelect(code: Code) {
		selectedCodeValue = code.code;
		onCodeSelect(code);
	}
</script>

<Card class="max-w-2xl mx-auto">
	<div class="space-y-6">
		<div class="flex items-center space-x-4">
			<Button variant="ghost" size="sm" onclick={onBack}>
				<ArrowLeft class="w-4 h-4 mr-2" />
				Back
			</Button>
			<div class="flex-1">
				<h2 class="text-2xl font-bold text-gray-900">Select Procedure Code</h2>
				<p class="mt-1 text-gray-600">
					Choose the CPT/HCPCS code for the requested procedure
				</p>
			</div>
		</div>
		
		<div>
			<label for="search" class="block text-sm font-medium text-gray-700 mb-2">
				Search Codes
			</label>
			<input
				id="search"
				type="text"
				bind:value={searchQuery}
				placeholder="Search by code or description..."
				class="w-full px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
			/>
		</div>
		
		<div class="space-y-3">
			{#each filteredCodes as code (code.code)}
				<button
					onclick={() => handleSelect(code)}
					class="w-full text-left p-4 border-2 rounded-lg transition-all hover:border-primary hover:bg-blue-50"
					class:border-primary={selectedCodeValue === code.code}
					class:bg-blue-50={selectedCodeValue === code.code}
					class:border-gray-200={selectedCodeValue !== code.code}
				>
					<div class="flex items-start justify-between">
						<div class="flex-1">
							<div class="flex items-center space-x-3">
								<span class="text-lg font-bold text-gray-900">{code.code}</span>
								{#if code.requiresPA}
									<Badge variant="warning">PA Required</Badge>
								{:else}
									<Badge variant="success">Auto-Approve</Badge>
								{/if}
							</div>
							<p class="text-gray-700 mt-1">{code.description}</p>
							<p class="text-sm text-gray-500 mt-2">
								Policy: {code.policyId}
							</p>
						</div>
					</div>
				</button>
			{/each}
		</div>
		
		{#if filteredCodes.length === 0}
			<div class="text-center py-8 text-gray-500">
				<p>No codes found matching your search.</p>
			</div>
		{/if}
	</div>
</Card>

