import DOMPurify from 'dompurify'
import validator from 'validator'

/**
 * Comprehensive input sanitization and validation service
 * Addresses critical security vulnerabilities in user input processing
 */

export interface SanitizationOptions {
  allowedTags?: string[]
  allowedAttributes?: Record<string, string[]>
  maxLength?: number
  stripHTML?: boolean
  escapeHTML?: boolean
}

export interface ValidationResult {
  isValid: boolean
  sanitizedValue: string
  errors: string[]
  warnings: string[]
}

export class InputSanitizationService {
  private static instance: InputSanitizationService
  
  // Secure configuration for DOMPurify
  private readonly purifyConfig = {
    ALLOWED_TAGS: [
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'p', 'br', 'strong', 'em', 'u', 's',
      'ul', 'ol', 'li',
      'blockquote', 'pre', 'code',
      'a', 'img',
      'table', 'thead', 'tbody', 'tr', 'th', 'td'
    ],
    ALLOWED_ATTR: [
      'href', 'title', 'alt', 'src',
      'class', 'id',
      'colspan', 'rowspan'
    ],
    ALLOW_DATA_ATTR: false,
    FORBID_TAGS: ['script', 'object', 'embed', 'form', 'input', 'iframe'],
    FORBID_ATTR: ['onerror', 'onclick', 'onload', 'onmouseover', 'style'],
    USE_PROFILES: { html: true }
  }

  static getInstance(): InputSanitizationService {
    if (!InputSanitizationService.instance) {
      InputSanitizationService.instance = new InputSanitizationService()
    }
    return InputSanitizationService.instance
  }

  /**
   * Sanitize markdown content for safe processing
   * Critical: Prevents XSS through markdown injection
   */
  sanitizeMarkdown(content: string, options: SanitizationOptions = {}): ValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    // Input validation
    if (typeof content !== 'string') {
      errors.push('Content must be a string')
      return { isValid: false, sanitizedValue: '', errors, warnings }
    }

    // Length validation
    const maxLength = options.maxLength || 50000 // 50KB max
    if (content.length > maxLength) {
      errors.push(`Content exceeds maximum length of ${maxLength} characters`)
      return { isValid: false, sanitizedValue: content.substring(0, maxLength), errors, warnings }
    }

    let sanitized = content

    // Remove potentially dangerous markdown patterns
    sanitized = this.removeDangerousMarkdownPatterns(sanitized, warnings)

    // Sanitize any embedded HTML
    sanitized = this.sanitizeHTML(sanitized)

    // Validate URLs in markdown links
    sanitized = this.sanitizeMarkdownLinks(sanitized, warnings)

    // Additional security checks
    this.performSecurityChecks(sanitized, warnings)

    return {
      isValid: errors.length === 0,
      sanitizedValue: sanitized,
      errors,
      warnings
    }
  }

  /**
   * Sanitize HTML content using DOMPurify
   */
  sanitizeHTML(html: string, options: SanitizationOptions = {}): string {
    if (typeof html !== 'string') return ''

    const config = {
      ...this.purifyConfig,
      ...(options.allowedTags && { ALLOWED_TAGS: options.allowedTags }),
      ...(options.allowedAttributes && { ALLOWED_ATTR: Object.keys(options.allowedAttributes) })
    }

    return DOMPurify.sanitize(html, config)
  }

  /**
   * Validate and sanitize network identity (four-word addresses)
   */
  sanitizeNetworkIdentity(identity: string): ValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    if (!identity || typeof identity !== 'string') {
      errors.push('Network identity must be a non-empty string')
      return { isValid: false, sanitizedValue: '', errors, warnings }
    }

    // Four-word format validation
    const sanitized = identity.toLowerCase().trim()
    const words = sanitized.split('-')

    if (words.length !== 4) {
      errors.push('Network identity must be exactly four words separated by hyphens')
      return { isValid: false, sanitizedValue: sanitized, errors, warnings }
    }

    // Validate each word
    const validatedWords = words.map(word => {
      // Remove any non-alphabetic characters
      const cleanWord = word.replace(/[^a-z]/g, '')
      
      if (cleanWord.length < 2) {
        errors.push(`Word "${word}" is too short (minimum 2 characters)`)
      }
      
      if (cleanWord.length > 12) {
        warnings.push(`Word "${word}" is long (${cleanWord.length} characters)`)
      }

      return cleanWord
    })

    const result = validatedWords.join('-')

    return {
      isValid: errors.length === 0,
      sanitizedValue: result,
      errors,
      warnings
    }
  }

  /**
   * Validate file paths to prevent directory traversal
   */
  sanitizeFilePath(path: string): ValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    if (!path || typeof path !== 'string') {
      errors.push('File path must be a non-empty string')
      return { isValid: false, sanitizedValue: '', errors, warnings }
    }

    let sanitized = path.trim()

    // Prevent directory traversal
    if (sanitized.includes('..') || sanitized.includes('//')) {
      errors.push('File path contains directory traversal patterns')
      return { isValid: false, sanitizedValue: '', errors, warnings }
    }

    // Ensure path starts with /web/ for markdown files
    if (!sanitized.startsWith('/web/') && sanitized.endsWith('.md')) {
      warnings.push('Markdown files should be in /web/ directory')
      sanitized = `/web/${sanitized.replace(/^\/+/, '')}`
    }

    // Validate file extension
    const allowedExtensions = ['.md', '.html', '.css', '.js', '.json', '.png', '.jpg', '.jpeg', '.gif', '.svg']
    const extension = sanitized.toLowerCase().substring(sanitized.lastIndexOf('.'))
    
    if (!allowedExtensions.includes(extension)) {
      errors.push(`File extension "${extension}" is not allowed`)
    }

    // Remove potentially dangerous characters
    sanitized = sanitized.replace(/[<>:"|?*]/g, '')

    return {
      isValid: errors.length === 0,
      sanitizedValue: sanitized,
      errors,
      warnings
    }
  }

  /**
   * Sanitize user input for search queries
   */
  sanitizeSearchQuery(query: string): ValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    if (!query || typeof query !== 'string') {
      errors.push('Search query must be a non-empty string')
      return { isValid: false, sanitizedValue: '', errors, warnings }
    }

    let sanitized = query.trim()

    // Prevent injection attacks
    const dangerousPatterns = [
      /javascript:/gi,
      /vbscript:/gi,
      /on\w+\s*=/gi,
      /<script/gi,
      /eval\(/gi,
      /expression\(/gi
    ]

    for (const pattern of dangerousPatterns) {
      if (pattern.test(sanitized)) {
        errors.push('Search query contains potentially dangerous content')
        sanitized = sanitized.replace(pattern, '')
      }
    }

    // Length limits
    if (sanitized.length > 200) {
      warnings.push('Search query is very long')
      sanitized = sanitized.substring(0, 200)
    }

    // Escape special regex characters if using for search
    sanitized = sanitized.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

    return {
      isValid: errors.length === 0,
      sanitizedValue: sanitized,
      errors,
      warnings
    }
  }

  private removeDangerousMarkdownPatterns(content: string, warnings: string[]): string {
    let sanitized = content

    // Remove dangerous markdown patterns
    const dangerousPatterns = [
      // JavaScript URLs
      { pattern: /\[([^\]]*)\]\(javascript:[^)]*\)/gi, replacement: '[$1](#)', warning: 'Removed JavaScript URL' },
      
      // Data URLs (can contain scripts)
      { pattern: /\[([^\]]*)\]\(data:[^)]*\)/gi, replacement: '[$1](#)', warning: 'Removed data URL' },
      
      // File URLs
      { pattern: /\[([^\]]*)\]\(file:[^)]*\)/gi, replacement: '[$1](#)', warning: 'Removed file URL' },
      
      // HTML with script tags
      { pattern: /<script[^>]*>.*?<\/script>/gis, replacement: '', warning: 'Removed script tag' },
      
      // Event handlers in markdown HTML
      { pattern: /\s*on\w+\s*=\s*['""][^'"]*['"]/gi, replacement: '', warning: 'Removed event handler' }
    ]

    for (const { pattern, replacement, warning } of dangerousPatterns) {
      if (pattern.test(sanitized)) {
        warnings.push(warning)
        sanitized = sanitized.replace(pattern, replacement)
      }
    }

    return sanitized
  }

  private sanitizeMarkdownLinks(content: string, warnings: string[]): string {
    return content.replace(/\[([^\]]*)\]\(([^)]*)\)/g, (match, text, url) => {
      // Validate URL
      if (!this.isValidURL(url)) {
        warnings.push(`Invalid URL found: ${url}`)
        return `[${text}](#invalid-url)`
      }

      // Check for allowed protocols
      const allowedProtocols = ['http:', 'https:', 'mailto:', 'dht:']
      const urlObj = new URL(url, 'https://example.com') // Fallback base for relative URLs
      
      if (!allowedProtocols.includes(urlObj.protocol)) {
        warnings.push(`Disallowed protocol: ${urlObj.protocol}`)
        return `[${text}](#disallowed-protocol)`
      }

      return match
    })
  }

  private isValidURL(url: string): boolean {
    try {
      // Handle relative URLs and four-word identities
      if (url.startsWith('/') || url.match(/^[a-z]+-[a-z]+-[a-z]+-[a-z]+/)) {
        return true
      }
      
      // Validate absolute URLs
      return validator.isURL(url, {
        protocols: ['http', 'https', 'mailto', 'dht'],
        require_protocol: true,
        allow_underscores: true
      })
    } catch {
      return false
    }
  }

  private performSecurityChecks(content: string, warnings: string[]): void {
    // Check for potential XSS vectors
    const xssPatterns = [
      /javascript:/gi,
      /vbscript:/gi,
      /data:text\/html/gi,
      /<iframe/gi,
      /<object/gi,
      /<embed/gi
    ]

    for (const pattern of xssPatterns) {
      if (pattern.test(content)) {
        warnings.push('Content contains potential XSS vector')
      }
    }

    // Check for excessively long lines (potential DoS)
    const lines = content.split('\n')
    const maxLineLength = 1000
    
    lines.forEach((line, index) => {
      if (line.length > maxLineLength) {
        warnings.push(`Line ${index + 1} is excessively long (${line.length} characters)`)
      }
    })

    // Check for excessive nesting (potential ReDoS)
    const nestingLevel = (content.match(/[([{]/g) || []).length - (content.match(/[)\]}]/g) || []).length
    if (Math.abs(nestingLevel) > 50) {
      warnings.push('Content has excessive bracket nesting')
    }
  }
}

// Export singleton instance
export const inputSanitizer = InputSanitizationService.getInstance()