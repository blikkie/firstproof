/**
 * Developer & Diagnostics Mode Utilities
 *
 * In accordance with First Proof anti-slop and restraint principles, diagnostic
 * and internal parity tools (such as ParityModal and Parity Audit buttons)
 * must be quarantined behind developer mode so that production interfaces
 * remain focused purely on photographic craft.
 *
 * Rules:
 * - Active during Vite development mode (`npm run dev` -> `import.meta.env.DEV === true`).
 * - Active in production if the URL query string contains `?debug=1` or `?dev=1`.
 */

export function isDebugMode(): boolean {
  if (typeof window === 'undefined') return false;
  if (import.meta.env.DEV) return true;
  const params = new URLSearchParams(window.location.search);
  return params.has('debug') || params.has('dev');
}
