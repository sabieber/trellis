export type Colorway = 'moss' | 'clay' | 'ink' | 'plum' | 'gold' | 'char' | 'sage' | 'rust' | 'teal' | 'navy';

export const COLORWAYS: Colorway[] = ['moss', 'clay', 'ink', 'plum', 'gold', 'char', 'sage', 'rust', 'teal', 'navy'];

export const COLORWAY_COLORS: Record<Colorway, { bg: string; text: string }> = {
  moss: {bg: '#33402a', text: '#e7e2c8'},
  clay: {bg: '#5a3b2c', text: '#f1e2d0'},
  ink: {bg: '#23314a', text: '#e3e7f0'},
  plum: {bg: '#412a3e', text: '#efdfe9'},
  gold: {bg: '#5c4a23', text: '#f6eccf'},
  char: {bg: '#262420', text: '#e7ddc8'},
  sage: {bg: '#3c4636', text: '#e9ecd9'},
  rust: {bg: '#532a23', text: '#f3ddcf'},
  teal: {bg: '#1f3d3a', text: '#dcece6'},
  navy: {bg: '#1d2740', text: '#dfe3ee'},
};

export function colorwayForTitle(title: string, explicit?: Colorway | ''): Colorway {
  if (explicit) return explicit;
  let h = 0;
  for (const ch of title) h = (h * 31 + ch.charCodeAt(0)) | 0;
  return COLORWAYS[Math.abs(h) % COLORWAYS.length];
}

export function spineWidthForTitle(title: string): number {
  let h = 0;
  for (const ch of title) h = (h * 37 + ch.charCodeAt(0)) | 0;
  return 26 + (Math.abs(h) % 17);
}

export function spineHeightOffset(title: string): number {
  let h = 0;
  for (const ch of title) h = (h * 41 + ch.charCodeAt(0)) | 0;
  const lenBias = Math.min(title.length, 40) * 0.3;
  return -20 + (Math.abs(h) % 26) + lenBias;
}
