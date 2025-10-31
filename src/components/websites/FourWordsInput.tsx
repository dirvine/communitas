import React, { useState, useCallback } from 'react';
import { TextField, TextFieldProps } from '@mui/material';

interface FourWordsInputProps extends Omit<TextFieldProps, 'onChange' | 'value'> {
  value: string;
  onChange: (value: string) => void;
  onValidChange?: (valid: boolean) => void;
}

/**
 * Four-Words Address Input Component
 * 
 * Validates and formats four-word addresses (e.g., "ocean-forest-moon-star")
 * Auto-hyphenates and validates word count.
 */
export const FourWordsInput: React.FC<FourWordsInputProps> = ({
  value,
  onChange,
  onValidChange,
  error,
  helperText,
  ...props
}) => {
  const [localError, setLocalError] = useState<string | null>(null);

  const validateFourWords = useCallback((input: string): { valid: boolean; error: string | null } => {
    if (!input) {
      return { valid: false, error: null };
    }

    // Remove extra spaces and normalize hyphens
    const normalized = input.trim().toLowerCase().replace(/\s+/g, '-');
    const words = normalized.split('-').filter(w => w.length > 0);

    // Must have exactly 4 words
    if (words.length !== 4) {
      return { 
        valid: false, 
        error: `Expected 4 words, got ${words.length}` 
      };
    }

    // Each word should be 2-12 characters (reasonable word length)
    for (const word of words) {
      if (word.length < 2 || word.length > 12) {
        return { 
          valid: false, 
          error: `Invalid word length: "${word}" (must be 2-12 chars)` 
        };
      }

      // Only lowercase letters allowed
      if (!/^[a-z]+$/.test(word)) {
        return { 
          valid: false, 
          error: `Invalid characters in "${word}" (only a-z allowed)` 
        };
      }
    }

    return { valid: true, error: null };
  }, []);

  const handleChange = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const input = event.target.value;
    
    // Auto-hyphenate spaces
    const normalized = input.toLowerCase().replace(/\s+/g, '-');
    
    // Validate
    const { valid, error } = validateFourWords(normalized);
    setLocalError(error);
    
    // Notify parent of validation state
    if (onValidChange) {
      onValidChange(valid);
    }
    
    // Always update value (even if invalid)
    onChange(normalized);
  }, [onChange, onValidChange, validateFourWords]);

  return (
    <TextField
      {...props}
      value={value}
      onChange={handleChange}
      error={error || Boolean(localError)}
      helperText={helperText || localError || 'Enter four words separated by hyphens (e.g., ocean-forest-moon-star)'}
      placeholder="ocean-forest-moon-star"
      inputProps={{
        pattern: '[a-z-]+',
        spellCheck: false,
        autoComplete: 'off',
      }}
    />
  );
};
