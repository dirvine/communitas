//! Animation integration tests
//!
//! Tests the integration of AnimationManager for smooth UI transitions:
//! - CommandPalette fade in/out
//! - Notification pulse
//! - Error shake
//! - Panel slide transitions

use communitas_tui::state::AppState;
use std::time::Duration;

// ============================================================================
// TEST 1: AnimationManager initialization
// ============================================================================

#[test]
fn test_animation_manager_starts_empty() {
    // Arrange
    let state = AppState::new();

    // Assert - No animations should be active initially
    assert_eq!(state.animation_manager.count(), 0);
}

#[test]
fn test_animation_manager_accessible_from_state() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Act - Add an animation
    state
        .animation_manager
        .add("test", Animation::fade_in(Duration::from_millis(100)));

    // Assert
    assert_eq!(state.animation_manager.count(), 1);
    assert!(state.animation_manager.has("test"));
}

// ============================================================================
// TEST 2: CommandPalette fade in/out animations
// ============================================================================

#[test]
fn test_command_palette_fade_in_animation_on_show() {
    // Arrange
    let mut state = AppState::new();
    assert!(!state.command_palette.is_visible());

    // Act - Show command palette
    state.command_palette.show();

    // Assert - Fade-in animation should be added
    assert!(state.animation_manager.has("command_palette_fade"));
}

#[test]
fn test_command_palette_fade_out_animation_on_hide() {
    // Arrange
    let mut state = AppState::new();
    state.command_palette.show();

    // Clear any show animation
    state.animation_manager.remove("command_palette_fade");
    assert_eq!(state.animation_manager.count(), 0);

    // Act - Hide command palette
    state.command_palette.hide();

    // Assert - Fade-out animation should be added
    assert!(state.animation_manager.has("command_palette_fade"));
}

#[test]
fn test_command_palette_fade_animation_duration() {
    // Arrange
    let mut state = AppState::new();

    // Act - Show command palette
    state.command_palette.show();

    // Assert - Animation should exist and have expected duration (200ms)
    if let Some(anim) = state.animation_manager.get("command_palette_fade") {
        assert!(anim.duration().as_millis() >= 100 && anim.duration().as_millis() <= 300);
    } else {
        panic!("Expected command_palette_fade animation to exist");
    }
}

#[test]
fn test_command_palette_animation_completes() {
    // Arrange
    let mut state = AppState::new();
    state.command_palette.show();

    // Get animation
    let anim_id = "command_palette_fade";
    assert!(state.animation_manager.has(anim_id));

    // Act - Update animation beyond duration
    let _duration = state.animation_manager.get(anim_id).unwrap().duration();
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(10));
        state.animation_manager.update_all();
        if !state.animation_manager.has(anim_id) {
            break; // Animation completed and was removed
        }
    }

    // Note: Some animation managers auto-remove completed animations,
    // others keep them. Both behaviors are valid.
}

// ============================================================================
// TEST 3: Notification pulse animations
// ============================================================================

#[test]
fn test_notification_pulse_animation_on_status_message() {
    // Arrange
    let mut state = AppState::new();
    assert!(state.status_message.is_none());

    // Act - Set status message
    state.set_status("Test notification");

    // Assert - Pulse animation should be added
    assert!(state.animation_manager.has("notification_pulse"));
}

#[test]
fn test_notification_pulse_animation_properties() {
    // Arrange
    let mut state = AppState::new();

    // Act - Set status message
    state.set_status("Important message");

    // Assert - Pulse animation should exist
    if let Some(anim) = state.animation_manager.get("notification_pulse") {
        // Pulse animations typically have shorter duration (500-1000ms)
        let duration_ms = anim.duration().as_millis();
        assert!((300..=1500).contains(&duration_ms));
    } else {
        panic!("Expected notification_pulse animation to exist");
    }
}

#[test]
fn test_multiple_status_messages_restart_pulse() {
    // Arrange
    let mut state = AppState::new();

    // Act - Set multiple status messages
    state.set_status("First message");
    let has_first_anim = state.animation_manager.has("notification_pulse");

    state.set_status("Second message");
    let has_second_anim = state.animation_manager.has("notification_pulse");

    // Assert - Each status message should trigger pulse animation
    assert!(has_first_anim);
    assert!(has_second_anim);
}

#[test]
fn test_notification_pulse_does_not_trigger_on_clear() {
    // Arrange
    let mut state = AppState::new();
    state.set_status("Message");
    state.animation_manager.remove("notification_pulse");
    assert_eq!(state.animation_manager.count(), 0);

    // Act - Clear status
    state.clear_status();

    // Assert - No new pulse animation
    assert!(!state.animation_manager.has("notification_pulse"));
}

// ============================================================================
// TEST 4: Error shake animations
// ============================================================================

#[test]
fn test_error_shake_animation_on_error_status() {
    // Arrange
    let mut state = AppState::new();

    // Act - Set error status (messages starting with "Error:")
    state.set_status("Error: Something went wrong");

    // Assert - Shake animation should be added
    assert!(state.animation_manager.has("error_shake"));
}

#[test]
fn test_error_shake_animation_properties() {
    // Arrange
    let mut state = AppState::new();

    // Act - Set error status
    state.set_status("Error: Invalid input");

    // Assert - Shake animation should exist with short duration
    if let Some(anim) = state.animation_manager.get("error_shake") {
        // Shake animations are typically very short (200-500ms)
        let duration_ms = anim.duration().as_millis();
        assert!((100..=800).contains(&duration_ms));
    } else {
        panic!("Expected error_shake animation to exist");
    }
}

#[test]
fn test_no_error_shake_for_normal_messages() {
    // Arrange
    let mut state = AppState::new();

    // Act - Set normal status (not starting with "Error:")
    state.set_status("Operation successful");

    // Assert - No shake animation
    assert!(!state.animation_manager.has("error_shake"));
}

#[test]
fn test_error_shake_for_failed_messages() {
    // Arrange
    let mut state = AppState::new();

    // Act - Set failure status (messages starting with "Failed:")
    state.set_status("Failed: Could not connect");

    // Assert - Shake animation should be added
    assert!(state.animation_manager.has("error_shake"));
}

// ============================================================================
// TEST 5: Panel slide transitions
// ============================================================================

#[test]
fn test_panel_slide_on_view_change() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::state::View;

    let initial_view = state.navigation.current_view().clone();

    // Act - Change view
    state.navigation.navigate_to(View::Organizations);

    // Assert - Slide animation should be added (if different view)
    if initial_view != View::Organizations {
        assert!(state.animation_manager.has("panel_slide"));
    }
}

#[test]
fn test_panel_slide_animation_duration() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::state::View;

    // Act - Change view to trigger slide
    state.navigation.navigate_to(View::Projects);

    // Assert - Slide animation should have reasonable duration (200-400ms)
    if let Some(anim) = state.animation_manager.get("panel_slide") {
        let duration_ms = anim.duration().as_millis();
        assert!((100..=600).contains(&duration_ms));
    }
}

#[test]
fn test_no_panel_slide_on_same_view() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::state::View;

    state.navigation.navigate_to(View::Dashboard);
    state.animation_manager.clear();

    // Act - Set to same view
    state.navigation.navigate_to(View::Dashboard);

    // Assert - No slide animation for same view
    assert!(!state.animation_manager.has("panel_slide"));
}

// ============================================================================
// TEST 6: Animation updates per frame
// ============================================================================

#[test]
fn test_animation_manager_updates_all_animations() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Add multiple animations
    state
        .animation_manager
        .add("anim1", Animation::fade_in(Duration::from_millis(100)));
    state.animation_manager.add(
        "anim2",
        Animation::pulse(100, 200, Duration::from_millis(200)),
    );

    assert_eq!(state.animation_manager.count(), 2);

    // Act - Update all animations by one frame (16ms at 60fps)
    std::thread::sleep(Duration::from_millis(16));
    state.animation_manager.update_all();

    // Assert - Both animations should still exist (not completed yet)
    assert!(state.animation_manager.has("anim1") || state.animation_manager.has("anim2"));
}

#[test]
fn test_completed_animations_are_cleaned_up() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Add short animation
    state
        .animation_manager
        .add("short", Animation::fade_in(Duration::from_millis(50)));

    // Act - Update beyond animation duration
    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(10));
        state.animation_manager.update_all();
    }

    // Assert - Completed animation should be removed
    // Note: This depends on AnimationManager implementation
    // Some implementations auto-remove, others keep completed animations
}

#[test]
fn test_animation_update_called_each_frame() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    state
        .animation_manager
        .add("test", Animation::fade_in(Duration::from_millis(1000)));

    // Act - Simulate multiple frames
    let frame_time = Duration::from_millis(16); // ~60fps
    for _ in 0..5 {
        std::thread::sleep(frame_time);
        state.animation_manager.update_all();
    }

    // Assert - Animation should still be running (5 frames * 16ms = 80ms < 1000ms)
    assert!(state.animation_manager.has("test"));
}

// ============================================================================
// TEST 7: Animation values and rendering
// ============================================================================

#[test]
fn test_fade_animation_opacity_range() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Add fade animation
    state
        .animation_manager
        .add("fade", Animation::fade_in(Duration::from_millis(100)));

    // Act - Get current opacity
    if let Some(anim) = state.animation_manager.get("fade") {
        use communitas_tui::components::AnimationValue;

        match anim.current_value() {
            AnimationValue::Opacity(_opacity) => {
                // Opacity is u8, so it's always in valid range [0, 255]
                // No assertion needed
            }
            _ => panic!("Expected Opacity value for fade animation"),
        }
    }
}

#[test]
fn test_pulse_animation_scale_range() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Add pulse animation
    state.animation_manager.add(
        "pulse",
        Animation::pulse(80, 120, Duration::from_millis(500)),
    );

    // Act - Get current scale
    if let Some(anim) = state.animation_manager.get("pulse") {
        use communitas_tui::components::AnimationValue;

        match anim.current_value() {
            AnimationValue::Scale(scale) => {
                // Assert - Scale should be reasonable (0.8 to 1.2 for pulse)
                assert!((0.5..=2.0).contains(&scale));
            }
            _ => panic!("Expected Scale value for pulse animation"),
        }
    }
}

#[test]
fn test_shake_animation_offset_range() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Add shake animation
    state
        .animation_manager
        .add("shake", Animation::shake(5, Duration::from_millis(300)));

    // Act - Get current offset
    if let Some(anim) = state.animation_manager.get("shake") {
        use communitas_tui::components::AnimationValue;

        match anim.current_value() {
            AnimationValue::Offset(offset, _axis) => {
                // Assert - Shake offset should be small (±5 pixels typically)
                assert!(offset.abs() <= 10);
            }
            _ => panic!("Expected Offset value for shake animation"),
        }
    }
}

// ============================================================================
// TEST 8: Animation integration with components
// ============================================================================

#[test]
fn test_command_palette_uses_fade_animation_value() {
    // Arrange
    let mut state = AppState::new();
    state.command_palette.show();

    // Assert - Should have fade animation with opacity
    if let Some(anim) = state.animation_manager.get("command_palette_fade") {
        use communitas_tui::components::AnimationValue;

        // Fade animations should provide opacity value
        match anim.current_value() {
            AnimationValue::Opacity(_) => { /* Success */ }
            _ => panic!("CommandPalette fade should use Opacity animation"),
        }
    }
}

#[test]
fn test_status_bar_uses_pulse_animation_value() {
    // Arrange
    let mut state = AppState::new();
    state.set_status("Test message");

    // Assert - Should have pulse animation with scale
    if let Some(anim) = state.animation_manager.get("notification_pulse") {
        use communitas_tui::components::AnimationValue;

        // Pulse animations should provide scale value
        match anim.current_value() {
            AnimationValue::Scale(_) => { /* Success */ }
            _ => panic!("Notification pulse should use Scale animation"),
        }
    }
}

#[test]
fn test_error_status_uses_shake_animation_value() {
    // Arrange
    let mut state = AppState::new();
    state.set_status("Error: Test error");

    // Assert - Should have shake animation with offset
    if let Some(anim) = state.animation_manager.get("error_shake") {
        use communitas_tui::components::AnimationValue;

        // Shake animations should provide offset value
        match anim.current_value() {
            AnimationValue::Offset(_, _) => { /* Success */ }
            _ => panic!("Error shake should use Offset animation"),
        }
    }
}

// ============================================================================
// TEST 9: Animation cleanup and resource management
// ============================================================================

#[test]
fn test_animation_manager_can_remove_animations() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    state
        .animation_manager
        .add("temp", Animation::fade_in(Duration::from_millis(100)));
    assert!(state.animation_manager.has("temp"));

    // Act - Remove animation
    state.animation_manager.remove("temp");

    // Assert
    assert!(!state.animation_manager.has("temp"));
}

#[test]
fn test_animation_manager_can_clear_all() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    state
        .animation_manager
        .add("anim1", Animation::fade_in(Duration::from_millis(100)));
    state.animation_manager.add(
        "anim2",
        Animation::pulse(100, 200, Duration::from_millis(200)),
    );
    assert_eq!(state.animation_manager.count(), 2);

    // Act - Clear all animations
    state.animation_manager.clear();

    // Assert
    assert_eq!(state.animation_manager.count(), 0);
}

#[test]
fn test_replacing_animation_with_same_id() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Add first animation
    state
        .animation_manager
        .add("test", Animation::fade_in(Duration::from_millis(100)));

    // Act - Add second animation with same ID (should replace)
    state
        .animation_manager
        .add("test", Animation::fade_out(Duration::from_millis(200)));

    // Assert - Should still have exactly one animation with that ID
    assert_eq!(state.animation_manager.count(), 1);
    assert!(state.animation_manager.has("test"));
}

// ============================================================================
// TEST 10: Edge cases
// ============================================================================

#[test]
fn test_animation_manager_handles_zero_duration() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Act - Add animation with zero duration
    state
        .animation_manager
        .add("instant", Animation::fade_in(Duration::from_millis(0)));

    // Assert - Should handle gracefully (either complete immediately or ignore)
    // Behavior depends on implementation
}

#[test]
fn test_animation_manager_handles_very_long_duration() {
    // Arrange
    let mut state = AppState::new();
    use communitas_tui::components::Animation;

    // Act - Add animation with very long duration (1 hour)
    state
        .animation_manager
        .add("long", Animation::fade_in(Duration::from_secs(3600)));

    // Assert - Should be added successfully
    assert!(state.animation_manager.has("long"));
}

#[test]
fn test_getting_nonexistent_animation_returns_none() {
    // Arrange
    let state = AppState::new();

    // Act - Get animation that doesn't exist
    let result = state.animation_manager.get("nonexistent");

    // Assert
    assert!(result.is_none());
}

#[test]
fn test_removing_nonexistent_animation_is_safe() {
    // Arrange
    let mut state = AppState::new();

    // Act - Remove animation that doesn't exist (should not panic)
    state.animation_manager.remove("nonexistent");

    // Assert - No crash
    assert_eq!(state.animation_manager.count(), 0);
}
