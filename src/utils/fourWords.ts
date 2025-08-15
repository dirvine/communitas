import blake3 from 'blake3'

/**
 * Generate a consistent gradient from a four-word address
 * Uses BLAKE3 hash to ensure deterministic colors
 */
export function generateFourWordGradient(fourWords: string): string {
  if (!fourWords) {
    return 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)'
  }

  // Hash the four words to get consistent colors
  const hash = blake3.hash(fourWords)
  const hashHex = Buffer.from(hash).toString('hex')

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

  const hash = blake3.hash(fourWords)
  const hashHex = Buffer.from(hash).toString('hex')

  return {
    primary: '#' + hashHex.substring(0, 6),
    secondary: '#' + hashHex.substring(6, 12),
    accent: '#' + hashHex.substring(12, 18),
  }
}

/**
 * Validate four-word address format
 */
export function isValidFourWords(fourWords: string): boolean {
  if (!fourWords) return false
  
  const words = fourWords.split('-')
  if (words.length !== 4) return false
  
  // Each word should be non-empty and contain only letters
  return words.every(word => /^[a-z]+$/.test(word))
}

/**
 * Generate initials from four-word address
 */
export function getFourWordInitials(fourWords: string): string {
  if (!fourWords) return '?'
  
  const words = fourWords.split('-').filter(Boolean)
  if (words.length === 0) return '?'
  
  return words.map(w => w[0]?.toUpperCase() || '').join('')
}

/**
 * Format four-word address for display
 */
export function formatFourWords(fourWords: string): string {
  if (!fourWords) return ''
  
  // Ensure proper formatting with hyphens
  return fourWords.toLowerCase().replace(/[^a-z-]/g, '')
}

/**
 * Generate a visual pattern from four-word address
 */
export function generateFourWordPattern(fourWords: string): string {
  if (!fourWords) return 'none'
  
  const hash = blake3.hash(fourWords)
  const hashHex = Buffer.from(hash).toString('hex')
  
  // Generate a unique pattern based on hash
  const angle = parseInt(hashHex.substring(0, 2), 16) % 360
  const size = (parseInt(hashHex.substring(2, 4), 16) % 20) + 10
  
  const color1 = '#' + hashHex.substring(4, 10) + '33'
  const color2 = '#' + hashHex.substring(10, 16) + '33'
  
  return `repeating-linear-gradient(
    ${angle}deg,
    ${color1},
    ${color1} ${size}px,
    ${color2} ${size}px,
    ${color2} ${size * 2}px
  )`
}