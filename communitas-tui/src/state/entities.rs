use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of entities in Communitas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Organization,
    Project,
    Group,
    Contact,
}

impl EntityType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Organization => "Organizations",
            Self::Project => "Projects",
            Self::Group => "Groups",
            Self::Contact => "Contacts",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Organization => "🏢",
            Self::Project => "📁",
            Self::Group => "👥",
            Self::Contact => "👤",
        }
    }

    pub fn key(&self) -> char {
        match self {
            Self::Organization => 'o',
            Self::Project => 'p',
            Self::Group => 'g',
            Self::Contact => 'c',
        }
    }
}

/// Entity data storage
#[derive(Debug)]
pub struct EntityData {
    /// Cached channels by organization
    pub channels: HashMap<String, Vec<ChannelData>>,
    /// Cached messages by channel
    pub messages: HashMap<String, Vec<MessageData>>,
    /// Cached projects
    pub projects: Vec<ProjectData>,
    /// Cached issues by project
    pub issues: HashMap<String, Vec<IssueData>>,
    /// Cached groups
    pub groups: Vec<GroupData>,
    /// Cached contacts
    pub contacts: Vec<ContactData>,
}

impl EntityData {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            messages: HashMap::new(),
            projects: Vec::new(),
            issues: HashMap::new(),
            groups: Vec::new(),
            contacts: Vec::new(),
        }
    }
}

impl Default for EntityData {
    fn default() -> Self {
        Self::new()
    }
}

// Data structures matching backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub unread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub timestamp: i64,
    pub thread_id: Option<String>,
    pub thread_count: usize,
    pub reactions: Vec<ReactionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionData {
    pub emoji: String,
    pub count: usize,
    pub users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub issue_counts: IssueStatusCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatusCounts {
    pub backlog: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub done: usize,
    pub canceled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueData {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<String>,
    pub reporter_id: String,
    pub comment_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupData {
    pub id: String,
    pub name: String,
    pub member_count: usize,
    pub last_message: Option<String>,
    pub unread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactData {
    pub id: String,
    pub four_words: String,
    pub display_name: String,
    pub last_seen: Option<i64>,
    pub unread_count: usize,
}
