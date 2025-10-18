-- CRDT Documents
-- Stores the serialized Yrs state for each collaborative entity
CREATE TABLE IF NOT EXISTS crdt_documents (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,  -- 'channel', 'issue', 'thread', 'message'
    entity_id TEXT NOT NULL,
    yrs_state BLOB NOT NULL,    -- Serialized Yrs document state
    version INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(entity_type, entity_id)
);

-- Organizations (top-level entities with four-word identities)
CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY,
    four_word_identity TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    private_disk_id TEXT NOT NULL,  -- Encrypted, member-only disk
    public_disk_id TEXT NOT NULL,   -- Public web disk
    website_root TEXT,               -- Optional website root hash
    crdt_doc_id TEXT REFERENCES crdt_documents(id)
);

-- Organization Members
CREATE TABLE IF NOT EXISTS organization_members (
    org_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',  -- 'owner', 'admin', 'member'
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (org_id, user_id),
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE
);

-- Members (users with four-word identities and personal disks)
CREATE TABLE IF NOT EXISTS members (
    id TEXT PRIMARY KEY,
    four_word_identity TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    email TEXT,
    avatar_url TEXT,
    bio TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    personal_disk_id TEXT NOT NULL,  -- Private personal storage
    website_root TEXT,               -- Optional personal website root
    crdt_doc_id TEXT REFERENCES crdt_documents(id)
);

-- Groups (organizational and personal)
CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY,
    four_word_identity TEXT NOT NULL UNIQUE,
    org_id TEXT,  -- NULL for personal groups
    name TEXT NOT NULL,
    description TEXT,
    private_disk_id TEXT NOT NULL,  -- Encrypted, member-only disk
    public_disk_id TEXT NOT NULL,   -- Public web disk
    website_root TEXT,               -- Optional website root hash
    crdt_doc_id TEXT REFERENCES crdt_documents(id),
    created_at INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    group_type TEXT NOT NULL  -- 'organization' or 'personal'
);

-- Group Members
CREATE TABLE IF NOT EXISTS group_members (
    group_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',  -- 'owner', 'admin', 'member'
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (group_id, user_id),
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

-- Channels (Slack-style)
CREATE TABLE IF NOT EXISTS channels (
    id TEXT PRIMARY KEY,
    four_word_identity TEXT NOT NULL UNIQUE,
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    private_disk_id TEXT NOT NULL,  -- Encrypted, member-only disk
    public_disk_id TEXT NOT NULL,   -- Public web disk
    website_root TEXT,               -- Optional website root hash
    crdt_doc_id TEXT REFERENCES crdt_documents(id),
    created_at INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE
);

-- Channel Members
CREATE TABLE IF NOT EXISTS channel_members (
    channel_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',  -- 'owner', 'admin', 'member'
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (channel_id, user_id),
    FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
);

-- Messages
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL,
    thread_id TEXT,              -- NULL for top-level messages
    author_id TEXT NOT NULL,
    content TEXT NOT NULL,
    crdt_doc_id TEXT REFERENCES crdt_documents(id),
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    deleted_at INTEGER,          -- Tombstone for soft delete
    FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE,
    FOREIGN KEY (thread_id) REFERENCES messages(id) ON DELETE CASCADE
);

-- Projects (Linear-style)
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    four_word_identity TEXT NOT NULL UNIQUE,
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    color TEXT,
    private_disk_id TEXT NOT NULL,  -- Encrypted, member-only disk
    public_disk_id TEXT NOT NULL,   -- Public web disk
    website_root TEXT,               -- Optional website root hash
    created_at INTEGER NOT NULL,
    created_by TEXT NOT NULL
);

-- Issues (Linear-style)
CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'backlog',  -- 'backlog', 'todo', 'in-progress', 'done', 'canceled'
    priority TEXT DEFAULT 'medium',           -- 'urgent', 'high', 'medium', 'low'
    assignee_id TEXT,
    reporter_id TEXT NOT NULL,
    crdt_doc_id TEXT REFERENCES crdt_documents(id),
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    status_updated_at INTEGER,                -- LWW timestamp for status changes
    priority_updated_at INTEGER,              -- LWW timestamp for priority changes
    assignee_updated_at INTEGER,              -- LWW timestamp for assignee changes
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Issue Comments
CREATE TABLE IF NOT EXISTS issue_comments (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    content TEXT NOT NULL,
    crdt_doc_id TEXT REFERENCES crdt_documents(id),
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    deleted_at INTEGER,          -- Tombstone for soft delete
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- Threads (for message replies)
CREATE TABLE IF NOT EXISTS threads (
    id TEXT PRIMARY KEY,
    parent_message_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    reply_count INTEGER DEFAULT 0,
    last_reply_at INTEGER,
    crdt_doc_id TEXT REFERENCES crdt_documents(id),
    FOREIGN KEY (parent_message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
);

-- Virtual Disks (private shared + public web per entity)
CREATE TABLE IF NOT EXISTS virtual_disks (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,  -- 'organization', 'channel', 'group', 'project', 'member'
    disk_type TEXT NOT NULL,    -- 'private_shared' or 'public_web'
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    crdt_doc_id TEXT REFERENCES crdt_documents(id)
);

-- Files stored in virtual disks
CREATE TABLE IF NOT EXISTS disk_files (
    id TEXT PRIMARY KEY,
    disk_id TEXT NOT NULL,
    path TEXT NOT NULL,
    content BLOB NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    crdt_doc_id TEXT,           -- For collaborative editing
    is_encrypted BOOLEAN NOT NULL DEFAULT 0,
    UNIQUE(disk_id, path),
    FOREIGN KEY (disk_id) REFERENCES virtual_disks(id) ON DELETE CASCADE
);

-- Call Sessions (WebRTC real-time communication)
CREATE TABLE IF NOT EXISTS call_sessions (
    id TEXT PRIMARY KEY,
    call_type TEXT NOT NULL,        -- 'AudioPeer', 'VideoPeer', 'AudioGroup', 'VideoGroup'
    initiator_id TEXT NOT NULL,
    participants TEXT NOT NULL,      -- JSON array of participant IDs
    state TEXT NOT NULL,             -- 'Idle', 'Initiating', 'Ringing', 'Connecting', 'Connected', 'Disconnecting', 'Ended', 'Failed'
    created_at INTEGER NOT NULL,
    connected_at INTEGER,
    ended_at INTEGER,
    group_id TEXT                    -- For group calls, references groups(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_organizations_four_words ON organizations(four_word_identity);
CREATE INDEX IF NOT EXISTS idx_organization_members_user ON organization_members(user_id);
CREATE INDEX IF NOT EXISTS idx_organization_members_org ON organization_members(org_id);
CREATE INDEX IF NOT EXISTS idx_members_four_words ON members(four_word_identity);
CREATE INDEX IF NOT EXISTS idx_members_email ON members(email);
CREATE INDEX IF NOT EXISTS idx_groups_four_words ON groups(four_word_identity);
CREATE INDEX IF NOT EXISTS idx_groups_org ON groups(org_id);
CREATE INDEX IF NOT EXISTS idx_groups_type ON groups(group_type);
CREATE INDEX IF NOT EXISTS idx_groups_created_by ON groups(created_by);
CREATE INDEX IF NOT EXISTS idx_group_members_user ON group_members(user_id);
CREATE INDEX IF NOT EXISTS idx_group_members_group ON group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_channels_four_words ON channels(four_word_identity);
CREATE INDEX IF NOT EXISTS idx_channels_org ON channels(org_id);
CREATE INDEX IF NOT EXISTS idx_channel_members_user ON channel_members(user_id);
CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel_id);
CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id);
CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_projects_four_words ON projects(four_word_identity);
CREATE INDEX IF NOT EXISTS idx_projects_org ON projects(org_id);
CREATE INDEX IF NOT EXISTS idx_issues_project ON issues(project_id);
CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status);
CREATE INDEX IF NOT EXISTS idx_issues_assignee ON issues(assignee_id);
CREATE INDEX IF NOT EXISTS idx_issue_comments_issue ON issue_comments(issue_id);
CREATE INDEX IF NOT EXISTS idx_threads_parent ON threads(parent_message_id);
CREATE INDEX IF NOT EXISTS idx_crdt_entity ON crdt_documents(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_virtual_disks_entity ON virtual_disks(entity_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_virtual_disks_type ON virtual_disks(disk_type);
CREATE INDEX IF NOT EXISTS idx_disk_files_disk ON disk_files(disk_id);
CREATE INDEX IF NOT EXISTS idx_disk_files_path ON disk_files(disk_id, path);
CREATE INDEX IF NOT EXISTS idx_call_sessions_initiator ON call_sessions(initiator_id);
CREATE INDEX IF NOT EXISTS idx_call_sessions_group ON call_sessions(group_id);
CREATE INDEX IF NOT EXISTS idx_call_sessions_state ON call_sessions(state);
CREATE INDEX IF NOT EXISTS idx_call_sessions_created ON call_sessions(created_at);
