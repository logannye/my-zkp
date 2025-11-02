/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			colors: {
				primary: '#3B82F6',
				success: '#10B981',
				warning: '#F59E0B',
				danger: '#EF4444',
				privacy: '#8B5CF6'
			}
		}
	},
	plugins: []
};

