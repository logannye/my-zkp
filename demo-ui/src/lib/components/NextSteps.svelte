<script lang="ts">
	import { Download, Copy, RotateCcw, FileText, Send, Building2, FileCheck, Loader, Sparkles } from 'lucide-svelte';
	import Card from './ui/card.svelte';
	import Button from './ui/button.svelte';
	import Modal from './ui/modal.svelte';
	import type { AuthorizationResult, DecisionRecord } from '$lib/types';
	
	interface Props {
		result: AuthorizationResult | null;
		decisionRecord: DecisionRecord | null;
		onReset: () => void;
	}
	
	let {
		result,
		decisionRecord,
		onReset
	}: Props = $props();
	
	let copySuccess = $state(false);
	let showSendModal = $state(false);
	let isSending = $state(false);
	
	const nextStepsConfig = $derived.by(() => {
		if (!result) return null;
		
		const configs = {
			APPROVE: {
				title: 'Next Steps',
				steps: [
					'Schedule procedure with patient',
					'Submit claim with attached proof',
					'No further authorization needed'
				]
			},
			NEEDS_PA: {
				title: 'Next Steps',
				steps: [
					'Download decision record (includes proof)',
					'Submit PA request via payer portal',
					'Attach decision record to PA submission',
					'Payer verifies proof (patient data stays private)'
				]
			},
			DENY: {
				title: 'Next Steps',
				steps: [
					'Review policy requirements',
					'Check patient eligibility criteria',
					'Consider alternative procedures',
					'Resubmit if criteria are met'
				]
			}
		};
		
		return configs[result];
	});
	
	function downloadDecisionRecord() {
		if (!decisionRecord) return;
		
		const blob = new Blob([JSON.stringify(decisionRecord, null, 2)], {
			type: 'application/json'
		});
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `authorization-${decisionRecord.code}-${Date.now()}.json`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}
	
	async function copyProofToClipboard() {
		if (!decisionRecord?.proof) return;
		
		try {
			await navigator.clipboard.writeText(decisionRecord.proof);
			copySuccess = true;
			setTimeout(() => {
				copySuccess = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy:', err);
		}
	}
	
	async function sendToPayer() {
		// Show loading state
		isSending = true;
		
		// Simulate proof transmission to payer (1.8 second delay)
		await new Promise(resolve => setTimeout(resolve, 1800));
		
		// Reset loading state and show success modal
		isSending = false;
		showSendModal = true;
	}
</script>

{#if result && nextStepsConfig}
	<Card class="max-w-2xl mx-auto mt-6">
		<div class="space-y-6">
			<div>
				<h3 class="text-xl font-bold text-gray-900 mb-4">
					{nextStepsConfig.title}
				</h3>
				<ol class="space-y-2">
					{#each nextStepsConfig.steps as step, index}
						<li class="flex items-start space-x-3">
							<span class="flex-shrink-0 flex items-center justify-center w-6 h-6 rounded-full bg-primary text-white text-sm font-semibold">
								{index + 1}
							</span>
							<span class="text-gray-700 pt-0.5">{step}</span>
						</li>
					{/each}
				</ol>
			</div>
			
			<!-- Actions -->
			<div class="border-t border-gray-200 pt-6">
				<h4 class="text-sm font-semibold text-gray-900 mb-3">Actions</h4>
				
				<!-- Primary CTA: Send to Payer -->
				{#if result !== 'DENY'}
					<div class="mb-4">
						<Button 
							variant="success" 
							onclick={sendToPayer} 
							class="w-full" 
							size="lg"
							disabled={isSending}
						>
							{#if isSending}
								<Loader class="w-5 h-5 mr-2 animate-spin" />
								Sending...
							{:else}
								<Send class="w-5 h-5 mr-2" />
								Send to Payer
							{/if}
						</Button>
					<p class="text-xs mt-2 text-center">
						{#if isSending}
							<span class="inline-flex items-center space-x-1.5 text-purple-600">
								<Sparkles class="w-3.5 h-3.5" />
								<span class="font-medium">AI agent securely transmitting proof to payer...</span>
							</span>
						{:else}
							<span class="text-gray-500">Securely submit authorization proof to payer portal</span>
						{/if}
					</p>
					</div>
				{/if}
				
				<!-- Secondary Actions -->
				<div class="grid grid-cols-2 gap-3">
					<Button variant="outline" onclick={downloadDecisionRecord} class="w-full">
						<Download class="w-4 h-4 mr-2" />
						Download
					</Button>
					<Button
						variant="outline"
						onclick={copyProofToClipboard}
						class="w-full"
					>
						{#if copySuccess}
							✓ Copied!
						{:else}
							<Copy class="w-4 h-4 mr-2" />
							Copy Proof
						{/if}
					</Button>
				</div>
			</div>
			
			<!-- Start New Authorization -->
			<div class="border-t border-gray-200 pt-6">
				<Button variant="secondary" onclick={onReset} class="w-full">
					<RotateCcw class="w-4 h-4 mr-2" />
					Start New Authorization
				</Button>
			</div>
			
			<!-- Additional Info -->
			{#if result === 'NEEDS_PA'}
				<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
					<div class="flex items-start space-x-3">
						<FileText class="w-5 h-5 text-warning flex-shrink-0 mt-0.5" />
						<div class="flex-1">
							<h4 class="text-sm font-semibold text-yellow-900">Prior Authorization Submission</h4>
							<p class="text-sm text-yellow-700 mt-1">
								When submitting to the payer, include the downloaded decision record. 
								The payer can verify the proof instantly without accessing patient data.
							</p>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</Card>
	
	<!-- Send to Payer Modal -->
	<Modal
		show={showSendModal}
		onClose={() => showSendModal = false}
		title="Proof Successfully Sent!"
	>
		<div class="space-y-4 text-center">
			<p class="text-gray-700 text-base leading-relaxed">
				Proof of medical necessity has been sent to the payer. No information about the patient was shared other than proven medical necessity of the requested treatment(s).
			</p>
			
			<div class="bg-blue-50 border-l-4 border-blue-500 rounded-lg p-4 text-left">
				<div class="flex items-start space-x-3">
					<Building2 class="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
					<div class="flex-1">
						<h4 class="text-sm font-semibold text-blue-900 mb-1">What Happens Next?</h4>
						<p class="text-sm text-blue-700 leading-relaxed">
							The payer will verify the proof. You'll receive a response within 24-48 hours.
						</p>
					</div>
				</div>
			</div>
			
			<div class="flex items-center justify-center space-x-2 text-sm text-gray-600">
				<FileCheck class="w-4 h-4 text-green-600" />
				<span class="font-medium">Privacy-preserving verification in progress</span>
			</div>
		</div>
	</Modal>
{/if}

