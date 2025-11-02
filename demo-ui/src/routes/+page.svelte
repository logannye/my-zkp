<script lang="ts">
	import { workflow } from '$lib/stores/workflow.svelte';
	import { availableCodes } from '$lib/utils/mock-data';
	import FileUpload from '$lib/components/FileUpload.svelte';
	import CodeSelector from '$lib/components/CodeSelector.svelte';
	import ReviewSummary from '$lib/components/ReviewSummary.svelte';
	import ProcessingAnimation from '$lib/components/ProcessingAnimation.svelte';
	import ResultsDisplay from '$lib/components/ResultsDisplay.svelte';
	import NextSteps from '$lib/components/NextSteps.svelte';
	import { XCircle } from 'lucide-svelte';
	import '../app.css';
</script>

<div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
	<!-- Header -->
	<header class="bg-white shadow-md border-b border-gray-200">
		<div class="container mx-auto px-4 py-8">
			<div class="flex items-center space-x-4">
				<div class="flex items-center justify-center w-14 h-14 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-600 shadow-lg">
					<svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
					</svg>
				</div>
				<div class="flex-1">
					<h1 class="text-3xl font-bold text-gray-900 tracking-tight">
						Medical Authorization Portal
					</h1>
					<p class="text-gray-600 mt-1">
						Privacy-Preserving Prior Authorization with Zero-Knowledge Proofs
					</p>
				</div>
			</div>
		</div>
	</header>
	
	<!-- Main Content -->
	<main class="container mx-auto px-4 py-8">
		{#key workflow.step}
		<div class="animate-in fade-in duration-300">
			{#if workflow.step === 'upload'}
				<FileUpload onFileSelect={(f) => workflow.uploadFile(f)} />
			{:else if workflow.step === 'select'}
				<CodeSelector
					availableCodes={availableCodes}
					onCodeSelect={(c) => workflow.selectCode(c)}
					onBack={() => workflow.goBack()}
				/>
			{:else if workflow.step === 'review'}
				<ReviewSummary
					patientInfo={workflow.patientInfo}
					selectedCode={workflow.selectedCode}
					onSubmit={() => workflow.submitAuthorization()}
					onBack={() => workflow.goBack()}
				/>
			{:else if workflow.step === 'processing'}
				<ProcessingAnimation isProcessing={workflow.isProcessing} />
			{:else if workflow.step === 'results'}
				<ResultsDisplay
					result={workflow.authorizationResult}
					decisionRecord={workflow.decisionRecord}
				/>
				<NextSteps
					result={workflow.authorizationResult}
					decisionRecord={workflow.decisionRecord}
					onReset={() => workflow.reset()}
				/>
			{/if}
		</div>
		{/key}
		
		<!-- Error Alert -->
		{#if workflow.error}
			<div class="max-w-2xl mx-auto mt-6 animate-in fade-in duration-300">
				<div class="bg-red-50/30 border-l-4 border-red-500 rounded-lg p-5 shadow-sm">
					<div class="flex items-start space-x-3">
						<div class="flex-shrink-0 mt-0.5">
							<div class="relative">
								<div class="absolute inset-0 bg-red-400 rounded-full blur-md opacity-20"></div>
								<XCircle class="relative w-6 h-6 text-red-600" />
							</div>
						</div>
						<div class="flex-1">
							<h3 class="text-sm font-semibold text-red-900">Error Processing Request</h3>
							<p class="text-sm text-red-700 mt-1 leading-relaxed">{workflow.error}</p>
							<button
								onclick={() => workflow.reset()}
								class="mt-3 text-sm font-medium text-red-600 hover:text-red-800 transition-colors"
							>
								Try Again →
							</button>
						</div>
					</div>
				</div>
			</div>
		{/if}
	</main>
	
	<!-- Footer -->
	<footer class="mt-16 border-t border-gray-200 bg-white/50">
		<div class="container mx-auto px-4 py-8 text-center">
			<div class="flex items-center justify-center space-x-2 text-gray-700">
				<svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
				</svg>
				<p class="text-sm font-medium">
					Powered by Zero-Knowledge Proofs
				</p>
				<span class="text-gray-400">|</span>
				<p class="text-sm font-medium">
					HIPAA-Compliant by Design
				</p>
			</div>
			<p class="text-xs text-gray-500 mt-3">
				Demo Application - For Educational Purposes
			</p>
		</div>
	</footer>
</div>

