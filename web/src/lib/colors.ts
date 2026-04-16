export type ColorName =
	| 'rust'
	| 'mustard'
	| 'moss'
	| 'crimson'
	| 'indigo'
	| 'violet'
	| 'teal'
	| 'stone';

export const COLORS: Record<ColorName, string> = {
	rust: '#C44536',
	mustard: '#B8851E',
	moss: '#3E6B4A',
	crimson: '#9B2C2C',
	indigo: '#3949AB',
	violet: '#7B5BA6',
	teal: '#1F5F5B',
	stone: '#8E867C'
};

export const COLOR_LIST = Object.entries(COLORS) as [ColorName, string][];

export function colorHex(name: string | null | undefined): string | null {
	if (!name) return null;
	return COLORS[name as ColorName] ?? null;
}
