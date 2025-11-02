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
				<div class="relative inline-flex items-center justify-center w-24 h-24 rounded-full mb-6">
					{#if result === 'APPROVE'}
						<div class="absolute inset-0 bg-gradient-to-br from-green-400 to-emerald-500 rounded-full blur-2xl opacity-30 animate-pulse"></div>
						<div class="relative bg-gradient-to-br from-green-500 to-emerald-600 rounded-full p-6 shadow-lg">
							<CheckCircle class="w-12 h-12 text-white" />
						</div>
					{:else if result === 'NEEDS_PA'}
						<div class="absolute inset-0 bg-gradient-to-br from-amber-400 to-orange-500 rounded-full blur-2xl opacity-30 animate-pulse"></div>
						<div class="relative bg-gradient-to-br from-amber-500 to-orange-600 rounded-full p-6 shadow-lg">
							<AlertTriangle class="w-12 h-12 text-white" />
						</div>
					{:else}
						<div class="absolute inset-0 bg-gradient-to-br from-red-400 to-rose-500 rounded-full blur-2xl opacity-30 animate-pulse"></div>
						<div class="relative bg-gradient-to-br from-red-500 to-rose-600 rounded-full p-6 shadow-lg">
							<XCircle class="w-12 h-12 text-white" />
						</div>
					{/if}
				</div>
				<h2 class={`text-4xl font-bold ${resultConfig.color}`}>
					{resultConfig.title}
				</h2>
				<p class="mt-4 text-lg text-gray-700 max-w-md mx-auto">
					{resultConfig.message}
				</p>
			</div>
			
		<!-- Privacy Guarantee -->
		<div class="bg-purple-50/30 border-l-4 border-purple-500 rounded-lg p-4 shadow-sm">
			<div class="flex items-start space-x-3">
				<div class="flex-shrink-0 mt-0">
					<div class="relative">
						<div class="absolute inset-0 bg-purple-400 rounded-full blur-lg opacity-30"></div>
						<Shield class="relative w-6 h-6 text-purple-600" />
					</div>
				</div>
				<div class="flex-1">
					<h3 class="text-base font-bold text-purple-900">Privacy Preserved</h3>
					<ul class="mt-2 space-y-1.5 text-sm text-purple-700">
							<li class="flex items-center space-x-2">
								<span class="text-purple-500 font-bold">✓</span>
								<span>No patient data shared with payer</span>
							</li>
							<li class="flex items-center space-x-2">
								<span class="text-purple-500 font-bold">✓</span>
								<span>Only cryptographic proof transmitted ({proofSize} KB)</span>
							</li>
							<li class="flex items-center space-x-2">
								<span class="text-purple-500 font-bold">✓</span>
								<span>HIPAA-compliant by design</span>
							</li>
							<li class="flex items-center space-x-2">
								<span class="text-purple-500 font-bold">✓</span>
								<span>Instant verification ({"<"}1ms)</span>
							</li>
						</ul>
					</div>
				</div>
			</div>
			
			<!-- Proof Statistics -->
			<div class="grid grid-cols-2 gap-4">
				<div class="bg-white border border-gray-200 rounded-xl p-5 shadow-sm hover:shadow-md transition-shadow">
					<p class="text-sm font-medium text-gray-500">Policy ID</p>
					<p class="text-lg font-bold text-gray-900 mt-2 truncate">
						{decisionRecord.policyId}
					</p>
				</div>
				<div class="bg-white border border-gray-200 rounded-xl p-5 shadow-sm hover:shadow-md transition-shadow">
					<p class="text-sm font-medium text-gray-500">Proof Size</p>
					<p class="text-lg font-bold text-gray-900 mt-2">
						{proofSize} KB
					</p>
				</div>
				<div class="bg-white border border-gray-200 rounded-xl p-5 shadow-sm hover:shadow-md transition-shadow">
					<p class="text-sm font-medium text-gray-500">Procedure Code</p>
					<p class="text-lg font-bold text-gray-900 mt-2">
						{decisionRecord.code}
					</p>
				</div>
				<div class="bg-white border border-gray-200 rounded-xl p-5 shadow-sm hover:shadow-md transition-shadow">
					<p class="text-sm font-medium text-gray-500">Line of Business</p>
					<p class="text-lg font-bold text-gray-900 mt-2 uppercase">
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

