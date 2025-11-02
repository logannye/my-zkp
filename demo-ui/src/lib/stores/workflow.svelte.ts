import type { WorkflowStep, Code, PatientInfo, AuthorizationResult, DecisionRecord } from '$lib/types';

class WorkflowState {
	step = $state<WorkflowStep>('upload');
	
	// Data
	uploadedFile = $state<File | null>(null);
	selectedCode = $state<Code | null>(null);
	patientInfo = $state<PatientInfo | null>(null);
	
	// Results
	authorizationResult = $state<AuthorizationResult | null>(null);
	decisionRecord = $state<DecisionRecord | null>(null);
	
	// Status
	isProcessing = $state(false);
	isProcessingUpload = $state(false);
	error = $state<string | null>(null);
	
	// Actions
	async uploadFile(file: File) {
		this.uploadedFile = file;
		this.error = null;
		this.isProcessingUpload = true;
		
		try {
			// Simulate processing time for LLM-powered data extraction (2.5 seconds)
			await new Promise(resolve => setTimeout(resolve, 2500));
			
			// Extract patient ID from filename (e.g., "PAT001.pdf" → "PAT001")
			const patientId = file.name.replace('.pdf', '');
			
			// Fetch the pre-parsed patient JSON
			const response = await fetch(`/patients/${patientId}.json`);
			if (!response.ok) {
				throw new Error(`Patient data not found for ${patientId}`);
			}
			
			const patientData = await response.json();
			
			// Transform to PatientInfo format
			this.patientInfo = {
				name: `Patient ${patientData.patient_id}`,
				dob: patientData.dob,
				id: patientData.patient_id,
				filename: file.name,
				rawData: patientData
			};
			
			this.isProcessingUpload = false;
			this.step = 'select';
		} catch (err) {
			this.isProcessingUpload = false;
			this.error = err instanceof Error ? err.message : 'Failed to load patient data';
			this.step = 'upload';
		}
	}
	
	selectCode(code: Code) {
		this.selectedCode = code;
		this.step = 'review';
		this.error = null;
	}
	
	goBack() {
		if (this.step === 'select') {
			this.step = 'upload';
		} else if (this.step === 'review') {
			this.step = 'select';
		}
		this.error = null;
	}
	
	async submitAuthorization() {
		this.step = 'processing';
		this.isProcessing = true;
		this.error = null;
		
		try {
			const response = await fetch('/api/authorize', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					patient: this.patientInfo,
					code: this.selectedCode
				})
			});
			
			if (!response.ok) {
				const errorData = await response.json();
				throw new Error(errorData.error || 'Authorization failed');
			}
			
			const data = await response.json();
			
			this.authorizationResult = data.result;
			this.decisionRecord = data.decisionRecord;
			this.step = 'results';
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Unknown error occurred';
			this.step = 'review'; // Go back to review on error
		} finally {
			this.isProcessing = false;
		}
	}
	
	reset() {
		this.step = 'upload';
		this.uploadedFile = null;
		this.selectedCode = null;
		this.patientInfo = null;
		this.authorizationResult = null;
		this.decisionRecord = null;
		this.error = null;
		this.isProcessing = false;
	}
}

export const workflow = new WorkflowState();

