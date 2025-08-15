import { invoke } from '@tauri-apps/api/core'

export interface NamedEntry { id: string; name: string }
export interface OrganizationEntry { id: string; name: string; groups: NamedEntry[]; projects: NamedEntry[] }
export interface Metadata { contacts: Array<{id: string; display_name: string; four_word_address: string}>; organizations: OrganizationEntry[] }
export interface ScopePath { scope: string }
export interface MarkdownFileInfo { name: string; path: string; size: number }

export async function initLocalStores(): Promise<string> {
  return invoke<string>('init_local_stores')
}

export async function getMetadata(): Promise<Metadata> {
  return invoke<Metadata>('get_metadata')
}

export async function createOrganization(name: string): Promise<OrganizationEntry> {
  return invoke<OrganizationEntry>('create_organization', { name })
}

export async function createGroup(orgId: string, name: string): Promise<NamedEntry> {
  return invoke<NamedEntry>('create_group', { orgId, name })
}

export async function createProject(orgId: string, name: string): Promise<NamedEntry> {
  return invoke<NamedEntry>('create_project', { orgId, name })
}

export async function addContact(displayName: string, fourWordAddress: string) {
  return invoke('add_contact_local', { displayName, fourWordAddress })
}

export async function listMarkdown(scope: string): Promise<MarkdownFileInfo[]> {
  return invoke<MarkdownFileInfo[]>('list_markdown', { scope: { scope } as ScopePath })
}

export async function createMarkdown(scope: string, name: string, initialContent?: string): Promise<string> {
  return invoke<string>('create_markdown', { scope: { scope } as ScopePath, name, initialContent })
}

export async function readMarkdownFile(path: string): Promise<string> {
  return invoke<string>('read_markdown_file', { path })
}

export async function writeMarkdownFile(path: string, content: string): Promise<void> {
  return invoke('write_markdown_file', { path, content })
}
