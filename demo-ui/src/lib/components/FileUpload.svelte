<script lang="ts">
	import { CloudUpload, CheckCircle } from 'lucide-svelte';
	import Card from './ui/card.svelte';
	
	interface Props {
		onFileSelect: (file: File) => void;
		accepted?: string[];
	}
	
	let {
		onFileSelect,
		accepted = ['.pdf', '.json', '.txt']
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
			class="relative border-2 border-dashed rounded-lg p-12 text-center transition-all cursor-pointer hover:border-primary hover:bg-blue-50/50"
			class:border-primary={isDragging}
			class:bg-blue-50={isDragging}
			class:border-gray-300={!isDragging}
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
				<div class="flex flex-col items-center space-y-3">
					<CheckCircle class="w-16 h-16 text-success" />
					<div>
						<p class="text-lg font-semibold text-gray-900">{selectedFile.name}</p>
						<p class="text-sm text-gray-500">
							{(selectedFile.size / 1024).toFixed(1)} KB
						</p>
					</div>
					<p class="text-sm text-success font-medium">File uploaded successfully</p>
				</div>
			{:else}
				<div class="flex flex-col items-center space-y-3">
					<CloudUpload class="w-16 h-16 text-gray-400" />
					<div>
						<p class="text-lg font-semibold text-gray-900">
							Drag patient record here or click to browse
						</p>
						<p class="text-sm text-gray-500 mt-1">
							Supports PDF, JSON, TXT (demo)
						</p>
					</div>
				</div>
			{/if}
		</div>
		
		<div class="bg-purple-50 border border-purple-200 rounded-lg p-4">
			<div class="flex items-start space-x-3">
				<div class="flex-shrink-0">
					<svg class="w-5 h-5 text-privacy" fill="currentColor" viewBox="0 0 20 20">
						<path fill-rule="evenodd" d="M10 1.944A11.954 11.954 0 012.166 5C2.056 5.649 2 6.319 2 7c0 5.225 3.34 9.67 8 11.317C14.66 16.67 18 12.225 18 7c0-.682-.057-1.35-.166-2.001A11.954 11.954 0 0110 1.944zM11 14a1 1 0 11-2 0 1 1 0 012 0zm0-7a1 1 0 10-2 0v3a1 1 0 102 0V7z" clip-rule="evenodd" />
					</svg>
				</div>
				<div class="flex-1">
					<h3 class="text-sm font-semibold text-purple-900">Privacy Note</h3>
					<p class="text-sm text-purple-700 mt-1">
						Your patient's data will be processed locally and encrypted. Only a cryptographic proof will be shared with the payer.
					</p>
				</div>
			</div>
		</div>
	</div>
</Card>

