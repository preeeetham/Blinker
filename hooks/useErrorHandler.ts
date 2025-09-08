import { useState, useCallback } from 'react';

export interface ErrorState {
  hasError: boolean;
  message: string;
  code?: string;
}

export const useErrorHandler = () => {
  const [error, setError] = useState<ErrorState>({
    hasError: false,
    message: '',
  });

  const showError = useCallback((message: string, code?: string) => {
    setError({
      hasError: true,
      message,
      code,
    });
  }, []);

  const hideError = useCallback(() => {
    setError({
      hasError: false,
      message: '',
    });
  }, []);

  const handleAsyncError = useCallback(async <T>(
    asyncFn: () => Promise<T>,
    errorMessage: string
  ): Promise<T | null> => {
    try {
      return await asyncFn();
    } catch (err) {
      console.error('Async error:', err);
      showError(errorMessage);
      return null;
    }
  }, [showError]);

  return {
    error,
    showError,
    hideError,
    handleAsyncError,
  };
};
