import { describe, it, expect } from 'vitest'
import { inputSanitizer } from '../inputSanitization'

describe('InputSanitizationService', () => {
  it('sanitizes markdown links and flags invalid URLs', () => {
    const md = '[bad](javascript:alert(1)) and [also bad](data:text/html,<script>) and [ok](https://example.com)'
    const res = inputSanitizer.sanitizeMarkdown(md)
    expect(res.isValid).toBe(true)
    expect(res.warnings.some(w => w.includes('Invalid URL'))).toBe(true)
    expect(res.sanitizedValue).toContain('#invalid-url')
  })

  it('removes script tags and inline handlers from HTML in markdown', () => {
    const md = '<h1 onclick="alert(1)">Title</h1> <script>alert(2)</script>'
    const res = inputSanitizer.sanitizeMarkdown(md)
    expect(res.isValid).toBe(true)
    expect(res.sanitizedValue).not.toContain('onclick')
    expect(res.sanitizedValue).not.toContain('<script')
  })

  it('flags file: and data: protocols in links', () => {
    const md = '[file](file:///etc/passwd) [data](data:text/html,<b>bad</b>)'
    const res = inputSanitizer.sanitizeMarkdown(md)
    const normalized = res.warnings.map(w => w.toLowerCase())
    expect(normalized.some(w => w.includes('removed file url'))).toBe(true)
    expect(normalized.some(w => w.includes('removed data url'))).toBe(true)
    expect(res.sanitizedValue).toContain('](#invalid-url)')
  })

  it('enforces markdown length limit', () => {
    const res = inputSanitizer.sanitizeMarkdown('a'.repeat(50001))
    expect(res.isValid).toBe(false)
    expect(res.errors[0]).toContain('maximum length')
    expect(res.sanitizedValue.length).toBe(50000)
  })

  it('sanitizes network identity format', () => {
    const res = inputSanitizer.sanitizeNetworkIdentity('A-b@-c-d111111111111')
    expect(res.sanitizedValue).toMatch(/^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/)
    expect(res.isValid).toBe(false) // includes a too-short/invalid word warning/error
  })

  it('sanitizes file path and prevents traversal', () => {
    const bad = inputSanitizer.sanitizeFilePath('../etc/passwd')
    expect(bad.isValid).toBe(false)

    const fixed = inputSanitizer.sanitizeFilePath('docs/guide.md')
    expect(fixed.isValid).toBe(true)
    expect(fixed.warnings.some(w => w.includes('/web/'))).toBe(true)
    expect(fixed.sanitizedValue.startsWith('/web/')).toBe(true)
  })

  it('sanitizes search query and escapes regex characters', () => {
    const res = inputSanitizer.sanitizeSearchQuery('find (this)|that?')
    expect(res.isValid).toBe(true)
    expect(res.sanitizedValue).toContain('find')
    // ensure parentheses and pipes escaped
    expect(res.sanitizedValue).toContain('\\(')
    expect(res.sanitizedValue).toContain('\\)')
    expect(res.sanitizedValue).toContain('\\|')
    expect(res.sanitizedValue).toContain('\\?')
  })
})
