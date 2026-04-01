// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contacts service layer for managing contact operations.
//!
//! This module provides a service wrapper around the Communitas core contact APIs,
//! following the same pattern as `DirectoryService`.

use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::auth::AuthController;

/// Errors returned by the contacts service.
#[derive(Debug, Error)]
pub enum ContactsError {
    /// User is not authenticated - must login first.
    #[error("not authenticated")]
    NotAuthenticated,

    /// Core API returned an error.
    #[error("core error: {0}")]
    Core(String),

    /// Invalid four-words format.
    #[error("invalid four-words format: {0}")]
    InvalidFourWords(String),
}

impl From<String> for ContactsError {
    fn from(value: String) -> Self {
        ContactsError::Core(value)
    }
}

/// Service for managing contacts through the UI layer.
///
/// Wraps the core `CommunitasApi` contact operations with authentication
/// checks and proper error handling.
pub struct ContactsService {
    auth: Arc<AuthController>,
}

impl ContactsService {
    /// Create a new contacts service with the given auth controller.
    pub fn new(auth: Arc<AuthController>) -> Self {
        Self { auth }
    }

    /// Validate four-words format.
    ///
    /// Four-words must be exactly 4 alphabetic words separated by hyphens.
    fn validate_four_words(four_words: &str) -> Result<(), ContactsError> {
        let trimmed = four_words.trim();
        let words: Vec<&str> = trimmed.split('-').collect();

        if words.len() != 4 {
            return Err(ContactsError::InvalidFourWords(
                "must contain exactly 4 words separated by hyphens".to_string(),
            ));
        }

        for word in &words {
            if word.is_empty() {
                return Err(ContactsError::InvalidFourWords(
                    "words cannot be empty".to_string(),
                ));
            }
            if !word.chars().all(|c| c.is_alphabetic()) {
                return Err(ContactsError::InvalidFourWords(
                    "words must contain only alphabetic characters".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Add a new contact with the given four-words and display name.
    ///
    /// # Arguments
    ///
    /// * `four_words` - The four-word network address of the contact (e.g., "ocean-forest-moon-star")
    /// * `display_name` - Human-friendly display name for the contact
    ///
    /// # Returns
    ///
    /// The contact ID of the newly created contact on success.
    ///
    /// # Errors
    ///
    /// Returns `ContactsError::NotAuthenticated` if the user is not logged in.
    /// Returns `ContactsError::InvalidFourWords` if the four-words format is invalid.
    /// Returns `ContactsError::Core` if the core API returns an error.
    #[instrument(name = "ui.contacts.add_contact", skip(self))]
    pub async fn add_contact(
        &self,
        four_words: &str,
        display_name: &str,
    ) -> Result<String, ContactsError> {
        // Validate four-words format
        Self::validate_four_words(four_words)?;

        // Get authenticated API
        let api = self
            .auth
            .api_async()
            .await
            .ok_or(ContactsError::NotAuthenticated)?;

        // Normalize inputs
        let four_words_normalized = four_words.trim().to_lowercase();
        let display_name_trimmed = display_name.trim();

        // Use display name if provided, otherwise use four-words
        let effective_display_name = if display_name_trimmed.is_empty() {
            four_words_normalized.clone()
        } else {
            display_name_trimmed.to_string()
        };

        // Create contact via core API
        api.contact_create(
            effective_display_name.clone(),
            Some(four_words_normalized.clone()),
            false, // not favourite by default
        )
        .await?;

        // Resolve contact ID from current contact list
        let contacts = api.contacts_list().await?;
        if let Some(contact) = contacts
            .iter()
            .find(|c| c.four_words.as_deref() == Some(four_words_normalized.as_str()))
        {
            tracing::info!(
                contact_id = %contact.id,
                four_words = %four_words_normalized,
                "Contact created successfully"
            );
            return Ok(contact.id.clone());
        }

        if let Some(contact) = contacts
            .iter()
            .filter(|c| c.display_name == effective_display_name)
            .max_by_key(|c| c.created_at)
        {
            tracing::info!(
                contact_id = %contact.id,
                display_name = %effective_display_name,
                "Contact created successfully (matched by display name)"
            );
            return Ok(contact.id.clone());
        }

        Err(ContactsError::Core(
            "contact created but ID could not be resolved".to_string(),
        ))
    }

    /// Delete a contact by ID.
    ///
    /// # Errors
    ///
    /// Returns `ContactsError::NotAuthenticated` if the user is not logged in.
    /// Returns `ContactsError::Core` if the core API returns an error.
    #[instrument(name = "ui.contacts.delete_contact", skip(self))]
    pub async fn delete_contact(&self, contact_id: &str) -> Result<(), ContactsError> {
        let api = self
            .auth
            .api_async()
            .await
            .ok_or(ContactsError::NotAuthenticated)?;

        api.contact_delete(contact_id.to_string()).await?;

        tracing::info!(contact_id = %contact_id, "Contact deleted successfully");
        Ok(())
    }

    /// Update a contact's display name or favourite status.
    ///
    /// # Errors
    ///
    /// Returns `ContactsError::NotAuthenticated` if the user is not logged in.
    /// Returns `ContactsError::Core` if the core API returns an error.
    #[instrument(name = "ui.contacts.update_contact", skip(self))]
    pub async fn update_contact(
        &self,
        contact_id: &str,
        display_name: Option<&str>,
        is_favourite: Option<bool>,
    ) -> Result<(), ContactsError> {
        let api = self
            .auth
            .api_async()
            .await
            .ok_or(ContactsError::NotAuthenticated)?;

        api.contact_update(
            contact_id.to_string(),
            display_name.map(|s| s.to_string()),
            is_favourite,
        )
        .await?;

        tracing::info!(contact_id = %contact_id, "Contact updated successfully");
        Ok(())
    }

    /// Toggle the favourite status of a contact.
    ///
    /// # Errors
    ///
    /// Returns `ContactsError::NotAuthenticated` if the user is not logged in.
    /// Returns `ContactsError::Core` if the core API returns an error.
    #[instrument(name = "ui.contacts.toggle_favourite", skip(self))]
    pub async fn toggle_favourite(
        &self,
        contact_id: &str,
        is_favourite: bool,
    ) -> Result<(), ContactsError> {
        self.update_contact(contact_id, None, Some(is_favourite))
            .await
    }

    /// Link a local-only contact to a network identity.
    ///
    /// # Errors
    ///
    /// Returns `ContactsError::NotAuthenticated` if the user is not logged in.
    /// Returns `ContactsError::InvalidFourWords` if the four-words format is invalid.
    /// Returns `ContactsError::Core` if the core API returns an error.
    #[instrument(name = "ui.contacts.link_contact", skip(self))]
    pub async fn link_contact(
        &self,
        contact_id: &str,
        four_words: &str,
    ) -> Result<(), ContactsError> {
        Self::validate_four_words(four_words)?;

        let api = self
            .auth
            .api_async()
            .await
            .ok_or(ContactsError::NotAuthenticated)?;

        let four_words_normalized = four_words.trim().to_lowercase();

        api.contact_link(contact_id.to_string(), four_words_normalized)
            .await?;

        tracing::info!(contact_id = %contact_id, "Contact linked to network identity");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_four_words_valid() {
        assert!(ContactsService::validate_four_words("ocean-forest-moon-star").is_ok());
        assert!(ContactsService::validate_four_words("alpha-beta-gamma-delta").is_ok());
        assert!(ContactsService::validate_four_words("  ocean-forest-moon-star  ").is_ok());
    }

    #[test]
    fn validate_four_words_wrong_count() {
        let result = ContactsService::validate_four_words("ocean-forest-moon");
        assert!(matches!(result, Err(ContactsError::InvalidFourWords(_))));

        let result = ContactsService::validate_four_words("ocean-forest-moon-star-extra");
        assert!(matches!(result, Err(ContactsError::InvalidFourWords(_))));

        let result = ContactsService::validate_four_words("single");
        assert!(matches!(result, Err(ContactsError::InvalidFourWords(_))));
    }

    #[test]
    fn validate_four_words_empty_words() {
        let result = ContactsService::validate_four_words("ocean--moon-star");
        assert!(matches!(result, Err(ContactsError::InvalidFourWords(_))));

        let result = ContactsService::validate_four_words("-forest-moon-star");
        assert!(matches!(result, Err(ContactsError::InvalidFourWords(_))));
    }

    #[test]
    fn validate_four_words_non_alphabetic() {
        let result = ContactsService::validate_four_words("ocean-forest123-moon-star");
        assert!(matches!(result, Err(ContactsError::InvalidFourWords(_))));

        let result = ContactsService::validate_four_words("ocean-forest-moon-star1");
        assert!(matches!(result, Err(ContactsError::InvalidFourWords(_))));

        let result = ContactsService::validate_four_words("123-456-789-000");
        assert!(matches!(result, Err(ContactsError::InvalidFourWords(_))));
    }
}
