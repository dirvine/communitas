import { useState, useEffect } from 'react';

export const useTauri = () => {
  const [isTauriAvailable, setIsTauriAvailable] = useState(false);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    let attempts = 0;
    const maxAttempts = 10;
    
    const checkTauri = () => {
      // Check for various Tauri indicators
      const hasTauri = !!(
        (window as any).__TAURI__ ||
        (window as any).__TAURI_IPC__ ||
        window.location.protocol === 'tauri:' ||
        navigator.userAgent.includes('Tauri')
      );
      
      if (hasTauri) {
        console.log('Tauri detected!', {
          __TAURI__: !!(window as any).__TAURI__,
          __TAURI_IPC__: !!(window as any).__TAURI_IPC__,
          protocol: window.location.protocol,
          userAgent: navigator.userAgent
        });
        setIsTauriAvailable(true);
        setIsChecking(false);
      } else if (attempts < maxAttempts) {
        attempts++;
        console.log(`Checking for Tauri... attempt ${attempts}/${maxAttempts}`);
        setTimeout(checkTauri, 100);
      } else {
        console.log('Tauri not detected after', maxAttempts, 'attempts');
        setIsTauriAvailable(false);
        setIsChecking(false);
      }
    };

    checkTauri();
  }, []);

  return { isTauriAvailable, isChecking };
};