//! Emoji data module providing hardcoded emoji entries organized by category.
//!
//! This module exposes a static list of ~200 common emojis, quick-reaction shortcuts,
//! and helper functions for searching and filtering by category.

/// The category an emoji belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiCategory {
    /// Smiley faces and emotion emojis.
    Smileys,
    /// People, hand gestures, and body-language emojis.
    People,
    /// Animals, plants, and nature emojis.
    Nature,
    /// Food, drink, and cooking emojis.
    Food,
    /// Transportation, places, and travel emojis.
    Travel,
    /// Everyday objects, tools, and technology emojis.
    Objects,
    /// Symbols, hearts, and abstract icons.
    Symbols,
    /// National and special-purpose flags.
    Flags,
}

/// A single emoji entry with its display glyph, searchable name, and category.
#[derive(Debug, Clone, PartialEq)]
pub struct EmojiEntry {
    /// The emoji character(s) to display.
    pub emoji: &'static str,
    /// A lowercase, space-separated name used for search.
    pub name: &'static str,
    /// Which category this emoji belongs to.
    pub category: EmojiCategory,
}

/// Six most common reaction emojis for the quick-reaction bar.
pub const QUICK_REACTIONS: [&str; 6] = ["👍", "❤️", "😂", "😮", "😢", "🔥"];

/// All supported emojis, approximately 200 of the most common across all categories.
pub static ALL_EMOJIS: &[EmojiEntry] = &[
    // ── Smileys ─────────────────────────────────────────────────────────────
    EmojiEntry { emoji: "😀", name: "grinning face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😃", name: "grinning face with big eyes", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😄", name: "grinning face with smiling eyes", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😁", name: "beaming face with smiling eyes", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😆", name: "grinning squinting face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😅", name: "grinning face with sweat", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🤣", name: "rolling on the floor laughing", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😂", name: "face with tears of joy", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🙂", name: "slightly smiling face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🙃", name: "upside down face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😉", name: "winking face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😊", name: "smiling face with smiling eyes", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😇", name: "smiling face with halo", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🥰", name: "smiling face with hearts", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😍", name: "smiling face with heart eyes", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🤩", name: "star struck", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😘", name: "face blowing a kiss", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😗", name: "kissing face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😚", name: "kissing face with closed eyes", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😙", name: "kissing face with smiling eyes", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🥲", name: "smiling face with tear", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😋", name: "face savoring food", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😛", name: "face with tongue", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😜", name: "winking face with tongue", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🤪", name: "zany face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "😝", name: "squinting face with tongue", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🤑", name: "money mouth face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🤗", name: "smiling face with open hands hugging", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🤭", name: "face with hand over mouth", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🤫", name: "shushing face", category: EmojiCategory::Smileys },
    EmojiEntry { emoji: "🤔", name: "thinking face", category: EmojiCategory::Smileys },

    // ── People ───────────────────────────────────────────────────────────────
    EmojiEntry { emoji: "👋", name: "waving hand", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤚", name: "raised back of hand", category: EmojiCategory::People },
    EmojiEntry { emoji: "🖐️", name: "hand with fingers splayed", category: EmojiCategory::People },
    EmojiEntry { emoji: "✋", name: "raised hand", category: EmojiCategory::People },
    EmojiEntry { emoji: "🖖", name: "vulcan salute", category: EmojiCategory::People },
    EmojiEntry { emoji: "👌", name: "ok hand", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤌", name: "pinched fingers", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤏", name: "pinching hand", category: EmojiCategory::People },
    EmojiEntry { emoji: "✌️", name: "victory hand peace", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤞", name: "crossed fingers", category: EmojiCategory::People },
    EmojiEntry { emoji: "🫰", name: "hand with index finger and thumb crossed", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤟", name: "love you gesture", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤘", name: "sign of the horns", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤙", name: "call me hand", category: EmojiCategory::People },
    EmojiEntry { emoji: "👈", name: "backhand index pointing left", category: EmojiCategory::People },
    EmojiEntry { emoji: "👉", name: "backhand index pointing right", category: EmojiCategory::People },
    EmojiEntry { emoji: "👆", name: "backhand index pointing up", category: EmojiCategory::People },
    EmojiEntry { emoji: "🖕", name: "middle finger", category: EmojiCategory::People },
    EmojiEntry { emoji: "👇", name: "backhand index pointing down", category: EmojiCategory::People },
    EmojiEntry { emoji: "☝️", name: "index pointing up", category: EmojiCategory::People },
    EmojiEntry { emoji: "👍", name: "thumbs up", category: EmojiCategory::People },
    EmojiEntry { emoji: "👎", name: "thumbs down", category: EmojiCategory::People },
    EmojiEntry { emoji: "✊", name: "raised fist", category: EmojiCategory::People },
    EmojiEntry { emoji: "👊", name: "oncoming fist", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤛", name: "left facing fist", category: EmojiCategory::People },
    EmojiEntry { emoji: "🤜", name: "right facing fist", category: EmojiCategory::People },
    EmojiEntry { emoji: "👏", name: "clapping hands", category: EmojiCategory::People },
    EmojiEntry { emoji: "🙌", name: "raising hands", category: EmojiCategory::People },
    EmojiEntry { emoji: "🫶", name: "heart hands", category: EmojiCategory::People },
    EmojiEntry { emoji: "👐", name: "open hands", category: EmojiCategory::People },

    // ── Nature ───────────────────────────────────────────────────────────────
    EmojiEntry { emoji: "🐶", name: "dog face", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐱", name: "cat face", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐭", name: "mouse face", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐹", name: "hamster", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐰", name: "rabbit face", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🦊", name: "fox", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐻", name: "bear", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐼", name: "panda", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐨", name: "koala", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐯", name: "tiger face", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🦁", name: "lion", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐮", name: "cow face", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐷", name: "pig face", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐸", name: "frog", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐵", name: "monkey face", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🙈", name: "see no evil monkey", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🙉", name: "hear no evil monkey", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🙊", name: "speak no evil monkey", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐒", name: "monkey", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🐔", name: "chicken", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🌸", name: "cherry blossom", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🌺", name: "hibiscus", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🌻", name: "sunflower", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🌹", name: "rose", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🌿", name: "herb leaf", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🍀", name: "four leaf clover", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🌊", name: "water wave ocean", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "🌙", name: "crescent moon", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "⭐", name: "star", category: EmojiCategory::Nature },
    EmojiEntry { emoji: "☀️", name: "sun", category: EmojiCategory::Nature },

    // ── Food ────────────────────────────────────────────────────────────────
    EmojiEntry { emoji: "🍎", name: "red apple", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍐", name: "pear", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍊", name: "tangerine orange", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍋", name: "lemon", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍌", name: "banana", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍉", name: "watermelon", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍇", name: "grapes", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍓", name: "strawberry", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🫐", name: "blueberries", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍈", name: "melon", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍒", name: "cherries", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍑", name: "peach", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🥭", name: "mango", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍍", name: "pineapple", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🥥", name: "coconut", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🥝", name: "kiwi fruit", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍅", name: "tomato", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍆", name: "eggplant aubergine", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🥑", name: "avocado", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🫛", name: "pea pod", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍕", name: "pizza", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍔", name: "hamburger burger", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍟", name: "french fries", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🌮", name: "taco", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍜", name: "steaming bowl noodles ramen", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍣", name: "sushi", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍩", name: "doughnut donut", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🍪", name: "cookie", category: EmojiCategory::Food },
    EmojiEntry { emoji: "🎂", name: "birthday cake", category: EmojiCategory::Food },
    EmojiEntry { emoji: "☕", name: "hot beverage coffee", category: EmojiCategory::Food },

    // ── Travel ───────────────────────────────────────────────────────────────
    EmojiEntry { emoji: "🚗", name: "automobile car", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚕", name: "taxi", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚙", name: "sport utility vehicle suv", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚌", name: "bus", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚎", name: "trolleybus", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🏎️", name: "racing car", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚓", name: "police car", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚑", name: "ambulance", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚒", name: "fire engine", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚐", name: "minibus", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🛻", name: "pickup truck", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚚", name: "delivery truck", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚛", name: "articulated lorry semi truck", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚜", name: "tractor", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🏍️", name: "motorcycle motorbike", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🛵", name: "motor scooter", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚲", name: "bicycle bike", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🛴", name: "kick scooter", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🛹", name: "skateboard", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🛼", name: "roller skate", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "✈️", name: "airplane plane", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚀", name: "rocket", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🛸", name: "flying saucer ufo", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "⛵", name: "sailboat", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🚢", name: "ship boat", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🏔️", name: "snow capped mountain", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🏖️", name: "beach with umbrella", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🏙️", name: "cityscape city", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🗺️", name: "world map", category: EmojiCategory::Travel },
    EmojiEntry { emoji: "🧭", name: "compass", category: EmojiCategory::Travel },

    // ── Objects ──────────────────────────────────────────────────────────────
    EmojiEntry { emoji: "💻", name: "laptop computer", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🖥️", name: "desktop computer monitor", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🖨️", name: "printer", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "⌨️", name: "keyboard", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🖱️", name: "computer mouse", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🖲️", name: "trackball", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "💾", name: "floppy disk", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "💿", name: "optical disk cd", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📱", name: "mobile phone smartphone", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📲", name: "mobile phone with arrow", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "☎️", name: "telephone", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📞", name: "telephone receiver", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📟", name: "pager", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📠", name: "fax machine", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📺", name: "television tv", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📻", name: "radio", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🎙️", name: "studio microphone", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🎚️", name: "level slider", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🎛️", name: "control knobs", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "⏰", name: "alarm clock", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📷", name: "camera", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🔑", name: "key", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🔒", name: "locked", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🔓", name: "unlocked", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🔔", name: "bell notification", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📚", name: "books", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "✏️", name: "pencil", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "📎", name: "paperclip", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🔧", name: "wrench tool", category: EmojiCategory::Objects },
    EmojiEntry { emoji: "🔥", name: "fire flame hot", category: EmojiCategory::Objects },

    // ── Symbols ──────────────────────────────────────────────────────────────
    EmojiEntry { emoji: "❤️", name: "red heart love", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🧡", name: "orange heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💛", name: "yellow heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💚", name: "green heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💙", name: "blue heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💜", name: "purple heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🖤", name: "black heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🤍", name: "white heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🤎", name: "brown heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💔", name: "broken heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "❣️", name: "heart exclamation", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💕", name: "two hearts", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💞", name: "revolving hearts", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💓", name: "beating heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💗", name: "growing heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💖", name: "sparkling heart", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💘", name: "heart with arrow cupid", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💝", name: "heart with ribbon", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🌟", name: "glowing star", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "✨", name: "sparkles", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "⚡", name: "high voltage lightning", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "💯", name: "hundred points", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🎉", name: "party popper celebration", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🎊", name: "confetti ball celebration", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🏆", name: "trophy award winner", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🥇", name: "first place gold medal", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "✅", name: "check mark button", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "❌", name: "cross mark x", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "⚠️", name: "warning caution", category: EmojiCategory::Symbols },
    EmojiEntry { emoji: "🔴", name: "red circle dot", category: EmojiCategory::Symbols },

    // ── Flags ────────────────────────────────────────────────────────────────
    EmojiEntry { emoji: "🏁", name: "chequered flag finish", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🚩", name: "triangular flag", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🎌", name: "crossed flags japan", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🏴", name: "black flag", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🏳️", name: "white flag", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🏳️‍🌈", name: "rainbow flag pride", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🏳️‍⚧️", name: "transgender flag", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🇺🇸", name: "united states flag us america", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🇬🇧", name: "united kingdom flag uk britain", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🇨🇦", name: "canada flag canadian", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🇦🇺", name: "australia flag australian", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🇩🇪", name: "germany flag german", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🇫🇷", name: "france flag french", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🇯🇵", name: "japan flag japanese", category: EmojiCategory::Flags },
    EmojiEntry { emoji: "🇰🇷", name: "south korea flag korean", category: EmojiCategory::Flags },
];

/// Search emojis by name using a case-insensitive substring match.
///
/// Passing an empty query returns all emojis.
///
/// # Examples
/// ```
/// let results = communitas_dioxus::components::emoji_data::search("heart");
/// assert!(!results.is_empty());
/// ```
pub fn search(query: &str) -> Vec<&'static EmojiEntry> {
    let lower = query.to_lowercase();
    ALL_EMOJIS
        .iter()
        .filter(|e| lower.is_empty() || e.name.contains(lower.as_str()))
        .collect()
}

/// Return all emojis belonging to the given category.
///
/// # Examples
/// ```
/// use communitas_dioxus::components::emoji_data::{EmojiCategory, by_category};
/// let smileys = by_category(EmojiCategory::Smileys);
/// assert!(!smileys.is_empty());
/// ```
pub fn by_category(category: EmojiCategory) -> Vec<&'static EmojiEntry> {
    ALL_EMOJIS
        .iter()
        .filter(|e| e.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_empty_returns_all() {
        let results = search("");
        assert_eq!(results.len(), ALL_EMOJIS.len());
    }

    #[test]
    fn search_smile_finds_smile_related() {
        // "smiling" is the canonical term used in emoji names (e.g. "smiling face with …")
        let results = search("smiling");
        assert!(!results.is_empty(), "Expected to find smile-related emojis");
        for entry in &results {
            assert!(entry.name.contains("smiling"), "Expected 'smiling' in name: {}", entry.name);
        }
    }

    #[test]
    fn search_case_insensitive_heart() {
        let lower = search("heart");
        let upper = search("HEART");
        assert!(!upper.is_empty(), "Expected heart emojis for 'HEART'");
        assert_eq!(
            lower.len(),
            upper.len(),
            "Case-insensitive search should return the same results"
        );
    }

    #[test]
    fn by_category_smileys_returns_only_smileys() {
        let smileys = by_category(EmojiCategory::Smileys);
        assert!(!smileys.is_empty());
        for entry in smileys {
            assert_eq!(
                entry.category,
                EmojiCategory::Smileys,
                "Expected Smileys category for emoji: {}",
                entry.emoji
            );
        }
    }

    #[test]
    fn quick_reactions_has_six_entries() {
        assert_eq!(QUICK_REACTIONS.len(), 6);
    }

    #[test]
    fn no_duplicate_emojis_in_all_emojis() {
        let mut seen = std::collections::HashSet::new();
        for entry in ALL_EMOJIS {
            assert!(
                seen.insert(entry.emoji),
                "Duplicate emoji found: {}",
                entry.emoji
            );
        }
    }
}
