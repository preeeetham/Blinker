// Input validation and sanitization utilities

export interface ValidationResult {
  isValid: boolean;
  error?: string;
}

export const validateUrl = (url: string): ValidationResult => {
  if (!url.trim()) {
    return { isValid: false, error: 'URL is required' };
  }

  try {
    const urlObj = new URL(url);
    if (!['http:', 'https:'].includes(urlObj.protocol)) {
      return { isValid: false, error: 'Only HTTP and HTTPS URLs are allowed' };
    }
    return { isValid: true };
  } catch {
    return { isValid: false, error: 'Invalid URL format' };
  }
};

export const validateText = (text: string, maxLength: number, fieldName: string): ValidationResult => {
  if (!text.trim()) {
    return { isValid: false, error: `${fieldName} is required` };
  }

  if (text.length > maxLength) {
    return { isValid: false, error: `${fieldName} must be ${maxLength} characters or less` };
  }

  // Check for potentially malicious content
  const dangerousPatterns = [
    /<script/i,
    /javascript:/i,
    /on\w+\s*=/i,
    /<iframe/i,
    /<object/i,
    /<embed/i
  ];

  for (const pattern of dangerousPatterns) {
    if (pattern.test(text)) {
      return { isValid: false, error: `${fieldName} contains invalid content` };
    }
  }

  return { isValid: true };
};

export const validateMintAddress = (mint: string): ValidationResult => {
  if (!mint.trim()) {
    return { isValid: false, error: 'Mint address is required' };
  }

  // Basic Solana address validation (44 characters, base58)
  const base58Regex = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;
  if (!base58Regex.test(mint)) {
    return { isValid: false, error: 'Invalid mint address format' };
  }

  return { isValid: true };
};

export const validatePercentage = (percentage: number): ValidationResult => {
  if (isNaN(percentage) || percentage < 0 || percentage > 1) {
    return { isValid: false, error: 'Percentage must be between 0 and 1' };
  }

  return { isValid: true };
};

export const sanitizeInput = (input: string): string => {
  return input
    .trim()
    .replace(/[<>]/g, '') // Remove potential HTML tags
    .replace(/javascript:/gi, '') // Remove javascript: protocol
    .replace(/on\w+\s*=/gi, ''); // Remove event handlers
};
