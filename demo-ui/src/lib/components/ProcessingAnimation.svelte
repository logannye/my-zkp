<script lang="ts">
	import { Loader } from 'lucide-svelte';
	import Card from './ui/card.svelte';
	import Progress from './ui/progress.svelte';
	import { onMount } from 'svelte';
	
	interface Props {
		isProcessing: boolean;
	}
	
	let { isProcessing }: Props = $props();
	
	let progress = $state(0);
	let currentStep = $state('');
	let stepIndex = $state(0);
	
	const steps = [
		{ text: '📄 Parsing patient record...', progress: 20 },
		{ text: '🔍 Extracting medical features...', progress: 40 },
		{ text: '⚖️ Evaluating authorization criteria...', progress: 60 },
		{ text: '🔐 Generating zero-knowledge proof...', progress: 80 },
		{ text: '✅ Creating decision record...', progress: 100 }
	];
	
	onMount(() => {
		if (isProcessing) {
			const interval = setInterval(() => {
				if (stepIndex < steps.length) {
					currentStep = steps[stepIndex].text;
					progress = steps[stepIndex].progress;
					stepIndex++;
				} else {
					clearInterval(interval);
				}
			}, 600);
			
			return () => clearInterval(interval);
		}
	});
</script>

<Card class="max-w-2xl mx-auto">
	<div class="space-y-6">
		<div class="text-center">
			<h2 class="text-2xl font-bold text-gray-900">Processing Authorization Request</h2>
			<p class="mt-2 text-gray-600">
				Generating zero-knowledge proof...
			</p>
		</div>
		
		<div class="flex justify-center py-8">
			<div class="relative">
				<div class="absolute inset-0 bg-gradient-to-br from-blue-400 to-indigo-500 rounded-full blur-2xl opacity-20 animate-pulse"></div>
				<div class="relative">
					<Loader class="w-20 h-20 text-primary animate-spin" />
				</div>
			</div>
		</div>
		
		<div class="space-y-4">
			<Progress value={progress} class="h-3" />
			
			<div class="text-center">
				<p class="text-lg font-semibold text-gray-900 min-h-[1.75rem] transition-all duration-300">
					{currentStep}
				</p>
				<p class="text-sm text-gray-500 mt-2">
					{progress}% complete
				</p>
			</div>
			
			<!-- Step indicators -->
			<div class="flex justify-center items-center space-x-2 pt-2">
				{#each steps as step, i}
					<div 
						class="w-2 h-2 rounded-full transition-all duration-300"
						class:bg-green-500={i < stepIndex}
						class:bg-blue-500={i === stepIndex}
						class:bg-gray-300={i > stepIndex}
					></div>
				{/each}
			</div>
		</div>
		
		<div class="bg-blue-50/30 border-l-4 border-blue-500 rounded-lg p-4 shadow-sm">
			<div class="flex items-start space-x-3">
				<div class="relative mt-0.5">
					<div class="absolute inset-0 bg-blue-400 rounded-full blur-md opacity-20"></div>
					<svg class="relative w-6 h-6 text-blue-600" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</div>
				<div class="flex-1">
					<h3 class="text-sm font-semibold text-blue-900 mb-2">What's Happening?</h3>
					<ul class="text-sm text-blue-700 space-y-1.5">
						<li class="flex items-center space-x-2">
							<span class="text-blue-500">•</span>
							<span>Converting medical data to mathematical constraints</span>
						</li>
						<li class="flex items-center space-x-2">
							<span class="text-blue-500">•</span>
							<span>Evaluating policy rules against patient criteria</span>
						</li>
						<li class="flex items-center space-x-2">
							<span class="text-blue-500">•</span>
							<span>Generating cryptographic commitment and proof</span>
						</li>
						<li class="flex items-center space-x-2">
							<span class="text-blue-500">•</span>
							<span>Creating verifiable decision record</span>
						</li>
					</ul>
				</div>
			</div>
		</div>
		
		<div class="text-center text-sm text-gray-500">
			<p>This process typically takes 3-5 seconds</p>
		</div>
	</div>
</Card>

