<script lang="ts">
	import { Loader, FileText, Sparkles } from 'lucide-svelte';
	import Card from './ui/card.svelte';
	
	interface Props {
		filename: string;
	}
	
	let { filename }: Props = $props();
</script>

<Card class="max-w-2xl mx-auto">
	<div class="space-y-8 text-center">
		<!-- Header -->
		<div>
			<h2 class="text-2xl font-bold text-gray-900">Processing Patient Record</h2>
			<p class="mt-2 text-gray-600">
				Extracting medical data from document...
			</p>
		</div>
		
		<!-- File Info -->
		<div class="flex items-center justify-center space-x-3 text-gray-700">
			<FileText class="w-5 h-5 text-blue-600" />
			<span class="font-medium">{filename}</span>
		</div>
		
		<!-- Animated Spinner -->
		<div class="flex justify-center py-6">
			<div class="relative">
				<div class="absolute inset-0 bg-gradient-to-br from-blue-400 to-indigo-500 rounded-full blur-2xl opacity-20 animate-pulse"></div>
				<div class="relative">
					<Loader class="w-16 h-16 text-primary animate-spin" />
				</div>
			</div>
		</div>
		
		<!-- Processing Steps Info -->
		<div class="bg-blue-50/30 border-l-4 border-blue-500 rounded-lg p-5 shadow-sm text-left">
			<div class="flex items-start space-x-3">
				<div class="relative mt-0.5">
					<div class="absolute inset-0 bg-blue-400 rounded-full blur-md opacity-20"></div>
					<Sparkles class="relative w-6 h-6 text-blue-600" />
				</div>
				<div class="flex-1">
					<h3 class="text-sm font-semibold text-blue-900 mb-2">What's Happening?</h3>
					<ul class="text-sm text-blue-700 space-y-1.5">
						<li class="flex items-center space-x-2">
							<span class="text-blue-500">•</span>
							<span>Reading patient demographics and medical history</span>
						</li>
						<li class="flex items-center space-x-2">
							<span class="text-blue-500">•</span>
							<span>Extracting diagnosis codes (ICD-10)</span>
						</li>
						<li class="flex items-center space-x-2">
							<span class="text-blue-500">•</span>
							<span>Parsing treatment information and service location</span>
						</li>
						<li class="flex items-center space-x-2">
							<span class="text-blue-500">•</span>
							<span>Structuring data for privacy-preserving verification</span>
						</li>
					</ul>
				</div>
			</div>
		</div>
		
		<!-- Technical Note -->
		<div class="text-xs text-gray-500 italic">
			In production, this step uses LLM-powered extraction to parse unstructured medical documents
		</div>
	</div>
</Card>

