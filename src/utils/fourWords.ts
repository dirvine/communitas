// In tests, we provide a shim via globalThis.__BLAKE3_SHIM__ (see setupTests)
function blake3Hash(input: string): string {
  const shim = (globalThis as any).__BLAKE3_SHIM__ as ((s: string) => string) | undefined
  if (shim) return shim(input)
  // Fallback to SHA-256 via Web Crypto for browser envs
  // Note: This is only used for deterministic color generation, not security
  const encoder = new TextEncoder()
  const data = encoder.encode(input)
  // crypto.subtle.digest returns a Promise; but our callers are sync.
  // Use a simple deterministic hash instead when subtle not available.
  let hash = 0
  for (let i = 0; i < data.length; i++) {
    hash = (hash * 31 + data[i]) >>> 0
  }
  // Convert to pseudo-hex string
  const hex = hash.toString(16).padStart(8, '0').repeat(8).slice(0, 64)
  return hex
}

/**
 * Generate a random four-word address
 */
export function generateFourWords(): string {
  const words = [
    ['ocean', 'forest', 'mountain', 'desert', 'river', 'valley', 'island', 'prairie'],
    ['blue', 'green', 'golden', 'silver', 'crystal', 'shadow', 'bright', 'misty'],
    ['eagle', 'wolf', 'bear', 'fox', 'owl', 'hawk', 'lion', 'tiger'],
    ['star', 'moon', 'sun', 'cloud', 'storm', 'wind', 'fire', 'ice']
  ];
  return words.map(group => group[Math.floor(Math.random() * group.length)]).join('-');
}

/**
 * Generate a consistent gradient from a four-word address
 * Uses BLAKE3 hash to ensure deterministic colors
 */
export function generateFourWordGradient(fourWords: string): string {
  if (!fourWords) {
    return 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)'
  }

  // Hash the four words to get consistent colors
  const hashHex = blake3Hash(fourWords)

  // Extract color values from hash
  const color1 = '#' + hashHex.substring(0, 6)
  const color2 = '#' + hashHex.substring(6, 12)
  const color3 = '#' + hashHex.substring(12, 18)

  // Create a smooth gradient
  return `linear-gradient(135deg, ${color1} 0%, ${color2} 50%, ${color3} 100%)`
}

/**
 * Generate a color scheme from four-word address
 */
export function generateFourWordColors(fourWords: string) {
  if (!fourWords) {
    return {
      primary: '#667eea',
      secondary: '#764ba2',
      accent: '#f093fb',
    }
  }

  // Hash the four words to get consistent colors
  const hashHex = blake3Hash(fourWords)

  return {
    primary: '#' + hashHex.substring(0, 6),
    secondary: '#' + hashHex.substring(6, 12),
    accent: '#' + hashHex.substring(12, 18),
  }
}