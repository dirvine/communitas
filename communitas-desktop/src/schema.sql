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

-- Channels (Slack-style)
CREATE TABLE IF NOT EXISTS channels (
    id TEXT PRIMARY KEY,
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    crdt_doc_id TEXT REFERENCES crdt_documents(id),
    created_at INTEGER NOT NULL,
    created_by TEXT NOT NULL
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
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    color TEXT,
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

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_channels_org ON channels(org_id);
CREATE INDEX IF NOT EXISTS idx_channel_members_user ON channel_members(user_id);
CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel_id);
CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id);
CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_projects_org ON projects(org_id);
CREATE INDEX IF NOT EXISTS idx_issues_project ON issues(project_id);
CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status);
CREATE INDEX IF NOT EXISTS idx_issues_assignee ON issues(assignee_id);
CREATE INDEX IF NOT EXISTS idx_issue_comments_issue ON issue_comments(issue_id);
CREATE INDEX IF NOT EXISTS idx_threads_parent ON threads(parent_message_id);
CREATE INDEX IF NOT EXISTS idx_crdt_entity ON crdt_documents(entity_type, entity_id);
