<script lang="ts">
	import { ArrowLeft, Shield, User, FileText, Building2 } from 'lucide-svelte';
	import Card from './ui/card.svelte';
	import Button from './ui/button.svelte';
	import Badge from './ui/badge.svelte';
	import type { PatientInfo, Code } from '$lib/types';
	
	interface Props {
		patientInfo: PatientInfo | null;
		selectedCode: Code | null;
		onSubmit: () => void;
		onBack: () => void;
	}
	
	let {
		patientInfo,
		selectedCode,
		onSubmit,
		onBack
	}: Props = $props();
</script>

<Card class="max-w-2xl mx-auto">
	<div class="space-y-6">
		<div class="flex items-center space-x-4">
			<Button variant="ghost" size="sm" onclick={onBack}>
				<ArrowLeft class="w-4 h-4 mr-2" />
				Back
			</Button>
			<div class="flex-1">
				<h2 class="text-2xl font-bold text-gray-900">Review Authorization Request</h2>
				<p class="mt-1 text-gray-600">
					Confirm the details before submitting
				</p>
			</div>
		</div>
		
		{#if patientInfo && selectedCode}
			<div class="space-y-4">
				<!-- Patient Information -->
				<div class="bg-gray-50 rounded-lg p-4">
					<div class="flex items-center space-x-2 mb-3">
						<User class="w-5 h-5 text-gray-700" />
						<h3 class="font-semibold text-gray-900">Patient Information</h3>
					</div>
					<div class="grid grid-cols-2 gap-3 text-sm">
						<div>
							<span class="text-gray-500">Name:</span>
							<span class="ml-2 font-medium text-gray-900">{patientInfo.name}</span>
						</div>
						<div>
							<span class="text-gray-500">DOB:</span>
							<span class="ml-2 font-medium text-gray-900">{patientInfo.dob}</span>
						</div>
						<div>
							<span class="text-gray-500">Patient ID:</span>
							<span class="ml-2 font-medium text-gray-900">{patientInfo.id}</span>
						</div>
						<div>
							<span class="text-gray-500">File:</span>
							<span class="ml-2 font-medium text-gray-900 truncate">{patientInfo.filename}</span>
						</div>
					</div>
					<p class="text-xs text-gray-500 mt-3 italic">
						Note: Displayed locally only. This data will not be shared.
					</p>
				</div>
				
				<!-- Procedure Information -->
				<div class="bg-gray-50 rounded-lg p-4">
					<div class="flex items-center space-x-2 mb-3">
						<FileText class="w-5 h-5 text-gray-700" />
						<h3 class="font-semibold text-gray-900">Requested Procedure</h3>
					</div>
					<div class="space-y-2">
						<div class="flex items-center space-x-3">
							<span class="text-2xl font-bold text-gray-900">{selectedCode.code}</span>
							{#if selectedCode.requiresPA}
								<Badge variant="warning">PA Required</Badge>
							{:else}
								<Badge variant="success">Auto-Approve</Badge>
							{/if}
						</div>
						<p class="text-gray-700">{selectedCode.description}</p>
					</div>
				</div>
				
				<!-- Policy Information -->
				<div class="bg-gray-50 rounded-lg p-4">
					<div class="flex items-center space-x-2 mb-3">
						<Building2 class="w-5 h-5 text-gray-700" />
						<h3 class="font-semibold text-gray-900">Applicable Policy</h3>
					</div>
					<div class="text-sm">
						<div>
							<span class="text-gray-500">Policy ID:</span>
							<span class="ml-2 font-medium text-gray-900">{selectedCode.policyId}</span>
						</div>
						<div class="mt-2">
							<span class="text-gray-500">Authorization Type:</span>
							<span class="ml-2 font-medium text-gray-900">
								{selectedCode.requiresPA ? 'Prior Authorization Required' : 'Auto-Approve'}
							</span>
						</div>
					</div>
				</div>
			</div>
			
			<!-- Privacy Guarantee -->
			<div class="bg-purple-50 border-2 border-purple-200 rounded-lg p-4">
				<div class="flex items-start space-x-3">
					<Shield class="w-6 h-6 text-privacy flex-shrink-0 mt-0.5" />
					<div class="flex-1">
						<h3 class="text-lg font-bold text-purple-900">🔒 Privacy Guarantee</h3>
						<p class="text-sm text-purple-700 mt-2">
							Patient data will be encrypted locally. Only a zero-knowledge cryptographic proof will be shared with the payer. No personal health information leaves your system.
						</p>
						<ul class="mt-3 space-y-1 text-sm text-purple-700">
							<li>✓ HIPAA-compliant by design</li>
							<li>✓ Proof size: ~2KB (vs. full medical records)</li>
							<li>✓ Instant verification by payer</li>
						</ul>
					</div>
				</div>
			</div>
			
			<!-- Action Buttons -->
			<div class="flex space-x-3 pt-4">
				<Button variant="outline" onclick={onBack} class="flex-1">
					Cancel
				</Button>
				<Button variant="default" onclick={onSubmit} class="flex-1">
					Submit Authorization Request
				</Button>
			</div>
		{/if}
	</div>
</Card>

