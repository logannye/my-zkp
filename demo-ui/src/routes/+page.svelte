<script lang="ts">
	import { workflow } from '$lib/stores/workflow.svelte';
	import { availableCodes } from '$lib/utils/mock-data';
	import FileUpload from '$lib/components/FileUpload.svelte';
	import CodeSelector from '$lib/components/CodeSelector.svelte';
	import ReviewSummary from '$lib/components/ReviewSummary.svelte';
	import ProcessingAnimation from '$lib/components/ProcessingAnimation.svelte';
	import ResultsDisplay from '$lib/components/ResultsDisplay.svelte';
	import NextSteps from '$lib/components/NextSteps.svelte';
	import '../app.css';
</script>

<div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
	<!-- Header -->
	<header class="bg-white shadow">
		<div class="container mx-auto px-4 py-6">
			<h1 class="text-3xl font-bold text-gray-900">
				🏥 Medical Authorization Portal
			</h1>
			<p class="text-gray-600 mt-2">
				Privacy-Preserving Prior Authorization with Zero-Knowledge Proofs
			</p>
		</div>
	</header>
	
	<!-- Main Content -->
	<main class="container mx-auto px-4 py-8">
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
		
		<!-- Error Alert -->
		{#if workflow.error}
			<div class="max-w-2xl mx-auto mt-6">
				<div class="bg-red-50 border-2 border-red-200 rounded-lg p-4">
					<div class="flex items-start space-x-3">
						<svg class="w-6 h-6 text-red-600 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
							<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
						</svg>
						<div class="flex-1">
							<h3 class="text-sm font-semibold text-red-900">Error</h3>
							<p class="text-sm text-red-700 mt-1">{workflow.error}</p>
						</div>
					</div>
				</div>
			</div>
		{/if}
	</main>
	
	<!-- Footer -->
	<footer class="mt-12 text-center text-gray-600 pb-8">
		<p class="text-sm">
			Powered by Zero-Knowledge Proofs | HIPAA-Compliant by Design
		</p>
		<p class="text-xs text-gray-500 mt-2">
			Demo Application - For Educational Purposes
		</p>
	</footer>
</div>

