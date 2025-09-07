// @ts-ignore - Tauri API import for desktop builds
import { invoke } from '@tauri-apps/api/core'

// Build canonical message bytes per saorsa-core for identity.website_root update
// Message: b"saorsa-identity:website_root:v1" || id || pk || CBOR(website_root)
// For convenience we accept hex inputs and return canonical bytes as hex.
export function buildWebsiteRootCanonicalHex(
  idHex: string,
  pkHex: string,
  websiteRootHex: string,
): string {
  const encoder = new TextEncoder()
  const DST = encoder.encode('saorsa-identity:website_root:v1')
  const id = hexToBytes(idHex)
  const pk = hexToBytes(pkHex)
  const website = hexToBytes(websiteRootHex)
  // Minimal CBOR for 32-byte bstr: 0x58 0x20 <32 bytes>
  const cbor = new Uint8Array(2 + website.length)
  cbor[0] = 0x58
  cbor[1] = 0x20
  cbor.set(website, 2)
  // Concat
  const total = new Uint8Array(DST.length + id.length + pk.length + cbor.length)
  total.set(DST, 0)
  total.set(id, DST.length)
  total.set(pk, DST.length + id.length)
  total.set(cbor, DST.length + id.length + pk.length)
  return bytesToHex(total)
}

export async function applyWebsiteRootWithSignature(
  idHex: string,
  websiteRootHex: string,
  sigHex: string,
): Promise<boolean> {
  await invoke('core_identity_set_website_root', { idHex, websiteRootHex, sigHex })
  return true
}

export async function publishWebsiteAndSetRoot(
  entityHex: string,
  websiteRootHex: string,
  signer?: (canonicalHex: string) => Promise<string>,
  pkHex?: string,
): Promise<{ published: boolean; updatedIdentity: boolean }> {
  const receipt = await invoke('core_website_publish_receipt', { entityHex, websiteRootHex })
  let updated = false
  if (signer && pkHex) {
    const canonical = buildWebsiteRootCanonicalHex(entityHex, pkHex, websiteRootHex)
    const sigHex = await signer(canonical)
    await invoke('core_identity_set_website_root', { idHex: entityHex, websiteRootHex, sigHex })
    updated = true
  }
  return { published: true, updatedIdentity: updated }
}

function hexToBytes(hex: string): Uint8Array {
  if (hex.startsWith('0x')) hex = hex.slice(2)
  const arr = new Uint8Array(hex.length / 2)
  for (let i = 0; i < hex.length; i += 2) {
    arr[i / 2] = parseInt(hex.slice(i, i + 2), 16)
  }
  return arr
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('')
}

