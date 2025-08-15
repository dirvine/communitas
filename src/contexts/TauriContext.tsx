import React, { createContext, useContext, ReactNode, useMemo } from 'react';
import { isTauriApp, getTauriApi, safeInvoke } from '../utils/tauri';

interface TauriContextType {
  isAvailable: boolean;
  invoke: typeof safeInvoke;
  api: any;
}

const TauriContext = createContext<TauriContextType>({
  isAvailable: false,
  invoke: async () => null,
  api: null
});

interface TauriProviderProps {
  children: ReactNode;
}

export const TauriProvider: React.FC<TauriProviderProps> = ({ children }) => {
  const value = useMemo(() => {
    const isAvailable = isTauriApp();
    const api = getTauriApi();
    
    return {
      isAvailable,
      invoke: safeInvoke,
      api
    };
  }, []);

  return (
    <TauriContext.Provider value={value}>
      {children}
    </TauriContext.Provider>
  );
};

export const useTauri = () => {
  const context = useContext(TauriContext);
  if (!context) {
    throw new Error('useTauri must be used within TauriProvider');
  }
  return context;
};