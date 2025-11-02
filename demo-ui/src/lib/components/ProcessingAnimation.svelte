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
			<Loader class="w-16 h-16 text-primary animate-spin" />
		</div>
		
		<div class="space-y-4">
			<Progress value={progress} class="h-2" />
			
			<div class="text-center">
				<p class="text-lg font-medium text-gray-900 min-h-[1.75rem]">
					{currentStep}
				</p>
				<p class="text-sm text-gray-500 mt-2">
					{progress}% complete
				</p>
			</div>
		</div>
		
		<div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
			<h3 class="text-sm font-semibold text-blue-900 mb-2">What's Happening?</h3>
			<ul class="text-sm text-blue-700 space-y-1">
				<li>• Converting medical data to mathematical constraints</li>
				<li>• Evaluating policy rules against patient criteria</li>
				<li>• Generating cryptographic commitment and proof</li>
				<li>• Creating verifiable decision record</li>
			</ul>
		</div>
		
		<div class="text-center text-sm text-gray-500">
			<p>This process typically takes 3-5 seconds</p>
		</div>
	</div>
</Card>

