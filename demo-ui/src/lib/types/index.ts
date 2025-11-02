export type AuthorizationResult = 'APPROVE' | 'NEEDS_PA' | 'DENY';

export interface Code {
	code: string;
	description: string;
	policyId: string;
	requiresPA: boolean;
	patientFile?: string;
}

export interface PatientInfo {
	name: string;
	dob: string;
	id: string;
	filename: string;
}

export interface DecisionRecord {
	policyId: string;
	policyHash: string;
	patientCommitment: string;
	claimedResult: string;
	proof: string;
	code: string;
	lob: string;
}

export type WorkflowStep = 'upload' | 'select' | 'review' | 'processing' | 'results';

