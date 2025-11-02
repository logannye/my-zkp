import { json } from '@sveltejs/kit';
import { exec } from 'child_process';
import { promisify } from 'util';
import { readFile, unlink } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';
import type { RequestHandler } from './$types';

const execAsync = promisify(exec);

export const POST: RequestHandler = async ({ request }) => {
	try {
		const { patient, code } = await request.json();
		
		if (!patient || !code) {
			return json({ error: 'Missing patient or code data' }, { status: 400 });
		}
		
		// Map code to patient and policy files
		const patientFile = mapPatientFileToCode(code);
		const policyFile = mapCodeToPolicyFile(code);
		
		// Generate unique output file in temp directory
		const outputFile = join(tmpdir(), `zkp_demo_${Date.now()}.json`);
		
		// Get project root (go up from demo-ui/src/routes/api/authorize)
		const projectRoot = join(process.cwd(), '..');
		
		// Build the authz command
		const command = `cd ${projectRoot} && cargo run --quiet --release --package zk-agent --bin authz -- prove --policy policies/${policyFile} --patient patients/${patientFile} --code ${code.code} --lob commercial --out ${outputFile}`;
		
		console.log('Executing:', command);
		
		// Execute the command with a timeout
		const { stdout, stderr } = await execAsync(command, {
			timeout: 30000, // 30 second timeout
			env: {
				...process.env,
				SSZKP_BLOCKED_IFFT: '1' // Enable streaming mode
			}
		});
		
		if (stderr) {
			console.error('Stderr:', stderr);
		}
		
		// Read the decision record
		const decisionRecordJson = await readFile(outputFile, 'utf-8');
		const decisionRecord = JSON.parse(decisionRecordJson);
		
		// Clean up the temp file
		await unlink(outputFile).catch((err) => {
			console.error('Failed to delete temp file:', err);
		});
		
		return json({
			result: decisionRecord.claimed_result,
			decisionRecord
		});
	} catch (error) {
		console.error('Authorization error:', error);
		
		const errorMessage = error instanceof Error ? error.message : 'Unknown error';
		
		return json(
			{
				error: 'Failed to generate authorization proof',
				details: errorMessage
			},
			{ status: 500 }
		);
	}
};

function mapPatientFileToCode(code: any): string {
	const patientMap: Record<string, string> = {
		'71250': 'p001-approve.json',
		'19081': 'p002-needs-pa.json',
		'70551': 'p007-mri-approve.json',
		'97110': 'p012-pt-approve.json',
		'J3590': 'p014-drug-approve.json',
		'G0472': 'p011-medicare-colonoscopy.json'
	};
	
	return patientMap[code.code] || 'p001-approve.json';
}

function mapCodeToPolicyFile(code: any): string {
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

