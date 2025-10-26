import { useEffect, useState } from 'react';

export const useTauri = () => {
  const [isTauriAvailable, setIsTauriAvailable] = useState(false);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    let attempts = 0;
    const maxAttempts = 10;
    
    const checkTauri = () => {
      // Check for various Tauri indicators
      const hasTauri = !!(
        (window as any)._TAURI_ ||
        (window as any)._TAURI_IPC_ ||
        window.location.protocol === 'tauri:' ||
        navigator.userAgent.includes('Tauri')
      );
      
      if (hasTauri) {
        console.log('Tauri detected!', {
          _TAURI_: !!(window as any)._TAURI_,
          _TAURI_IPC_: !!(window as any)._TAURI_IPC_,
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