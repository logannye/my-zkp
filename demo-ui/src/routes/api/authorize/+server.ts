import { json } from '@sveltejs/kit';
import { exec } from 'child_process';
import { promisify } from 'util';
import { readFile, writeFile, unlink } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';
import type { RequestHandler } from './$types';

const execAsync = promisify(exec);

function toCamelCase(snakeCaseObj: any): any {
	return {
		policyId: snakeCaseObj.policy_id,
		policyHash: snakeCaseObj.policy_hash,
		patientCommitment: snakeCaseObj.patient_commitment,
		claimedResult: snakeCaseObj.claimed_result,
		code: snakeCaseObj.code,
		lob: snakeCaseObj.lob,
		proof: snakeCaseObj.proof
	};
}

export const POST: RequestHandler = async ({ request }) => {
	try {
		const { patient, code } = await request.json();
		
		if (!patient || !code || !patient.rawData) {
			return json({ error: 'Missing patient or code data' }, { status: 400 });
		}
		
		// Use the CPT code directly for policy mapping
		const policyFile = `${code.code}.json`;
		
		// Create temp file for patient data
		const patientTempFile = join(tmpdir(), `patient_${Date.now()}.json`);
		await writeFile(patientTempFile, JSON.stringify(patient.rawData, null, 2));
		
		// Generate unique output file in temp directory
		const outputFile = join(tmpdir(), `zkp_demo_${Date.now()}.json`);
		
		// Get project root (go up from demo-ui)
		const projectRoot = join(process.cwd(), '..');
		
		// Build the authz command - use temp patient file
		const command = `cd ${projectRoot} && cargo run --quiet --release --package zk-agent --bin authz -- prove --policy policies/${policyFile} --patient ${patientTempFile} --code ${code.code} --lob commercial --out ${outputFile}`;
		
		console.log('Executing:', command);
		
		// Execute with timeout
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
		
		// Transform snake_case to camelCase
		const transformedRecord = toCamelCase(decisionRecord);
		
		// Clean up temp files
		await Promise.all([
			unlink(outputFile).catch(console.error),
			unlink(patientTempFile).catch(console.error)
		]);
		
		return json({
			result: transformedRecord.claimedResult,
			decisionRecord: transformedRecord
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
