import { json } from '@sveltejs/kit';
import { readFile } from 'fs/promises';
import { join } from 'path';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params }) => {
	try {
		const { code } = params;
		
		if (!code) {
			return json({ error: 'Missing CPT code' }, { status: 400 });
		}
		
		// Get project root (go up from demo-ui)
		const projectRoot = join(process.cwd(), '..');
		const policyPath = join(projectRoot, 'policies', `${code}.json`);
		
		// Read the policy file
		const policyJson = await readFile(policyPath, 'utf-8');
		const policy = JSON.parse(policyJson);
		
		return json(policy);
	} catch (error) {
		console.error('Policy fetch error:', error);
		return json(
			{ error: 'Policy not found' },
			{ status: 404 }
		);
	}
};

