//! Animation system for smooth transitions and visual feedback
//!
//! Provides time-based animations with easing functions for polished UI interactions.
//! Supports fade, slide, pulse, shake, and scale animations.

use std::time::{Duration, Instant};

/// Easing function for smooth animation curves
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    /// Constant speed (y = x)
    Linear,
    /// Slow start, fast end (y = x²)
    EaseIn,
    /// Fast start, slow end (y = 1 - (1-x)²)
    EaseOut,
    /// Slow start and end, fast middle (y = smooth curve)
    EaseInOut,
    /// Bouncing effect at end
    Bounce,
}

impl EasingFunction {
    /// Apply easing function to linear progress (0.0 to 1.0)
    pub fn apply(&self, progress: f32) -> f32 {
        match self {
            EasingFunction::Linear => progress,
            EasingFunction::EaseIn => progress * progress,
            EasingFunction::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            EasingFunction::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
                }
            }
            EasingFunction::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                let mut p = progress;

                if p < 1.0 / d1 {
                    n1 * p * p
                } else if p < 2.0 / d1 {
                    p -= 1.5 / d1;
                    n1 * p * p + 0.75
                } else if p < 2.5 / d1 {
                    p -= 2.25 / d1;
                    n1 * p * p + 0.9375
                } else {
                    p -= 2.625 / d1;
                    n1 * p * p + 0.984375
                }
            }
        }
    }
}

/// Axis for directional animations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Type of animation effect
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationType {
    /// Fade in from transparent to opaque
    FadeIn { from: u8, to: u8 },
    /// Fade out from opaque to transparent
    FadeOut { from: u8, to: u8 },
    /// Slide along an axis
    Slide { from: i16, to: i16, axis: Axis },
    /// Scale size up or down
    Scale { from: f32, to: f32 },
    /// Pulse between min and max values
    Pulse { min: u8, max: u8 },
    /// Shake effect (rapid oscillation with decay)
    Shake { amplitude: i16, frequency: f32 },
}

/// Animation state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Running,
    Completed,
    Paused,
}

/// Core animation controller
#[derive(Debug, Clone)]
pub struct Animation {
    animation_type: AnimationType,
    duration: Duration,
    start_time: Instant,
    easing: EasingFunction,
    state: AnimationState,
    pause_time: Option<Instant>,
    elapsed_when_paused: Duration,
}

impl Animation {
    /// Create a new animation
    pub fn new(animation_type: AnimationType, duration: Duration, easing: EasingFunction) -> Self {
        Self {
            animation_type,
            duration,
            start_time: Instant::now(),
            easing,
            state: AnimationState::Running,
            pause_time: None,
            elapsed_when_paused: Duration::ZERO,
        }
    }

    /// Create a fade-in animation
    pub fn fade_in(duration: Duration) -> Self {
        Self::new(
            AnimationType::FadeIn { from: 0, to: 255 },
            duration,
            EasingFunction::EaseOut,
        )
    }

    /// Create a fade-out animation
    pub fn fade_out(duration: Duration) -> Self {
        Self::new(
            AnimationType::FadeOut { from: 255, to: 0 },
            duration,
            EasingFunction::EaseIn,
        )
    }

    /// Create a slide animation
    pub fn slide(from: i16, to: i16, axis: Axis, duration: Duration) -> Self {
        Self::new(
            AnimationType::Slide { from, to, axis },
            duration,
            EasingFunction::EaseInOut,
        )
    }

    /// Create a pulse animation (loops indefinitely)
    pub fn pulse(min: u8, max: u8, duration: Duration) -> Self {
        Self::new(
            AnimationType::Pulse { min, max },
            duration,
            EasingFunction::EaseInOut,
        )
    }

    /// Create a shake animation (error feedback)
    pub fn shake(amplitude: i16, duration: Duration) -> Self {
        Self::new(
            AnimationType::Shake {
                amplitude,
                frequency: 10.0,
            },
            duration,
            EasingFunction::Linear,
        )
    }

    /// Create a scale animation
    pub fn scale(from: f32, to: f32, duration: Duration) -> Self {
        Self::new(
            AnimationType::Scale { from, to },
            duration,
            EasingFunction::EaseInOut,
        )
    }

    /// Get current animation progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.state == AnimationState::Paused {
            let progress = self.elapsed_when_paused.as_secs_f32() / self.duration.as_secs_f32();
            return progress.clamp(0.0, 1.0);
        }

        let elapsed = self.start_time.elapsed();
        let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        progress.clamp(0.0, 1.0)
    }

    /// Get eased progress with easing function applied
    pub fn eased_progress(&self) -> f32 {
        self.easing.apply(self.progress())
    }

    /// Get current animation value based on type
    pub fn current_value(&self) -> AnimationValue {
        let progress = self.eased_progress();

        match &self.animation_type {
            AnimationType::FadeIn { from, to } | AnimationType::FadeOut { from, to } => {
                let value = *from as f32 + (*to as i16 - *from as i16) as f32 * progress;
                AnimationValue::Opacity(value as u8)
            }
            AnimationType::Slide { from, to, axis } => {
                let value = *from as f32 + (*to - *from) as f32 * progress;
                AnimationValue::Offset(value as i16, *axis)
            }
            AnimationType::Scale { from, to } => {
                let value = from + (to - from) * progress;
                AnimationValue::Scale(value)
            }
            AnimationType::Pulse { min, max } => {
                // Pulse oscillates using sine wave
                let sine_progress = (progress * std::f32::consts::PI * 2.0).sin();
                let value = *min as f32 + (*max - *min) as f32 * ((sine_progress + 1.0) / 2.0);
                AnimationValue::Opacity(value as u8)
            }
            AnimationType::Shake {
                amplitude,
                frequency,
            } => {
                // Shake decays over time with sine oscillation
                let decay = 1.0 - progress;
                let oscillation = (progress * frequency * std::f32::consts::PI * 2.0).sin();
                let value = (*amplitude as f32 * decay * oscillation) as i16;
                AnimationValue::Offset(value, Axis::Horizontal)
            }
        }
    }

    /// Check if animation is completed
    pub fn is_completed(&self) -> bool {
        self.state == AnimationState::Completed || self.progress() >= 1.0
    }

    /// Update animation state (call each frame)
    pub fn update(&mut self) {
        if self.state == AnimationState::Running && self.progress() >= 1.0 {
            // Special case: Pulse animations loop indefinitely
            if matches!(self.animation_type, AnimationType::Pulse { .. }) {
                self.reset();
            } else {
                self.state = AnimationState::Completed;
            }
        }
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
            self.pause_time = Some(Instant::now());
            self.elapsed_when_paused = self.start_time.elapsed();
        }
    }

    /// Resume the animation
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Running;
            self.start_time = Instant::now() - self.elapsed_when_paused;
            self.pause_time = None;
        }
    }

    /// Reset the animation to start
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.state = AnimationState::Running;
        self.pause_time = None;
        self.elapsed_when_paused = Duration::ZERO;
    }

    /// Get current state
    pub fn state(&self) -> AnimationState {
        self.state
    }

    /// Get animation type
    pub fn animation_type(&self) -> &AnimationType {
        &self.animation_type
    }

    /// Get duration
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Get easing function
    pub fn easing(&self) -> EasingFunction {
        self.easing
    }
}

/// Value returned by animation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationValue {
    Opacity(u8),
    Offset(i16, Axis),
    Scale(f32),
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // ========================================================================
    // Easing Function Tests
    // ========================================================================

    #[test]
    fn test_linear_easing() {
        let easing = EasingFunction::Linear;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_starts_slow() {
        let easing = EasingFunction::EaseIn;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.2) < 0.2); // Slower than linear at start
        assert!(easing.apply(0.8) < 0.8); // Quadratic is always below linear for 0 < x < 1
        assert!(easing.apply(0.8) > easing.apply(0.2)); // But accelerating
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_out_starts_fast() {
        let easing = EasingFunction::EaseOut;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.2) > 0.2); // Faster than linear at start
        assert!(easing.apply(0.8) > 0.8); // Quadratic ease-out is always above linear
        // Verify deceleration by checking rate of change decreases
        let delta_early = easing.apply(0.2) - easing.apply(0.0);
        let delta_late = easing.apply(1.0) - easing.apply(0.8);
        assert!(delta_late < delta_early); // Slowing down
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_out_symmetric() {
        let easing = EasingFunction::EaseInOut;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.2) < 0.2); // Slow at start
        assert_eq!(easing.apply(0.5), 0.5); // Mid-point
        assert!(easing.apply(0.8) > 0.8); // Fast in middle, slow at end
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_bounce_ends_at_one() {
        let easing = EasingFunction::Bounce;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.5) < 1.0); // Bounces below 1.0
        let final_value = easing.apply(1.0);
        assert!((final_value - 1.0).abs() < 0.01); // Ends very close to 1.0
    }

    // ========================================================================
    // Animation Creation Tests
    // ========================================================================

    #[test]
    fn test_create_fade_in() {
        let anim = Animation::fade_in(Duration::from_millis(500));
        assert!(matches!(
            anim.animation_type(),
            AnimationType::FadeIn { from: 0, to: 255 }
        ));
        assert_eq!(anim.duration(), Duration::from_millis(500));
        assert_eq!(anim.easing(), EasingFunction::EaseOut);
    }

    #[test]
    fn test_create_fade_out() {
        let anim = Animation::fade_out(Duration::from_millis(300));
        assert!(matches!(
            anim.animation_type(),
            AnimationType::FadeOut { from: 255, to: 0 }
        ));
        assert_eq!(anim.easing(), EasingFunction::EaseIn);
    }

    #[test]
    fn test_create_slide() {
        let anim = Animation::slide(0, 100, Axis::Horizontal, Duration::from_millis(400));
        assert!(matches!(
            anim.animation_type(),
            AnimationType::Slide {
                from: 0,
                to: 100,
                axis: Axis::Horizontal
            }
        ));
        assert_eq!(anim.easing(), EasingFunction::EaseInOut);
    }

    #[test]
    fn test_create_pulse() {
        let anim = Animation::pulse(100, 200, Duration::from_millis(1000));
        assert!(matches!(
            anim.animation_type(),
            AnimationType::Pulse { min: 100, max: 200 }
        ));
    }

    #[test]
    fn test_create_shake() {
        let anim = Animation::shake(10, Duration::from_millis(200));
        assert!(matches!(
            anim.animation_type(),
            AnimationType::Shake {
                amplitude: 10,
                frequency: 10.0
            }
        ));
    }

    #[test]
    fn test_create_scale() {
        let anim = Animation::scale(0.5, 1.0, Duration::from_millis(600));
        assert!(matches!(
            anim.animation_type(),
            AnimationType::Scale { from: 0.5, to: 1.0 }
        ));
    }

    // ========================================================================
    // Animation Progress Tests
    // ========================================================================

    #[test]
    fn test_initial_progress_is_zero() {
        let anim = Animation::fade_in(Duration::from_secs(1));
        assert!(anim.progress() < 0.1); // Very close to 0, accounting for test execution time
    }

    #[test]
    fn test_progress_increases_over_time() {
        let anim = Animation::fade_in(Duration::from_millis(100));
        let initial_progress = anim.progress();
        thread::sleep(Duration::from_millis(50));
        let mid_progress = anim.progress();
        assert!(mid_progress > initial_progress);
    }

    #[test]
    fn test_progress_clamped_at_one() {
        let anim = Animation::fade_in(Duration::from_millis(1));
        thread::sleep(Duration::from_millis(10));
        assert_eq!(anim.progress(), 1.0);
    }

    #[test]
    fn test_eased_progress_applies_easing() {
        let anim = Animation::new(
            AnimationType::FadeIn { from: 0, to: 255 },
            Duration::from_millis(100),
            EasingFunction::EaseIn,
        );
        thread::sleep(Duration::from_millis(50));
        let linear_progress = anim.progress();
        let eased = anim.eased_progress();
        // EaseIn should be slower than linear at midpoint
        assert!(eased < linear_progress);
    }

    // ========================================================================
    // Animation Value Tests
    // ========================================================================

    #[test]
    fn test_fade_in_value_at_start() {
        let anim = Animation::fade_in(Duration::from_secs(1));
        if let AnimationValue::Opacity(opacity) = anim.current_value() {
            assert!(opacity < 10); // Very close to 0
        } else {
            panic!("Expected Opacity value");
        }
    }

    #[test]
    fn test_fade_in_value_at_end() {
        let anim = Animation::fade_in(Duration::from_millis(1));
        thread::sleep(Duration::from_millis(10));
        if let AnimationValue::Opacity(opacity) = anim.current_value() {
            assert!(opacity > 250); // Very close to 255
        } else {
            panic!("Expected Opacity value");
        }
    }

    #[test]
    fn test_slide_value_interpolation() {
        let anim = Animation::slide(0, 100, Axis::Horizontal, Duration::from_millis(100));
        thread::sleep(Duration::from_millis(50));
        if let AnimationValue::Offset(value, axis) = anim.current_value() {
            assert_eq!(axis, Axis::Horizontal);
            assert!(value > 20 && value < 80); // Roughly mid-point
        } else {
            panic!("Expected Offset value");
        }
    }

    #[test]
    fn test_scale_value_interpolation() {
        let anim = Animation::scale(1.0, 2.0, Duration::from_millis(1));
        thread::sleep(Duration::from_millis(10));
        if let AnimationValue::Scale(scale) = anim.current_value() {
            assert!(scale > 1.9); // Very close to 2.0
        } else {
            panic!("Expected Scale value");
        }
    }

    #[test]
    fn test_pulse_oscillates() {
        let anim = Animation::pulse(100, 200, Duration::from_millis(1000));
        // At start
        if let AnimationValue::Opacity(opacity) = anim.current_value() {
            assert!((100..=200).contains(&opacity));
        }
    }

    #[test]
    fn test_shake_decays_over_time() {
        let anim_early = Animation::shake(100, Duration::from_millis(100));
        let anim_late = Animation::shake(100, Duration::from_millis(1));
        thread::sleep(Duration::from_millis(10));

        if let (AnimationValue::Offset(early, _), AnimationValue::Offset(late, _)) =
            (anim_early.current_value(), anim_late.current_value())
        {
            // Later in animation should have smaller shake amplitude
            assert!(late.abs() < early.abs() || late == 0);
        }
    }

    // ========================================================================
    // Animation State Tests
    // ========================================================================

    #[test]
    fn test_initial_state_is_running() {
        let anim = Animation::fade_in(Duration::from_secs(1));
        assert_eq!(anim.state(), AnimationState::Running);
    }

    #[test]
    fn test_completed_when_progress_at_one() {
        let mut anim = Animation::fade_in(Duration::from_millis(1));
        thread::sleep(Duration::from_millis(10));
        anim.update();
        assert!(anim.is_completed());
        assert_eq!(anim.state(), AnimationState::Completed);
    }

    #[test]
    fn test_pause_changes_state() {
        let mut anim = Animation::fade_in(Duration::from_secs(1));
        anim.pause();
        assert_eq!(anim.state(), AnimationState::Paused);
    }

    #[test]
    fn test_resume_changes_state() {
        let mut anim = Animation::fade_in(Duration::from_secs(1));
        anim.pause();
        anim.resume();
        assert_eq!(anim.state(), AnimationState::Running);
    }

    #[test]
    fn test_pause_freezes_progress() {
        let mut anim = Animation::fade_in(Duration::from_millis(100));
        thread::sleep(Duration::from_millis(30));
        let progress_before_pause = anim.progress();
        anim.pause();
        thread::sleep(Duration::from_millis(30));
        let progress_after_pause = anim.progress();
        // Progress should be frozen while paused
        assert!((progress_before_pause - progress_after_pause).abs() < 0.01);
    }

    #[test]
    fn test_reset_restarts_animation() {
        let mut anim = Animation::fade_in(Duration::from_millis(50));
        thread::sleep(Duration::from_millis(30));
        assert!(anim.progress() > 0.3);
        anim.reset();
        assert!(anim.progress() < 0.1); // Back to start
        assert_eq!(anim.state(), AnimationState::Running);
    }

    #[test]
    fn test_pulse_loops_indefinitely() {
        let mut anim = Animation::pulse(100, 200, Duration::from_millis(10));
        thread::sleep(Duration::from_millis(20));
        anim.update();
        // Pulse should reset and keep running
        assert_eq!(anim.state(), AnimationState::Running);
    }

    #[test]
    fn test_non_pulse_completes_once() {
        let mut anim = Animation::fade_in(Duration::from_millis(10));
        thread::sleep(Duration::from_millis(20));
        anim.update();
        assert_eq!(anim.state(), AnimationState::Completed);
    }

    // ========================================================================
    // AnimationType Tests
    // ========================================================================

    #[test]
    fn test_animation_type_equality() {
        let fade1 = AnimationType::FadeIn { from: 0, to: 255 };
        let fade2 = AnimationType::FadeIn { from: 0, to: 255 };
        let fade3 = AnimationType::FadeIn { from: 0, to: 200 };
        assert_eq!(fade1, fade2);
        assert_ne!(fade1, fade3);
    }

    #[test]
    fn test_axis_equality() {
        assert_eq!(Axis::Horizontal, Axis::Horizontal);
        assert_eq!(Axis::Vertical, Axis::Vertical);
        assert_ne!(Axis::Horizontal, Axis::Vertical);
    }

    // ========================================================================
    // AnimationValue Tests
    // ========================================================================

    #[test]
    fn test_animation_value_equality() {
        let val1 = AnimationValue::Opacity(128);
        let val2 = AnimationValue::Opacity(128);
        let val3 = AnimationValue::Opacity(64);
        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_animation_value_variants() {
        let opacity = AnimationValue::Opacity(255);
        let offset = AnimationValue::Offset(50, Axis::Horizontal);
        let scale = AnimationValue::Scale(1.5);

        assert!(matches!(opacity, AnimationValue::Opacity(_)));
        assert!(matches!(offset, AnimationValue::Offset(_, _)));
        assert!(matches!(scale, AnimationValue::Scale(_)));
    }
}
