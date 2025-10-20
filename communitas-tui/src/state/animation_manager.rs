//! Animation manager for handling multiple concurrent animations
//!
//! Provides centralized management of UI animations with automatic cleanup
//! of completed animations.

use crate::components::Animation;
use std::collections::HashMap;

/// Manages multiple concurrent animations
#[derive(Debug)]
pub struct AnimationManager {
    /// Active animations indexed by unique ID
    active_animations: HashMap<String, Animation>,
}

impl AnimationManager {
    /// Create a new animation manager
    pub fn new() -> Self {
        Self {
            active_animations: HashMap::new(),
        }
    }

    /// Add a new animation with a unique ID
    ///
    /// If an animation with the same ID already exists, it will be replaced.
    pub fn add(&mut self, id: impl Into<String>, animation: Animation) {
        self.active_animations.insert(id.into(), animation);
    }

    /// Update all active animations and remove completed ones
    ///
    /// Should be called every frame to advance animation progress.
    pub fn update_all(&mut self) {
        self.active_animations.retain(|_, anim| {
            anim.update();
            !anim.is_completed()
        });
    }

    /// Get an animation by ID
    pub fn get(&self, id: &str) -> Option<&Animation> {
        self.active_animations.get(id)
    }

    /// Get a mutable reference to an animation by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Animation> {
        self.active_animations.get_mut(id)
    }

    /// Check if an animation exists
    pub fn has(&self, id: &str) -> bool {
        self.active_animations.contains_key(id)
    }

    /// Remove an animation by ID
    pub fn remove(&mut self, id: &str) -> Option<Animation> {
        self.active_animations.remove(id)
    }

    /// Clear all animations
    pub fn clear(&mut self) {
        self.active_animations.clear();
    }

    /// Get number of active animations
    pub fn count(&self) -> usize {
        self.active_animations.len()
    }

    /// Get all animation IDs
    pub fn ids(&self) -> Vec<String> {
        self.active_animations.keys().cloned().collect()
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_animation_manager_creation() {
        let manager = AnimationManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_animation_manager_default() {
        let manager = AnimationManager::default();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_add_animation() {
        let mut manager = AnimationManager::new();
        let anim = Animation::fade_in(Duration::from_millis(100));
        manager.add("fade", anim);
        assert_eq!(manager.count(), 1);
        assert!(manager.has("fade"));
    }

    #[test]
    fn test_add_replaces_existing() {
        let mut manager = AnimationManager::new();
        manager.add("fade", Animation::fade_in(Duration::from_millis(100)));
        manager.add("fade", Animation::fade_out(Duration::from_millis(200)));
        assert_eq!(manager.count(), 1); // Still just one
    }

    #[test]
    fn test_get_animation() {
        let mut manager = AnimationManager::new();
        let anim = Animation::fade_in(Duration::from_millis(100));
        manager.add("fade", anim);

        let retrieved = manager.get("fade");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_get_nonexistent() {
        let manager = AnimationManager::new();
        assert!(manager.get("nonexistent").is_none());
    }

    #[test]
    fn test_get_mut() {
        let mut manager = AnimationManager::new();
        manager.add("fade", Animation::fade_in(Duration::from_millis(100)));

        let anim = manager.get_mut("fade");
        assert!(anim.is_some());

        // Modify the animation
        if let Some(a) = anim {
            a.pause();
        }
    }

    #[test]
    fn test_has_animation() {
        let mut manager = AnimationManager::new();
        assert!(!manager.has("fade"));

        manager.add("fade", Animation::fade_in(Duration::from_millis(100)));
        assert!(manager.has("fade"));
    }

    #[test]
    fn test_remove_animation() {
        let mut manager = AnimationManager::new();
        manager.add("fade", Animation::fade_in(Duration::from_millis(100)));
        assert_eq!(manager.count(), 1);

        let removed = manager.remove("fade");
        assert!(removed.is_some());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut manager = AnimationManager::new();
        let removed = manager.remove("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_clear_all() {
        let mut manager = AnimationManager::new();
        manager.add("fade1", Animation::fade_in(Duration::from_millis(100)));
        manager.add("fade2", Animation::fade_out(Duration::from_millis(100)));
        manager.add(
            "slide",
            Animation::slide(
                0,
                100,
                crate::components::Axis::Horizontal,
                Duration::from_millis(100),
            ),
        );
        assert_eq!(manager.count(), 3);

        manager.clear();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_count() {
        let mut manager = AnimationManager::new();
        assert_eq!(manager.count(), 0);

        manager.add("fade1", Animation::fade_in(Duration::from_millis(100)));
        assert_eq!(manager.count(), 1);

        manager.add("fade2", Animation::fade_out(Duration::from_millis(100)));
        assert_eq!(manager.count(), 2);

        manager.remove("fade1");
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_ids() {
        let mut manager = AnimationManager::new();
        manager.add("fade", Animation::fade_in(Duration::from_millis(100)));
        manager.add(
            "slide",
            Animation::slide(
                0,
                100,
                crate::components::Axis::Horizontal,
                Duration::from_millis(100),
            ),
        );

        let ids = manager.ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"fade".to_string()));
        assert!(ids.contains(&"slide".to_string()));
    }

    #[test]
    fn test_update_all_removes_completed() {
        let mut manager = AnimationManager::new();

        // Add a very short animation that will complete immediately
        manager.add("short", Animation::fade_in(Duration::from_millis(1)));

        // Sleep to ensure animation completes
        std::thread::sleep(Duration::from_millis(10));

        // Update all - should remove completed animation
        manager.update_all();

        // The short animation should be gone
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_update_all_keeps_running() {
        let mut manager = AnimationManager::new();

        // Add a long animation
        manager.add("long", Animation::fade_in(Duration::from_secs(10)));

        // Update
        manager.update_all();

        // Should still be there
        assert_eq!(manager.count(), 1);
        assert!(manager.has("long"));
    }

    #[test]
    fn test_multiple_animations_mixed_states() {
        let mut manager = AnimationManager::new();

        // Add multiple animations with different durations
        manager.add("short", Animation::fade_in(Duration::from_millis(1)));
        manager.add("long", Animation::fade_in(Duration::from_secs(10)));
        manager.add("medium", Animation::fade_in(Duration::from_millis(500)));

        assert_eq!(manager.count(), 3);

        // Sleep to complete short animation
        std::thread::sleep(Duration::from_millis(10));

        // Update all
        manager.update_all();

        // Short should be removed, others should remain
        assert_eq!(manager.count(), 2);
        assert!(!manager.has("short"));
        assert!(manager.has("long"));
        assert!(manager.has("medium"));
    }

    #[test]
    fn test_pulse_animation_loops() {
        let mut manager = AnimationManager::new();

        // Pulse animations loop indefinitely
        manager.add(
            "pulse",
            Animation::pulse(100, 200, Duration::from_millis(10)),
        );

        // Sleep past the duration
        std::thread::sleep(Duration::from_millis(25));

        // Update
        manager.update_all();

        // Pulse should still be there (loops)
        assert!(manager.has("pulse"));
    }
}
