<script lang="ts">
	import { CheckCircle, AlertTriangle, XCircle, Shield, ChevronDown, ChevronUp } from 'lucide-svelte';
	import Card from './ui/card.svelte';
	import Badge from './ui/badge.svelte';
	import type { AuthorizationResult, DecisionRecord } from '$lib/types';
	
	interface Props {
		result: AuthorizationResult | null;
		decisionRecord: DecisionRecord | null;
	}
	
	let {
		result,
		decisionRecord
	}: Props = $props();
	
	let showProofDetails = $state(false);
	
	const resultConfig = $derived.by(() => {
		if (!result) return null;
		
		const configs = {
			APPROVE: {
				color: 'text-success',
				bgColor: 'bg-green-50',
				borderColor: 'border-green-500',
				title: '✅ APPROVED',
				message: 'Authorization approved. Proceed with scheduling the procedure.'
			},
			NEEDS_PA: {
				color: 'text-warning',
				bgColor: 'bg-yellow-50',
				borderColor: 'border-yellow-500',
				title: '⚠️ PRIOR AUTHORIZATION REQUIRED',
				message: 'Submit PA request to payer with the attached proof below.'
			},
			DENY: {
				color: 'text-danger',
				bgColor: 'bg-red-50',
				borderColor: 'border-red-500',
				title: '❌ DENIED',
				message: 'Request denied. Review policy requirements and patient eligibility.'
			}
		};
		
		return configs[result];
	});
	
	const proofSize = $derived(
		decisionRecord?.proof ? (decisionRecord.proof.length * 0.75 / 1024).toFixed(2) : '0'
	);
</script>

{#if result && decisionRecord && resultConfig}
	<Card class="max-w-2xl mx-auto">
		<div class="space-y-6">
			<!-- Result Badge -->
			<div class="text-center">
				<div class={`inline-flex items-center justify-center w-20 h-20 rounded-full ${resultConfig.bgColor} mb-4`}>
					{#if result === 'APPROVE'}
						<CheckCircle class={`w-12 h-12 ${resultConfig.color}`} />
					{:else if result === 'NEEDS_PA'}
						<AlertTriangle class={`w-12 h-12 ${resultConfig.color}`} />
					{:else}
						<XCircle class={`w-12 h-12 ${resultConfig.color}`} />
					{/if}
				</div>
				<h2 class={`text-3xl font-bold ${resultConfig.color}`}>
					{resultConfig.title}
				</h2>
				<p class="mt-3 text-lg text-gray-700">
					{resultConfig.message}
				</p>
			</div>
			
			<!-- Privacy Guarantee -->
			<div class="bg-purple-50 border-2 border-purple-200 rounded-lg p-4">
				<div class="flex items-start space-x-3">
					<Shield class="w-6 h-6 text-privacy flex-shrink-0 mt-0.5" />
					<div class="flex-1">
						<h3 class="text-lg font-bold text-purple-900">Privacy Preserved</h3>
						<ul class="mt-2 space-y-1 text-sm text-purple-700">
							<li>✓ No patient data shared with payer</li>
							<li>✓ Only cryptographic proof transmitted ({proofSize} KB)</li>
							<li>✓ HIPAA-compliant by design</li>
							<li>✓ Instant verification ({"<"}1ms)</li>
						</ul>
					</div>
				</div>
			</div>
			
			<!-- Proof Statistics -->
			<div class="grid grid-cols-2 gap-4">
				<div class="bg-gray-50 rounded-lg p-4">
					<p class="text-sm text-gray-500">Policy ID</p>
					<p class="text-lg font-semibold text-gray-900 mt-1 truncate">
						{decisionRecord.policyId}
					</p>
				</div>
				<div class="bg-gray-50 rounded-lg p-4">
					<p class="text-sm text-gray-500">Proof Size</p>
					<p class="text-lg font-semibold text-gray-900 mt-1">
						{proofSize} KB
					</p>
				</div>
				<div class="bg-gray-50 rounded-lg p-4">
					<p class="text-sm text-gray-500">Procedure Code</p>
					<p class="text-lg font-semibold text-gray-900 mt-1">
						{decisionRecord.code}
					</p>
				</div>
				<div class="bg-gray-50 rounded-lg p-4">
					<p class="text-sm text-gray-500">Line of Business</p>
					<p class="text-lg font-semibold text-gray-900 mt-1 uppercase">
						{decisionRecord.lob}
					</p>
				</div>
			</div>
			
			<!-- Decision Record Details -->
			<div class="border border-gray-200 rounded-lg">
				<button
					onclick={() => showProofDetails = !showProofDetails}
					class="w-full flex items-center justify-between p-4 hover:bg-gray-50 transition-colors"
				>
					<span class="font-semibold text-gray-900">View Decision Record Details</span>
					{#if showProofDetails}
						<ChevronUp class="w-5 h-5 text-gray-500" />
					{:else}
						<ChevronDown class="w-5 h-5 text-gray-500" />
					{/if}
				</button>
				
				{#if showProofDetails}
					<div class="border-t border-gray-200 p-4 bg-gray-50">
						<div class="space-y-3 text-sm">
							<div>
								<p class="text-gray-500 font-medium">Policy Hash</p>
								<p class="text-gray-900 font-mono text-xs mt-1 break-all">
									{decisionRecord.policyHash}
								</p>
							</div>
							<div>
								<p class="text-gray-500 font-medium">Patient Commitment</p>
								<p class="text-gray-900 font-mono text-xs mt-1 break-all">
									{decisionRecord.patientCommitment}
								</p>
							</div>
							<div>
								<p class="text-gray-500 font-medium">Claimed Result</p>
								<p class="text-gray-900 mt-1">
									<Badge variant={result === 'APPROVE' ? 'success' : result === 'NEEDS_PA' ? 'warning' : 'danger'}>
										{decisionRecord.claimedResult}
									</Badge>
								</p>
							</div>
							<div>
								<p class="text-gray-500 font-medium">Proof (Base64, truncated)</p>
								<p class="text-gray-900 font-mono text-xs mt-1 break-all">
									{decisionRecord.proof.substring(0, 200)}...
								</p>
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	</Card>
{/if}

