import type { Code } from '$lib/types';

// Curated subset of CPT codes for demo (from 270+ available)
const SHOWCASE_CODES = [
	{ code: '71250', description: 'CT Chest without contrast' },
	{ code: '27447', description: 'Total Knee Replacement' },
	{ code: '27137', description: 'Hip Revision Surgery' },
	{ code: '97110', description: 'Physical Therapy - Therapeutic Exercise' },
	{ code: '70551', description: 'MRI Head without contrast' },
	{ code: '43235', description: 'Upper GI Endoscopy' },
	{ code: '90832', description: 'Psychotherapy - 30 minutes' },
	{ code: '92014', description: 'Comprehensive Eye Exam' },
	{ code: '95700', description: 'Electroencephalogram (EEG)' },
	{ code: '11640', description: 'Skin Lesion Excision' },
	{ code: '76536', description: 'Ultrasound - Soft Tissue Head/Neck' }
];

export async function loadAvailableCodes(): Promise<Code[]> {
	const codes: Code[] = [];
	
	for (const { code, description } of SHOWCASE_CODES) {
		try {
			// Fetch policy from API endpoint
			const response = await fetch(`/api/policies/${code}`);
			if (response.ok) {
				const policy = await response.json();
				
				// Special handling for exception-based policies
				// MRI Head has auto-approval for qualifying conditions, but we show "PA Required"
				// badge to indicate it's typically a restricted/expensive procedure
				const requiresPA = code === '70551' ? true : policy.requires_pa;
				
				codes.push({
					code: code,
					description: description,
					policyId: policy.policy_id,
					requiresPA: requiresPA
				});
			}
		} catch (err) {
			console.warn(`Failed to load policy for ${code}:`, err);
		}
	}
	
	return codes;
}

