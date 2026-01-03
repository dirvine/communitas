//! Resource type definitions for granular permissions.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Types of resources that can have granular permissions.
///
/// Each resource type represents a category of data or functionality
/// within an entity that can be independently controlled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// Chat messages within the entity.
    ///
    /// Controls:
    /// - Viewing message history
    /// - Sending new messages
    /// - Reacting to messages
    /// - Creating/viewing threads
    Messages,

    /// Collaborative documents (CRDT-synchronized).
    ///
    /// Controls:
    /// - Viewing document list
    /// - Reading document content
    /// - Creating new documents
    /// - Editing document content
    /// - Deleting documents
    Documents,

    /// Kanban boards (only applicable to Project entities).
    ///
    /// Controls:
    /// - Viewing boards and cards
    /// - Creating/modifying boards
    /// - Creating/moving/editing cards
    /// - Adding comments to cards
    KanbanBoards,

    /// Files in entity storage (virtual disk).
    ///
    /// Controls:
    /// - Viewing file list
    /// - Downloading files
    /// - Uploading new files
    /// - Deleting files
    Files,

    /// Member list and information.
    ///
    /// Controls:
    /// - Viewing member list
    /// - Viewing member profiles
    /// - Adding/removing members (Edit level)
    /// - Modifying member roles (Edit level)
    Members,

    /// Entity settings and configuration.
    ///
    /// Controls:
    /// - Viewing entity settings
    /// - Modifying entity name/description
    /// - Changing entity configuration
    /// - Managing entity metadata
    Settings,
}

impl ResourceType {
    /// Get all resource types as a slice.
    ///
    /// Useful for iterating over all types when checking or displaying permissions.
    pub fn all() -> &'static [ResourceType] {
        &[
            ResourceType::Messages,
            ResourceType::Documents,
            ResourceType::KanbanBoards,
            ResourceType::Files,
            ResourceType::Members,
            ResourceType::Settings,
        ]
    }

    /// Convert to string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            ResourceType::Messages => "messages",
            ResourceType::Documents => "documents",
            ResourceType::KanbanBoards => "kanban_boards",
            ResourceType::Files => "files",
            ResourceType::Members => "members",
            ResourceType::Settings => "settings",
        }
    }
}

/// Error when parsing an invalid resource type string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResourceTypeError {
    /// The invalid input string.
    pub invalid_value: String,
}

impl std::fmt::Display for ParseResourceTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid resource type '{}': expected 'messages', 'documents', 'kanban_boards', 'files', 'members', or 'settings'",
            self.invalid_value
        )
    }
}

impl std::error::Error for ParseResourceTypeError {}

impl FromStr for ResourceType {
    type Err = ParseResourceTypeError;

    /// Parse from string representation.
    ///
    /// Accepts the snake_case name (case-insensitive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "messages" | "message" | "chat" => Ok(ResourceType::Messages),
            "documents" | "document" | "docs" | "doc" => Ok(ResourceType::Documents),
            "kanban_boards" | "kanbanboards" | "kanban" | "boards" => {
                Ok(ResourceType::KanbanBoards)
            }
            "files" | "file" | "storage" => Ok(ResourceType::Files),
            "members" | "member" | "users" => Ok(ResourceType::Members),
            "settings" | "setting" | "config" => Ok(ResourceType::Settings),
            _ => Err(ParseResourceTypeError {
                invalid_value: s.to_string(),
            }),
        }
    }
}

impl ResourceType {
    /// Get a human-readable display name.
    pub fn display_name(self) -> &'static str {
        match self {
            ResourceType::Messages => "Messages",
            ResourceType::Documents => "Documents",
            ResourceType::KanbanBoards => "Kanban Boards",
            ResourceType::Files => "Files",
            ResourceType::Members => "Members",
            ResourceType::Settings => "Settings",
        }
    }

    /// Get a description of what this resource type controls.
    pub fn description(self) -> &'static str {
        match self {
            ResourceType::Messages => "Chat messages, threads, and reactions",
            ResourceType::Documents => "Collaborative documents with real-time editing",
            ResourceType::KanbanBoards => "Project boards, columns, cards, and comments",
            ResourceType::Files => "File storage and downloads",
            ResourceType::Members => "Member list and role management",
            ResourceType::Settings => "Entity configuration and metadata",
        }
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_types() {
        let all = ResourceType::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&ResourceType::Messages));
        assert!(all.contains(&ResourceType::Documents));
        assert!(all.contains(&ResourceType::KanbanBoards));
        assert!(all.contains(&ResourceType::Files));
        assert!(all.contains(&ResourceType::Members));
        assert!(all.contains(&ResourceType::Settings));
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            "messages".parse::<ResourceType>().unwrap(),
            ResourceType::Messages
        );
        assert_eq!(
            "KANBAN_BOARDS".parse::<ResourceType>().unwrap(),
            ResourceType::KanbanBoards
        );
        assert_eq!(
            "docs".parse::<ResourceType>().unwrap(),
            ResourceType::Documents
        );
        assert!("invalid".parse::<ResourceType>().is_err());
    }

    #[test]
    fn test_as_str() {
        assert_eq!(ResourceType::Messages.as_str(), "messages");
        assert_eq!(ResourceType::KanbanBoards.as_str(), "kanban_boards");
    }

    #[test]
    fn test_serialization() {
        let resource = ResourceType::KanbanBoards;
        let json = serde_json::to_string(&resource).unwrap();
        assert_eq!(json, "\"kanban_boards\"");

        let parsed: ResourceType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, resource);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ResourceType::Messages), "Messages");
        assert_eq!(format!("{}", ResourceType::KanbanBoards), "Kanban Boards");
    }

    #[test]
    fn test_display_all() {
        // Test Display for all resource types
        assert_eq!(format!("{}", ResourceType::Messages), "Messages");
        assert_eq!(format!("{}", ResourceType::Documents), "Documents");
        assert_eq!(format!("{}", ResourceType::KanbanBoards), "Kanban Boards");
        assert_eq!(format!("{}", ResourceType::Files), "Files");
        assert_eq!(format!("{}", ResourceType::Members), "Members");
        assert_eq!(format!("{}", ResourceType::Settings), "Settings");
    }

    #[test]
    fn test_display_name() {
        assert_eq!(ResourceType::Messages.display_name(), "Messages");
        assert_eq!(ResourceType::Documents.display_name(), "Documents");
        assert_eq!(ResourceType::KanbanBoards.display_name(), "Kanban Boards");
        assert_eq!(ResourceType::Files.display_name(), "Files");
        assert_eq!(ResourceType::Members.display_name(), "Members");
        assert_eq!(ResourceType::Settings.display_name(), "Settings");
    }

    #[test]
    fn test_description() {
        // Verify descriptions are non-empty and meaningful
        assert!(ResourceType::Messages.description().contains("message"));
        assert!(ResourceType::Documents.description().contains("document"));
        assert!(ResourceType::KanbanBoards.description().contains("board"));
        assert!(
            ResourceType::Files.description().contains("file")
                || ResourceType::Files.description().contains("storage")
        );
        assert!(
            ResourceType::Members.description().contains("member")
                || ResourceType::Members.description().contains("role")
        );
        assert!(
            ResourceType::Settings.description().contains("config")
                || ResourceType::Settings.description().contains("metadata")
        );
    }

    #[test]
    fn test_parse_error_display() {
        let err: Result<ResourceType, _> = "invalid_resource".parse();
        assert!(err.is_err());
        let err = err.unwrap_err();
        assert!(err.to_string().contains("invalid_resource"));
        assert!(err.to_string().contains("messages"));
        assert!(err.to_string().contains("documents"));
    }

    #[test]
    fn test_parse_error_invalid_value() {
        let err: Result<ResourceType, _> = "bad".parse();
        let err = err.unwrap_err();
        assert_eq!(err.invalid_value, "bad");
    }

    #[test]
    fn test_from_str_all_aliases() {
        // Messages aliases
        assert_eq!(
            "message".parse::<ResourceType>().unwrap(),
            ResourceType::Messages
        );
        assert_eq!(
            "chat".parse::<ResourceType>().unwrap(),
            ResourceType::Messages
        );

        // Documents aliases
        assert_eq!(
            "document".parse::<ResourceType>().unwrap(),
            ResourceType::Documents
        );
        assert_eq!(
            "doc".parse::<ResourceType>().unwrap(),
            ResourceType::Documents
        );

        // KanbanBoards aliases
        assert_eq!(
            "kanbanboards".parse::<ResourceType>().unwrap(),
            ResourceType::KanbanBoards
        );
        assert_eq!(
            "kanban".parse::<ResourceType>().unwrap(),
            ResourceType::KanbanBoards
        );
        assert_eq!(
            "boards".parse::<ResourceType>().unwrap(),
            ResourceType::KanbanBoards
        );

        // Files aliases
        assert_eq!("file".parse::<ResourceType>().unwrap(), ResourceType::Files);
        assert_eq!(
            "storage".parse::<ResourceType>().unwrap(),
            ResourceType::Files
        );

        // Members aliases
        assert_eq!(
            "member".parse::<ResourceType>().unwrap(),
            ResourceType::Members
        );
        assert_eq!(
            "users".parse::<ResourceType>().unwrap(),
            ResourceType::Members
        );

        // Settings aliases
        assert_eq!(
            "setting".parse::<ResourceType>().unwrap(),
            ResourceType::Settings
        );
        assert_eq!(
            "config".parse::<ResourceType>().unwrap(),
            ResourceType::Settings
        );
    }

    #[test]
    fn test_as_str_all() {
        assert_eq!(ResourceType::Messages.as_str(), "messages");
        assert_eq!(ResourceType::Documents.as_str(), "documents");
        assert_eq!(ResourceType::KanbanBoards.as_str(), "kanban_boards");
        assert_eq!(ResourceType::Files.as_str(), "files");
        assert_eq!(ResourceType::Members.as_str(), "members");
        assert_eq!(ResourceType::Settings.as_str(), "settings");
    }

    #[test]
    fn test_serialization_all_types() {
        for resource in ResourceType::all() {
            let json = serde_json::to_string(resource).unwrap();
            let parsed: ResourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, resource);
        }
    }

    #[test]
    fn test_hash_and_eq() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ResourceType::Messages);
        set.insert(ResourceType::Documents);
        set.insert(ResourceType::Messages); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&ResourceType::Messages));
        assert!(set.contains(&ResourceType::Documents));
        assert!(!set.contains(&ResourceType::Files));
    }

    #[test]
    fn test_copy_semantics() {
        let original = ResourceType::KanbanBoards;
        let copied = original; // Copy happens automatically for Copy types

        assert_eq!(original, copied);
    }

    #[test]
    fn test_debug_format() {
        let debug_str = format!("{:?}", ResourceType::Messages);
        assert!(debug_str.contains("Messages"));
    }
}
