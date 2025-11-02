<script lang="ts">
	import { CloudUpload, CheckCircle } from 'lucide-svelte';
	import Card from './ui/card.svelte';
	
	interface Props {
		onFileSelect: (file: File) => void;
		accepted?: string[];
	}
	
	let {
		onFileSelect,
		accepted = ['.pdf']
	}: Props = $props();
	
	let isDragging = $state(false);
	let selectedFile = $state<File | null>(null);
	let fileInput: HTMLInputElement;
	
	function handleDragEnter(e: DragEvent) {
		e.preventDefault();
		isDragging = true;
	}
	
	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		isDragging = false;
	}
	
	function handleDragOver(e: DragEvent) {
		e.preventDefault();
	}
	
	function handleDrop(e: DragEvent) {
		e.preventDefault();
		isDragging = false;
		
		const files = e.dataTransfer?.files;
		if (files && files.length > 0) {
			selectFile(files[0]);
		}
	}
	
	function handleFileInput(e: Event) {
		const target = e.target as HTMLInputElement;
		const files = target.files;
		if (files && files.length > 0) {
			selectFile(files[0]);
		}
	}
	
	function selectFile(file: File) {
		selectedFile = file;
		onFileSelect(file);
	}
	
	function handleClick() {
		fileInput.click();
	}
	
	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			fileInput.click();
		}
	}
</script>

<Card class="max-w-2xl mx-auto">
	<div class="space-y-6">
		<div class="text-center">
			<h2 class="text-2xl font-bold text-gray-900">Upload Patient Record</h2>
			<p class="mt-2 text-gray-600">
				Start by uploading a patient's medical record to request authorization
			</p>
		</div>
		
		<div
			class="relative border-2 border-dashed rounded-xl p-12 text-center transition-all duration-200 cursor-pointer"
			class:border-blue-400={isDragging}
			class:bg-blue-50={isDragging}
			class:shadow-lg={isDragging}
			class:scale-[1.02]={isDragging}
			class:border-gray-300={!isDragging}
			class:hover:border-gray-400={!isDragging}
			class:hover:bg-gray-50={!isDragging}
			class:hover:scale-[1.01]={!isDragging}
			ondragenter={handleDragEnter}
			ondragleave={handleDragLeave}
			ondragover={handleDragOver}
			ondrop={handleDrop}
			onclick={handleClick}
			onkeydown={handleKeyDown}
			role="button"
			tabindex="0"
		>
			<input
				bind:this={fileInput}
				type="file"
				accept={accepted.join(',')}
				onchange={handleFileInput}
				class="hidden"
			/>
			
			{#if selectedFile}
				<div class="flex flex-col items-center space-y-4 animate-in fade-in duration-300">
					<div class="relative">
						<div class="absolute inset-0 bg-green-400 rounded-full blur-xl opacity-20"></div>
						<CheckCircle class="relative w-20 h-20 text-green-500" />
					</div>
					<div>
						<p class="text-lg font-semibold text-gray-900">{selectedFile.name}</p>
						<p class="text-sm text-gray-500">
							{(selectedFile.size / 1024).toFixed(1)} KB
						</p>
					</div>
					<p class="text-sm text-green-600 font-semibold">✓ File uploaded successfully</p>
				</div>
			{:else}
				<div class="flex flex-col items-center space-y-4">
					<div class="relative">
						<div class="absolute inset-0 bg-blue-400 rounded-full blur-2xl opacity-10"></div>
						<CloudUpload class="relative w-20 h-20 text-gray-400 transition-transform duration-300 hover:scale-110" />
					</div>
					<div>
						<p class="text-lg font-semibold text-gray-900">
							Drop patient medical record PDF here
						</p>
						<p class="text-sm text-gray-500 mt-2">
							or click to browse (PDF files only)
						</p>
					</div>
				</div>
			{/if}
		</div>
		
		<div class="bg-purple-50/30 border-l-4 border-purple-500 rounded-lg p-4 shadow-sm">
			<div class="flex items-start space-x-3">
				<div class="flex-shrink-0">
					<div class="relative">
						<div class="absolute inset-0 bg-purple-400 rounded-full blur-md opacity-20"></div>
						<svg class="relative w-6 h-6 text-purple-600" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
						</svg>
					</div>
				</div>
				<div class="flex-1">
					<h3 class="text-sm font-semibold text-purple-900">Privacy Note</h3>
					<p class="text-sm text-purple-700 mt-1 leading-relaxed">
						Your patient's data will be processed locally and encrypted. Only a cryptographic proof will be shared with the payer.
					</p>
				</div>
			</div>
		</div>
	</div>
</Card>

