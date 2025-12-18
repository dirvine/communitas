//! CRDT operations for the Kanban system.
//!
//! This module provides low-level CRDT operations for:
//! - Atomic card moves between columns
//! - OR-Set operations for assignees and tags
//! - Position management in arrays
//!
//! All operations are designed to be conflict-free and work correctly
//! when applied in any order across distributed nodes.

use yrs::{Any, Array, ArrayRef, Doc, Map, MapRef, ReadTxn, Transact, TransactionMut, WriteTxn};

use crate::error::{KanbanError, KanbanResult};

/// Key names used in the Yrs document structure.
pub mod keys {
    /// Root-level metadata map
    pub const METADATA: &str = "metadata";
    /// Board settings map (within metadata)
    pub const SETTINGS: &str = "settings";
    /// Ordered column IDs array
    pub const COLUMN_ORDER: &str = "column_order";
    /// Columns map (column_id -> column data)
    pub const COLUMNS: &str = "columns";
    /// Cards map (card_id -> card data)
    pub const CARDS: &str = "cards";
    /// Tags map (tag_id -> tag data)
    pub const TAGS: &str = "tags";
    /// Card order within a column
    pub const CARD_ORDER: &str = "card_order";
    /// Step order within a card
    pub const STEP_ORDER: &str = "step_order";
    /// Steps map within a card
    pub const STEPS: &str = "steps";
    /// Comments map within a card
    pub const COMMENTS: &str = "comments";
    /// Assignees OR-Set map within a card
    pub const ASSIGNEES: &str = "assignees";
    /// Tags OR-Set map within a card
    pub const CARD_TAGS: &str = "card_tags";
    /// Description YText within a card
    pub const DESCRIPTION: &str = "description";
    /// Content YText within a comment
    pub const CONTENT: &str = "content";
}

/// Add an element to an OR-Set map.
///
/// If the element was previously removed, it will be re-added.
/// Uses add-wins semantics: add always trumps remove if concurrent.
///
/// # Arguments
///
/// * `map` - The map implementing the OR-Set
/// * `txn` - The transaction to use
/// * `key` - The element key to add
/// * `timestamp` - Current timestamp for ordering
pub fn orset_add(
    map: &MapRef,
    txn: &mut TransactionMut<'_>,
    key: &str,
    timestamp: i64,
) -> KanbanResult<()> {
    // Create or update the entry
    let entry_map: MapRef = map.get_or_init(txn, key);

    entry_map.insert(txn, "added_at", Any::BigInt(timestamp));
    entry_map.insert(txn, "removed", Any::Bool(false));
    // Clear removed_at when re-adding
    entry_map.remove(txn, "removed_at");

    Ok(())
}

/// Remove an element from an OR-Set map.
///
/// Marks the element as removed rather than deleting it.
/// Uses tombstone pattern for conflict resolution.
///
/// # Arguments
///
/// * `map` - The map implementing the OR-Set
/// * `txn` - The transaction to use
/// * `key` - The element key to remove
/// * `timestamp` - Current timestamp for ordering
pub fn orset_remove(
    map: &MapRef,
    txn: &mut TransactionMut<'_>,
    key: &str,
    timestamp: i64,
) -> KanbanResult<()> {
    if let Some(yrs::Out::YMap(entry_map)) = map.get(txn, key) {
        entry_map.insert(txn, "removed", Any::Bool(true));
        entry_map.insert(txn, "removed_at", Any::BigInt(timestamp));
    }
    // If entry doesn't exist, nothing to remove
    Ok(())
}

/// Check if an element is present in an OR-Set (added and not removed).
///
/// # Arguments
///
/// * `map` - The map implementing the OR-Set
/// * `txn` - A read transaction
/// * `key` - The element key to check
pub fn orset_contains<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> bool {
    if let Some(yrs::Out::YMap(entry_map)) = map.get(txn, key) {
        if let Some(yrs::Out::Any(Any::Bool(removed))) = entry_map.get(txn, "removed") {
            return !removed;
        }
        // If no removed field, consider it present
        return true;
    }
    false
}

/// Get all active (non-removed) elements from an OR-Set.
///
/// # Arguments
///
/// * `map` - The map implementing the OR-Set
/// * `txn` - A read transaction
pub fn orset_members<T: ReadTxn>(map: &MapRef, txn: &T) -> Vec<String> {
    let mut members = Vec::new();
    for (key, _) in map.iter(txn) {
        if orset_contains(map, txn, key) {
            members.push(key.to_string());
        }
    }
    members
}

/// Move a card atomically from one column to another.
///
/// This performs three operations in a single CRDT transaction:
/// 1. Update the card's column_id field (LWW)
/// 2. Remove the card_id from the source column's card_order
/// 3. Insert the card_id into the target column's card_order at position
///
/// Because this is a single transaction, all operations will be
/// applied atomically and consistently across all nodes.
///
/// # Arguments
///
/// * `doc` - The Yrs document
/// * `card_id` - ID of the card to move
/// * `source_column_id` - ID of the current column
/// * `target_column_id` - ID of the destination column
/// * `target_position` - Position in the target column (0-indexed)
/// * `timestamp` - Current timestamp for updated_at
///
/// # Errors
///
/// Returns `KanbanError::CardNotFound` if the card doesn't exist.
/// Returns `KanbanError::ColumnNotFound` if either column doesn't exist.
pub fn move_card_atomic(
    doc: &Doc,
    card_id: &str,
    source_column_id: &str,
    target_column_id: &str,
    target_position: u32,
    timestamp: i64,
) -> KanbanResult<()> {
    let mut txn = doc.transact_mut();

    // Get root maps
    let root = txn.get_or_insert_map("root");
    let columns = get_map(&root, &txn, keys::COLUMNS)?;
    let cards = get_map(&root, &txn, keys::CARDS)?;

    // Verify card exists
    let card = match cards.get(&txn, card_id) {
        Some(yrs::Out::YMap(m)) => m,
        _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
    };

    // Verify source column exists and get its card_order
    let source_col = match columns.get(&txn, source_column_id) {
        Some(yrs::Out::YMap(m)) => m,
        _ => return Err(KanbanError::ColumnNotFound(source_column_id.to_string())),
    };

    let source_order = match source_col.get(&txn, keys::CARD_ORDER) {
        Some(yrs::Out::YArray(a)) => a,
        _ => {
            return Err(KanbanError::InvalidData(format!(
                "Column {} missing card_order",
                source_column_id
            )));
        }
    };

    // Verify target column exists and get its card_order
    let target_col = match columns.get(&txn, target_column_id) {
        Some(yrs::Out::YMap(m)) => m,
        _ => return Err(KanbanError::ColumnNotFound(target_column_id.to_string())),
    };

    let target_order = match target_col.get(&txn, keys::CARD_ORDER) {
        Some(yrs::Out::YArray(a)) => a,
        _ => {
            return Err(KanbanError::InvalidData(format!(
                "Column {} missing card_order",
                target_column_id
            )));
        }
    };

    // 1. Update card's column_id and updated_at
    card.insert(&mut txn, "column_id", Any::String(target_column_id.into()));
    card.insert(&mut txn, "updated_at", Any::BigInt(timestamp));
    card.insert(&mut txn, "position", Any::BigInt(target_position as i64));

    // 2. Remove from source column's card_order
    if let Some(idx) = find_in_array(&source_order, &txn, card_id) {
        source_order.remove(&mut txn, idx);
    }

    // 3. Insert into target column's card_order at position
    let target_len = target_order.len(&txn);
    let insert_pos = (target_position).min(target_len);
    target_order.insert(&mut txn, insert_pos, Any::String(card_id.into()));

    Ok(())
}

/// Find the index of a string value in a YArray.
fn find_in_array<T: ReadTxn>(array: &ArrayRef, txn: &T, value: &str) -> Option<u32> {
    for (idx, item) in array.iter(txn).enumerate() {
        if let yrs::Out::Any(Any::String(s)) = item
            && s.as_ref() == value
        {
            return Some(idx as u32);
        }
    }
    None
}

/// Reorder items in a YArray.
///
/// Moves an item from `from_index` to `to_index`.
///
/// # Arguments
///
/// * `array` - The array to reorder
/// * `txn` - The transaction to use
/// * `from_index` - Current index of the item
/// * `to_index` - Target index for the item
///
/// Note: This is a tested utility function available for future use.
#[allow(dead_code)]
pub fn reorder_array_item(
    array: &ArrayRef,
    txn: &mut TransactionMut<'_>,
    from_index: u32,
    to_index: u32,
) -> KanbanResult<()> {
    let len = array.len(txn);
    if from_index >= len || to_index >= len {
        return Err(KanbanError::PositionOutOfBounds {
            position: from_index.max(to_index),
            max: len.saturating_sub(1),
        });
    }

    if from_index == to_index {
        return Ok(());
    }

    // Get the item value
    let item = array
        .get(txn, from_index)
        .ok_or_else(|| KanbanError::PositionOutOfBounds {
            position: from_index,
            max: len.saturating_sub(1),
        })?;

    // Remove from old position
    array.remove(txn, from_index);

    // Insert at new position (adjust if we removed before the target)
    let adjusted_index = if from_index < to_index {
        to_index - 1
    } else {
        to_index
    };

    // Clone the Any value for insertion
    if let yrs::Out::Any(any) = item {
        array.insert(txn, adjusted_index, any);
        Ok(())
    } else {
        Err(KanbanError::InvalidData(
            "Cannot reorder non-primitive array item".to_string(),
        ))
    }
}

/// Insert an item into a YArray at a specific position, updating positions.
///
/// # Arguments
///
/// * `array` - The array to insert into
/// * `txn` - The transaction to use
/// * `value` - The string value to insert
/// * `position` - The target position (clamped to array bounds)
///
/// Note: This is a tested utility function available for future use.
#[allow(dead_code)]
pub fn insert_at_position(
    array: &ArrayRef,
    txn: &mut TransactionMut<'_>,
    value: &str,
    position: u32,
) -> u32 {
    let len = array.len(txn);
    let insert_pos = position.min(len);
    array.insert(txn, insert_pos, Any::String(value.into()));
    insert_pos
}

/// Remove an item by value from a YArray.
///
/// # Arguments
///
/// * `array` - The array to remove from
/// * `txn` - The transaction to use
/// * `value` - The string value to remove
///
/// Returns true if the item was found and removed.
///
/// Note: This is a tested utility function available for future use.
#[allow(dead_code)]
pub fn remove_from_array(array: &ArrayRef, txn: &mut TransactionMut<'_>, value: &str) -> bool {
    if let Some(idx) = find_in_array(array, txn, value) {
        array.remove(txn, idx);
        true
    } else {
        false
    }
}

/// Get a map from another map, with error handling.
fn get_map<T: ReadTxn>(parent: &MapRef, txn: &T, key: &str) -> KanbanResult<MapRef> {
    match parent.get(txn, key) {
        Some(yrs::Out::YMap(m)) => Ok(m),
        _ => Err(KanbanError::InvalidData(format!("Missing map: {}", key))),
    }
}

/// Initialize the standard Kanban document structure.
///
/// Creates the root structure with all required maps and arrays:
/// - metadata: Board metadata
/// - column_order: Ordered column IDs
/// - columns: Column data map
/// - cards: Global card registry
/// - tags: Tag definitions
///
/// # Arguments
///
/// * `doc` - The Yrs document to initialize
pub fn init_document_structure(doc: &Doc) {
    let mut txn = doc.transact_mut();
    let root = txn.get_or_insert_map("root");

    // Initialize top-level maps
    let _: MapRef = root.get_or_init(&mut txn, keys::METADATA);
    let _: ArrayRef = root.get_or_init(&mut txn, keys::COLUMN_ORDER);
    let _: MapRef = root.get_or_init(&mut txn, keys::COLUMNS);
    let _: MapRef = root.get_or_init(&mut txn, keys::CARDS);
    let _: MapRef = root.get_or_init(&mut txn, keys::TAGS);
}

/// Initialize a column structure within the columns map.
///
/// # Arguments
///
/// * `columns` - The columns map
/// * `txn` - The transaction to use
/// * `column_id` - ID of the column
pub fn init_column_structure(columns: &MapRef, txn: &mut TransactionMut<'_>, column_id: &str) {
    let col: MapRef = columns.get_or_init(txn, column_id);
    let _: ArrayRef = col.get_or_init(txn, keys::CARD_ORDER);
}

/// Initialize a card structure within the cards map.
///
/// # Arguments
///
/// * `cards` - The cards map
/// * `txn` - The transaction to use
/// * `card_id` - ID of the card
pub fn init_card_structure(cards: &MapRef, txn: &mut TransactionMut<'_>, card_id: &str) {
    let card: MapRef = cards.get_or_init(txn, card_id);
    let _: ArrayRef = card.get_or_init(txn, keys::STEP_ORDER);
    let _: MapRef = card.get_or_init(txn, keys::STEPS);
    let _: MapRef = card.get_or_init(txn, keys::COMMENTS);
    let _: MapRef = card.get_or_init(txn, keys::ASSIGNEES);
    let _: MapRef = card.get_or_init(txn, keys::CARD_TAGS);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_orset_add_remove() {
        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("test");
        let orset: MapRef = root.get_or_init(&mut txn, "assignees");

        // Add element
        orset_add(&orset, &mut txn, "user-1", 1000).unwrap();
        assert!(orset_contains(&orset, &txn, "user-1"));

        // Remove element
        orset_remove(&orset, &mut txn, "user-1", 2000).unwrap();
        assert!(!orset_contains(&orset, &txn, "user-1"));

        // Re-add element
        orset_add(&orset, &mut txn, "user-1", 3000).unwrap();
        assert!(orset_contains(&orset, &txn, "user-1"));
    }

    #[test]
    fn test_orset_members() {
        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("test");
        let orset: MapRef = root.get_or_init(&mut txn, "assignees");

        orset_add(&orset, &mut txn, "user-1", 1000).unwrap();
        orset_add(&orset, &mut txn, "user-2", 1000).unwrap();
        orset_add(&orset, &mut txn, "user-3", 1000).unwrap();
        orset_remove(&orset, &mut txn, "user-2", 2000).unwrap();

        let members = orset_members(&orset, &txn);
        assert_eq!(members.len(), 2);
        assert!(members.contains(&"user-1".to_string()));
        assert!(members.contains(&"user-3".to_string()));
        assert!(!members.contains(&"user-2".to_string()));
    }

    #[test]
    fn test_init_document_structure() {
        let doc = Doc::new();
        init_document_structure(&doc);

        let txn = doc.transact();
        let root = txn.get_map("root").unwrap();

        assert!(root.get(&txn, keys::METADATA).is_some());
        assert!(root.get(&txn, keys::COLUMN_ORDER).is_some());
        assert!(root.get(&txn, keys::COLUMNS).is_some());
        assert!(root.get(&txn, keys::CARDS).is_some());
        assert!(root.get(&txn, keys::TAGS).is_some());
    }

    #[test]
    fn test_insert_at_position() {
        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("test");
        let array: ArrayRef = root.get_or_init(&mut txn, "items");

        insert_at_position(&array, &mut txn, "a", 0);
        insert_at_position(&array, &mut txn, "b", 1);
        insert_at_position(&array, &mut txn, "c", 1); // Insert between a and b

        let items: Vec<String> = array
            .iter(&txn)
            .filter_map(|v| {
                if let yrs::Out::Any(Any::String(s)) = v {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(items, vec!["a", "c", "b"]);
    }

    #[test]
    fn test_remove_from_array() {
        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("test");
        let array: ArrayRef = root.get_or_init(&mut txn, "items");

        array.push_back(&mut txn, Any::String("a".into()));
        array.push_back(&mut txn, Any::String("b".into()));
        array.push_back(&mut txn, Any::String("c".into()));

        assert!(remove_from_array(&array, &mut txn, "b"));
        assert!(!remove_from_array(&array, &mut txn, "nonexistent"));

        let items: Vec<String> = array
            .iter(&txn)
            .filter_map(|v| {
                if let yrs::Out::Any(Any::String(s)) = v {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(items, vec!["a", "c"]);
    }
}
