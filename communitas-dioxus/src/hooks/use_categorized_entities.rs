//! Entity categorization hook for efficient sidebar rendering.
//!
//! This module provides a hook and data structure for categorizing entities
//! from a directory snapshot in a single O(n) pass, avoiding repeated filtering.

use communitas_ui_api::{OrganizationCategory, UnifiedContact, UnifiedEntity, UnifiedEntityType};
use communitas_ui_service::directory::DirectorySnapshot;

/// Entities categorized by type in a single pass.
///
/// Avoids O(n*k) filtering where k is the number of categories.
/// Instead, categorizes all entities in O(n) time.
///
/// # Example
///
/// ```rust,ignore
/// use communitas_dioxus::hooks::CategorizedEntities;
///
/// let snapshot = services.directory().current_snapshot();
/// let categorized = CategorizedEntities::from_snapshot(&snapshot);
///
/// // Now access each category directly
/// for org in &categorized.organizations {
///     println!("Organization: {}", org.name);
/// }
/// ```
#[derive(Default, Clone, PartialEq)]
pub struct CategorizedEntities {
    /// Organizations (non-community type)
    pub organizations: Vec<UnifiedEntity>,
    /// Organizations marked as Community category
    pub communities: Vec<UnifiedEntity>,
    /// Projects
    pub projects: Vec<UnifiedEntity>,
    /// All groups (both personal and organization groups)
    pub groups: Vec<UnifiedEntity>,
    /// Channels
    pub channels: Vec<UnifiedEntity>,
    /// Groups with no parent (personal groups, not attached to an organization)
    pub personal_groups: Vec<UnifiedEntity>,
    /// Contacts from the directory
    pub contacts: Vec<UnifiedContact>,
}

impl CategorizedEntities {
    /// Categorize all entities from a directory snapshot in a single pass.
    ///
    /// This is more efficient than filtering multiple times:
    /// ```rust,ignore
    /// // Inefficient: O(n) * k where k = number of categories
    /// let orgs = entities.iter().filter(|e| e.entity_type == Organization);
    /// let projects = entities.iter().filter(|e| e.entity_type == Project);
    /// // etc.
    ///
    /// // Efficient: O(n) single pass
    /// let categorized = CategorizedEntities::from_snapshot(&snapshot);
    /// ```
    pub fn from_snapshot(snapshot: &DirectorySnapshot) -> Self {
        let mut result = Self {
            contacts: snapshot.contacts.clone(),
            ..Default::default()
        };

        for entity in &snapshot.entities {
            match entity.entity_type {
                UnifiedEntityType::Organization => {
                    if entity.category == Some(OrganizationCategory::Community) {
                        result.communities.push(entity.clone());
                    } else {
                        result.organizations.push(entity.clone());
                    }
                }
                UnifiedEntityType::Project => {
                    result.projects.push(entity.clone());
                }
                UnifiedEntityType::Group => {
                    result.groups.push(entity.clone());
                    if entity.parent_id.is_none() {
                        result.personal_groups.push(entity.clone());
                    }
                }
                UnifiedEntityType::Channel => {
                    result.channels.push(entity.clone());
                }
                UnifiedEntityType::Person => {
                    // Persons are handled separately via contacts
                }
            }
        }

        result
    }

    /// Categorize from a slice of entities (legacy method).
    ///
    /// Prefer `from_snapshot` when you have access to the full snapshot
    /// as it also extracts contacts.
    pub fn from_entities(entities: &[UnifiedEntity]) -> Self {
        let mut result = Self::default();

        for entity in entities {
            match entity.entity_type {
                UnifiedEntityType::Organization => {
                    if entity.category == Some(OrganizationCategory::Community) {
                        result.communities.push(entity.clone());
                    } else {
                        result.organizations.push(entity.clone());
                    }
                }
                UnifiedEntityType::Project => {
                    result.projects.push(entity.clone());
                }
                UnifiedEntityType::Group => {
                    result.groups.push(entity.clone());
                    if entity.parent_id.is_none() {
                        result.personal_groups.push(entity.clone());
                    }
                }
                UnifiedEntityType::Channel => {
                    result.channels.push(entity.clone());
                }
                UnifiedEntityType::Person => {
                    // Persons are handled separately via contacts
                }
            }
        }

        result
    }

    /// Filter all categories by a search query.
    ///
    /// Returns a new `CategorizedEntities` with only matching items.
    pub fn filter_by_search(&self, query: &str) -> Self {
        if query.is_empty() {
            return self.clone();
        }

        let query_lower = query.to_lowercase();
        let filter_entities = |entities: &[UnifiedEntity]| -> Vec<UnifiedEntity> {
            entities
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&query_lower))
                .cloned()
                .collect()
        };

        Self {
            organizations: filter_entities(&self.organizations),
            communities: filter_entities(&self.communities),
            projects: filter_entities(&self.projects),
            groups: filter_entities(&self.groups),
            channels: filter_entities(&self.channels),
            personal_groups: filter_entities(&self.personal_groups),
            contacts: self
                .contacts
                .iter()
                .filter(|c| c.display_name.to_lowercase().contains(&query_lower))
                .cloned()
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(id: &str, name: &str, entity_type: UnifiedEntityType) -> UnifiedEntity {
        UnifiedEntity {
            id: id.to_string(),
            name: name.to_string(),
            entity_type,
            parent_id: None,
            category: None,
            description: String::new(),
            member_count: 0,
        }
    }

    #[test]
    fn test_from_entities_categorizes_correctly() {
        let entities = vec![
            make_entity("org1", "My Org", UnifiedEntityType::Organization),
            make_entity("proj1", "My Project", UnifiedEntityType::Project),
            make_entity("grp1", "My Group", UnifiedEntityType::Group),
        ];

        let categorized = CategorizedEntities::from_entities(&entities);

        assert_eq!(categorized.organizations.len(), 1);
        assert_eq!(categorized.projects.len(), 1);
        assert_eq!(categorized.groups.len(), 1);
        assert_eq!(categorized.personal_groups.len(), 1);
        assert_eq!(categorized.communities.len(), 0);
    }

    #[test]
    fn test_filter_by_search() {
        let entities = vec![
            make_entity("org1", "Alpha Org", UnifiedEntityType::Organization),
            make_entity("org2", "Beta Org", UnifiedEntityType::Organization),
            make_entity("proj1", "Alpha Project", UnifiedEntityType::Project),
        ];

        let categorized = CategorizedEntities::from_entities(&entities);
        let filtered = categorized.filter_by_search("alpha");

        assert_eq!(filtered.organizations.len(), 1);
        assert_eq!(filtered.organizations[0].name, "Alpha Org");
        assert_eq!(filtered.projects.len(), 1);
    }

    #[test]
    fn test_empty_search_returns_all() {
        let entities = vec![
            make_entity("org1", "Alpha Org", UnifiedEntityType::Organization),
            make_entity("org2", "Beta Org", UnifiedEntityType::Organization),
        ];

        let categorized = CategorizedEntities::from_entities(&entities);
        let filtered = categorized.filter_by_search("");

        assert_eq!(filtered.organizations.len(), 2);
    }

    #[test]
    fn test_community_vs_organization() {
        let org = make_entity("org1", "Regular Org", UnifiedEntityType::Organization);
        let mut community = make_entity("org2", "My Community", UnifiedEntityType::Organization);
        community.category = Some(OrganizationCategory::Community);

        let categorized = CategorizedEntities::from_entities(&[org, community]);

        assert_eq!(categorized.organizations.len(), 1);
        assert_eq!(categorized.organizations[0].name, "Regular Org");
        assert_eq!(categorized.communities.len(), 1);
        assert_eq!(categorized.communities[0].name, "My Community");
    }

    #[test]
    fn test_personal_vs_org_groups() {
        let personal_group = make_entity("grp1", "Personal Group", UnifiedEntityType::Group);
        let mut org_group = make_entity("grp2", "Org Group", UnifiedEntityType::Group);
        org_group.parent_id = Some("org1".to_string());

        let categorized = CategorizedEntities::from_entities(&[personal_group, org_group]);

        assert_eq!(categorized.groups.len(), 2);
        assert_eq!(categorized.personal_groups.len(), 1);
        assert_eq!(categorized.personal_groups[0].name, "Personal Group");
    }
}
