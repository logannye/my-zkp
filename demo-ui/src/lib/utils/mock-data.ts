import type { Code, PatientInfo } from '$lib/types';

export const availableCodes: Code[] = [
	{
		code: '71250',
		description: 'CT Chest without contrast',
		policyId: 'UHC-COMM-CT-CHEST-001',
		requiresPA: false,
		patientFile: 'p001-approve.json'
	},
	{
		code: '19081',
		description: 'Breast Biopsy',
		policyId: 'UHC-COMM-BIOPSY-001',
		requiresPA: true,
		patientFile: 'p002-needs-pa.json'
	},
	{
		code: '70551',
		description: 'MRI Head without contrast',
		policyId: 'UHC-COMM-MRI-HEAD-001',
		requiresPA: true,
		patientFile: 'p007-mri-approve.json'
	},
	{
		code: '97110',
		description: 'Physical Therapy - Therapeutic Exercise',
		policyId: 'UHC-COMM-PHYSICAL-THERAPY-001',
		requiresPA: true,
		patientFile: 'p012-pt-approve.json'
	},
	{
		code: 'J3590',
		description: 'Specialty Drug - Unclassified Biologic',
		policyId: 'UHC-COMM-SPECIALTY-DRUG-001',
		requiresPA: true,
		patientFile: 'p014-drug-approve.json'
	},
	{
		code: 'G0472',
		description: 'Screening Colonoscopy (Medicare)',
		policyId: 'UHC-MEDICARE-COLONOSCOPY-001',
		requiresPA: false,
		patientFile: 'p011-medicare-colonoscopy.json'
	}
];

export function extractPatientInfo(file: File): PatientInfo {
	// For demo purposes, extract patient info from filename
	// In production, this would parse the actual file content
	const filename = file.name;
	
	// Mock extraction based on common patterns
	let name = 'Patient';
	let dob = '1970-01-01';
	let id = 'PAT-DEMO-001';
	
	if (filename.includes('john') || filename.includes('001')) {
		name = 'John Doe';
		dob = '1970-05-15';
		id = 'PAT-DEMO-001';
	} else if (filename.includes('jane') || filename.includes('002')) {
		name = 'Jane Smith';
		dob = '1979-08-22';
		id = 'PAT-DEMO-002';
	} else if (filename.includes('007')) {
		name = 'Alice Johnson';
		dob = '1985-03-10';
		id = 'PAT-DEMO-007';
	} else {
		// Generate random mock data
		const firstNames = ['John', 'Jane', 'Michael', 'Sarah', 'David', 'Emily'];
		const lastNames = ['Doe', 'Smith', 'Johnson', 'Williams', 'Brown', 'Davis'];
		name = `${firstNames[Math.floor(Math.random() * firstNames.length)]} ${lastNames[Math.floor(Math.random() * lastNames.length)]}`;
		
		const year = 1940 + Math.floor(Math.random() * 60);
		const month = String(Math.floor(Math.random() * 12) + 1).padStart(2, '0');
		const day = String(Math.floor(Math.random() * 28) + 1).padStart(2, '0');
		dob = `${year}-${month}-${day}`;
		
		id = `PAT-DEMO-${String(Math.floor(Math.random() * 999) + 1).padStart(3, '0')}`;
	}
	
	return { name, dob, id, filename };
}

export function mapPatientFileToCode(code: Code): string {
	// Map code to appropriate patient file for demo
	return code.patientFile || 'p001-approve.json';
}

export function mapCodeToPolicyFile(code: Code): string {
	// Map code to policy file
	const policyMap: Record<string, string> = {
		'71250': 'UHC-COMM-CT-CHEST-001.json',
		'19081': 'UHC-COMM-BIOPSY-001.json',
		'70551': 'UHC-COMM-MRI-HEAD-001.json',
		'97110': 'UHC-COMM-PHYSICAL-THERAPY-001.json',
		'J3590': 'UHC-COMM-SPECIALTY-DRUG-001.json',
		'G0472': 'UHC-MEDICARE-COLONOSCOPY-001.json'
	};
	
	return policyMap[code.code] || 'UHC-COMM-CT-CHEST-001.json';
}

