import type { WorkflowStep, Code, PatientInfo, AuthorizationResult, DecisionRecord } from '$lib/types';
import { extractPatientInfo } from '$lib/utils/mock-data';

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
	error = $state<string | null>(null);
	
	// Actions
	uploadFile(file: File) {
		this.uploadedFile = file;
		this.patientInfo = extractPatientInfo(file);
		this.step = 'select';
		this.error = null;
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

