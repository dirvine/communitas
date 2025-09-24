import { IdentityInfo } from '../types'

const IDENTITY_STORAGE_KEY = 'communitas.identities'

export const loadStoredIdentities = (): IdentityInfo[] => {
  if (typeof window === 'undefined') return []
  try {
    const raw = window.localStorage.getItem(IDENTITY_STORAGE_KEY)
    if (!raw) return []
    return JSON.parse(raw) as IdentityInfo[]
  } catch (error) {
    console.warn('[Communitas] Failed to parse stored identities', error)
    return []
  }
}

export const persistIdentity = (identity: IdentityInfo) => {
  const identities = upsertIdentity(identity)
  if (typeof window === 'undefined') return identities
  try {
    window.localStorage.setItem(IDENTITY_STORAGE_KEY, JSON.stringify(identities))
  } catch (error) {
    console.warn('[Communitas] Failed to persist identity', error)
  }
  return identities
}

export const storeIdentities = (identities: IdentityInfo[]) => {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(IDENTITY_STORAGE_KEY, JSON.stringify(identities))
  } catch (error) {
    console.warn('[Communitas] Failed to persist identities', error)
  }
}

const upsertIdentity = (identity: IdentityInfo): IdentityInfo[] => {
  const existing = loadStoredIdentities().filter((item) => item.four_word_address !== identity.four_word_address)
  existing.unshift(identity)
  return existing.slice(0, 10)
}
