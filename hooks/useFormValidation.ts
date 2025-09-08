import { useState, useCallback, useMemo } from 'react';
import { validateText, validateUrl, validateMintAddress, validatePercentage } from '@/lib/validation';

export interface FormField {
  value: string;
  error?: string;
  touched: boolean;
}

export interface FormState {
  [key: string]: FormField;
}

export const useFormValidation = (initialState: FormState) => {
  const [formState, setFormState] = useState<FormState>(initialState);

  const updateField = useCallback((fieldName: string, value: string) => {
    setFormState(prev => ({
      ...prev,
      [fieldName]: {
        ...prev[fieldName],
        value,
        error: undefined, // Clear error when user starts typing
      }
    }));
  }, []);

  const validateField = useCallback((fieldName: string, validationFn: (value: string) => { isValid: boolean; error?: string }) => {
    const field = formState[fieldName];
    if (!field) return;

    const validation = validationFn(field.value);
    setFormState(prev => ({
      ...prev,
      [fieldName]: {
        ...prev[fieldName],
        error: validation.isValid ? undefined : validation.error,
        touched: true,
      }
    }));
  }, [formState]);

  const validateAll = useCallback((validations: Record<string, (value: string) => { isValid: boolean; error?: string }>) => {
    const newState = { ...formState };
    let hasErrors = false;

    Object.entries(validations).forEach(([fieldName, validationFn]) => {
      const field = newState[fieldName];
      if (field) {
        const validation = validationFn(field.value);
        newState[fieldName] = {
          ...field,
          error: validation.isValid ? undefined : validation.error,
          touched: true,
        };
        if (!validation.isValid) hasErrors = true;
      }
    });

    setFormState(newState);
    return !hasErrors;
  }, [formState]);

  const resetForm = useCallback(() => {
    setFormState(initialState);
  }, [initialState]);

  const isFormValid = useMemo(() => {
    return Object.values(formState).every(field => !field.error && field.value.trim() !== '');
  }, [formState]);

  const getFieldValue = useCallback((fieldName: string) => {
    return formState[fieldName]?.value || '';
  }, [formState]);

  const getFieldError = useCallback((fieldName: string) => {
    return formState[fieldName]?.error;
  }, [formState]);

  const isFieldTouched = useCallback((fieldName: string) => {
    return formState[fieldName]?.touched || false;
  }, [formState]);

  return {
    formState,
    updateField,
    validateField,
    validateAll,
    resetForm,
    isFormValid,
    getFieldValue,
    getFieldError,
    isFieldTouched,
  };
};
