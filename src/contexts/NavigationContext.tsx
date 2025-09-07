import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react'

// Navigation context types
export type NavigationContextType = 'personal' | 'organization'
export type EntityType = 'individual' | 'group' | 'project' | 'channel' | 'overview'

export interface Breadcrumb {
  label: string
  path: string
  type: NavigationContextType | EntityType
  id?: string
}

export interface NavigationState {
  context: NavigationContextType
  organizationId?: string
  organizationName?: string
  entityType: EntityType
  entityId?: string
  entityName?: string
  breadcrumbs: Breadcrumb[]
  history: NavigationState[]
}

interface NavigationContextValue {
  state: NavigationState
  switchToPersonal: () => void
  switchToOrganization: (orgId: string, orgName: string) => void
  selectEntity: (type: EntityType, id?: string, name?: string) => void
  navigateBack: () => void
  clearHistory: () => void
  canGoBack: boolean
}

const initialState: NavigationState = {
  context: 'personal',
  entityType: 'overview',
  breadcrumbs: [
    { label: 'Personal', path: '/', type: 'personal' }
  ],
  history: []
}

const NavigationContext = createContext<NavigationContextValue | undefined>(undefined)

export const useNavigation = () => {
  const context = useContext(NavigationContext)
  if (!context) {
    throw new Error('useNavigation must be used within NavigationProvider')
  }
  return context
}

interface NavigationProviderProps {
  children: ReactNode
}

export const NavigationProvider: React.FC<NavigationProviderProps> = ({ children }) => {
  const [state, setState] = useState<NavigationState>(initialState)
  const [history, setHistory] = useState<NavigationState[]>([])

  const updateBreadcrumbs = useCallback((newState: NavigationState): Breadcrumb[] => {
    const breadcrumbs: Breadcrumb[] = []
    
    // Base breadcrumb
    if (newState.context === 'personal') {
      breadcrumbs.push({ label: 'Personal', path: '/', type: 'personal' })
    } else if (newState.context === 'organization' && newState.organizationName) {
      breadcrumbs.push({ label: 'Organizations', path: '/organizations', type: 'organization' })
      breadcrumbs.push({ 
        label: newState.organizationName, 
        path: `/organizations/${newState.organizationId}`, 
        type: 'organization',
        id: newState.organizationId
      })
    }
    
    // Entity breadcrumb
    if (newState.entityType !== 'overview' && newState.entityName) {
      const entityPath = newState.context === 'personal' 
        ? `/${newState.entityType}/${newState.entityId}`
        : `/organizations/${newState.organizationId}/${newState.entityType}/${newState.entityId}`
      
      breadcrumbs.push({
        label: newState.entityName,
        path: entityPath,
        type: newState.entityType,
        id: newState.entityId
      })
    }
    
    return breadcrumbs
  }, [])

  const switchToPersonal = useCallback(() => {
    setHistory(prev => [...prev, state])
    const newState: NavigationState = {
      context: 'personal',
      entityType: 'overview',
      breadcrumbs: [{ label: 'Personal', path: '/', type: 'personal' }],
      history: [...history, state]
    }
    setState(newState)
  }, [state, history])

  const switchToOrganization = useCallback((orgId: string, orgName: string) => {
    setHistory(prev => [...prev, state])
    const newState: NavigationState = {
      context: 'organization',
      organizationId: orgId,
      organizationName: orgName,
      entityType: 'overview',
      breadcrumbs: [],
      history: [...history, state]
    }
    newState.breadcrumbs = updateBreadcrumbs(newState)
    setState(newState)
  }, [state, history, updateBreadcrumbs])

  const selectEntity = useCallback((type: EntityType, id?: string, name?: string) => {
    setHistory(prev => [...prev, state])
    const newState: NavigationState = {
      ...state,
      entityType: type,
      entityId: id,
      entityName: name,
      history: [...history, state]
    }
    newState.breadcrumbs = updateBreadcrumbs(newState)
    setState(newState)
  }, [state, history, updateBreadcrumbs])

  const navigateBack = useCallback(() => {
    if (history.length > 0) {
      const previousState = history[history.length - 1]
      setHistory(prev => prev.slice(0, -1))
      setState(previousState)
    }
  }, [history])

  const clearHistory = useCallback(() => {
    setHistory([])
    setState(initialState)
  }, [])

  const value: NavigationContextValue = {
    state,
    switchToPersonal,
    switchToOrganization,
    selectEntity,
    navigateBack,
    clearHistory,
    canGoBack: history.length > 0
  }

  return (
    <NavigationContext.Provider value={value}>
      {children}
    </NavigationContext.Provider>
  )
}

export default NavigationContext
