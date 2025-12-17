# Communitas Dioxus UI/UX Component Architecture Plan

## Executive Summary

This plan outlines the UI/UX component architecture for a polished, production-ready Communitas application built with Dioxus (Rust cross-platform UI framework). The design targets Windows, Linux, macOS, iOS, and Android with professional polish comparable to WhatsApp and Slack.

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Component Hierarchy](#2-component-hierarchy)
3. [Reusable Component Library](#3-reusable-component-library)
4. [Theming System](#4-theming-system)
5. [Navigation Patterns](#5-navigation-patterns)
6. [Platform-Specific Adaptations](#6-platform-specific-adaptations)
7. [State Management](#7-state-management)
8. [Implementation Priority](#8-implementation-priority)
9. [Critical Files Reference](#9-critical-files-reference)

---

## 1. Architecture Overview

### 1.1 Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **UI Framework** | Dioxus 0.6+ | Cross-platform Rust UI |
| **Bindings** | UniFFI (communitas-bindings) | Rust core <-> UI bridge |
| **Styling** | Tailwind CSS + inline styles | Responsive design |
| **State** | Dioxus Signals + Context API | Reactive state management |
| **Navigation** | Custom router | Platform-aware navigation |

### 1.2 Project Structure

```
communitas-dioxus/
├── Cargo.toml
├── Dioxus.toml
├── assets/
│   ├── styles/
│   │   ├── tailwind.css
│   │   ├── themes/
│   │   │   ├── light.css
│   │   │   └── dark.css
│   │   └── animations.css
│   ├── fonts/
│   │   └── Inter/
│   └── icons/
├── src/
│   ├── main.rs
│   ├── app.rs                    # Root App component
│   ├── components/
│   │   ├── mod.rs
│   │   ├── atoms/                # Smallest building blocks
│   │   │   ├── mod.rs
│   │   │   ├── avatar.rs         # FourWordAvatar
│   │   │   ├── button.rs         # Button variants
│   │   │   ├── badge.rs          # Notification badges
│   │   │   ├── input.rs          # Text input
│   │   │   ├── icon.rs           # Icon wrapper
│   │   │   ├── presence.rs       # Online/offline indicator
│   │   │   └── spinner.rs        # Loading spinner
│   │   ├── molecules/            # Composed from atoms
│   │   │   ├── mod.rs
│   │   │   ├── four_word_input.rs
│   │   │   ├── message_bubble.rs
│   │   │   ├── message_composer.rs
│   │   │   ├── contact_row.rs
│   │   │   ├── entity_row.rs
│   │   │   ├── file_row.rs
│   │   │   ├── document_row.rs
│   │   │   ├── call_controls.rs
│   │   │   └── search_bar.rs
│   │   ├── organisms/            # Complex UI sections
│   │   │   ├── mod.rs
│   │   │   ├── sidebar.rs
│   │   │   ├── profile_header.rs
│   │   │   ├── chat_panel.rs
│   │   │   ├── drive_panel.rs
│   │   │   ├── document_editor.rs
│   │   │   ├── call_view.rs
│   │   │   ├── contacts_panel.rs
│   │   │   └── network_settings.rs
│   │   └── modals/               # Overlay dialogs
│   │       ├── mod.rs
│   │       ├── create_entity.rs
│   │       ├── add_contact.rs
│   │       ├── incoming_call.rs
│   │       └── confirm_dialog.rs
│   ├── screens/
│   │   ├── mod.rs
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   ├── welcome.rs
│   │   │   ├── login.rs
│   │   │   ├── create_identity.rs
│   │   │   └── vault_selection.rs
│   │   ├── main/
│   │   │   ├── mod.rs
│   │   │   ├── home.rs
│   │   │   ├── chat.rs
│   │   │   ├── contact_chat.rs
│   │   │   ├── drive.rs
│   │   │   ├── documents.rs
│   │   │   └── call.rs
│   │   └── settings/
│   │       ├── mod.rs
│   │       ├── profile.rs
│   │       └── network.rs
│   ├── layouts/
│   │   ├── mod.rs
│   │   ├── auth_layout.rs        # Centered auth screens
│   │   ├── main_layout.rs        # Sidebar + detail
│   │   └── mobile_layout.rs      # Tab-based mobile
│   ├── hooks/
│   │   ├── mod.rs
│   │   ├── use_client.rs         # CommunitasClient access
│   │   ├── use_auth.rs           # Authentication state
│   │   ├── use_entities.rs       # Entity management
│   │   ├── use_messages.rs       # Messaging
│   │   ├── use_contacts.rs       # Contact management
│   │   ├── use_presence.rs       # Online status
│   │   ├── use_network.rs        # Network state
│   │   └── use_platform.rs       # Platform detection
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── colors.rs             # Color palette
│   │   ├── typography.rs         # Font scales
│   │   ├── spacing.rs            # Spacing tokens
│   │   └── animations.rs         # Animation presets
│   ├── platform/
│   │   ├── mod.rs
│   │   ├── desktop.rs            # Desktop-specific
│   │   ├── mobile.rs             # Mobile-specific
│   │   └── web.rs                # Web-specific
│   ├── router.rs                 # Navigation router
│   └── utils/
│       ├── mod.rs
│       ├── four_words.rs         # Four-word utilities
│       ├── format.rs             # Formatting helpers
│       └── gradient.rs           # Gradient generation
```

---

## 2. Component Hierarchy

### 2.1 Application Tree

```
App
├── ThemeProvider
│   └── Router
│       ├── AuthRoutes (unauthenticated)
│       │   ├── AuthLayout
│       │   │   ├── WelcomeScreen
│       │   │   ├── LoginScreen
│       │   │   ├── CreateIdentityScreen
│       │   │   └── VaultSelectionScreen
│       │
│       └── MainRoutes (authenticated)
│           ├── MainLayout
│           │   ├── ProfileHeader
│           │   ├── Sidebar
│           │   │   ├── OrganisationsSection
│           │   │   │   └── OrganisationNode[]
│           │   │   │       ├── ProjectsSection
│           │   │   │       ├── ChannelsSection
│           │   │   │       └── GroupsSection
│           │   │   └── PersonalSection
│           │   │       ├── GroupsSection
│           │   │       └── ContactsSection
│           │   │
│           │   └── DetailPane
│           │       ├── WelcomePane (default)
│           │       ├── EntityDetailPane
│           │       │   ├── ChatTab
│           │       │   ├── DriveTab
│           │       │   ├── DocumentsTab
│           │       │   └── DetailsTab
│           │       ├── ContactChatPane
│           │       ├── CallPane
│           │       └── SettingsPane
│           │
│           └── ModalLayer
│               ├── CreateEntityModal
│               ├── AddContactModal
│               ├── IncomingCallModal
│               └── ConfirmDialog
```

### 2.2 Mobile Layout (iOS/Android)

```
MobileApp
├── ThemeProvider
│   └── Router
│       ├── AuthStack
│       │   └── (same as desktop)
│       │
│       └── MainTabBar
│           ├── ChatsTab
│           │   └── ChatList → ChatDetail
│           ├── ContactsTab
│           │   └── ContactList → ContactChat
│           ├── FilesTab
│           │   └── DriveList → FileDetail
│           └── SettingsTab
│               └── SettingsMenu
```

---

## 3. Reusable Component Library

### 3.1 Atoms

#### Avatar Component

```rust
// src/components/atoms/avatar.rs

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum AvatarSize {
    XS,  // 24px
    SM,  // 32px
    MD,  // 48px
    LG,  // 64px
    XL,  // 96px
}

impl AvatarSize {
    pub fn dimension(&self) -> u32 {
        match self {
            Self::XS => 24,
            Self::SM => 32,
            Self::MD => 48,
            Self::LG => 64,
            Self::XL => 96,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum AvatarType {
    Personal,
    Organisation,
    Project,
    Group,
    Channel,
}

#[derive(Clone, PartialEq)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    pub four_words: String,
    #[props(default = AvatarSize::MD)]
    pub size: AvatarSize,
    #[props(default)]
    pub presence: Option<PresenceStatus>,
    #[props(default = AvatarType::Personal)]
    pub avatar_type: AvatarType,
    #[props(default = false)]
    pub show_tooltip: bool,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn FourWordAvatar(props: AvatarProps) -> Element {
    let dim = props.size.dimension();
    let initials = generate_initials(&props.four_words);
    let gradient = generate_gradient(&props.four_words);
    let border = get_type_border(&props.avatar_type);
    
    let presence_color = props.presence.as_ref().map(|p| match p {
        PresenceStatus::Online => "bg-green-500",
        PresenceStatus::Away => "bg-yellow-500",
        PresenceStatus::Busy => "bg-red-500",
        PresenceStatus::Offline => "bg-gray-400",
    });
    
    rsx! {
        div {
            class: "relative inline-block",
            
            // Avatar circle
            div {
                class: "rounded-full flex items-center justify-center font-semibold text-white shadow-lg transition-transform hover:scale-105",
                style: "width: {dim}px; height: {dim}px; background: {gradient}; border: {border}; font-size: {dim / 3}px;",
                onclick: move |e| if let Some(handler) = &props.onclick { handler.call(e) },
                
                "{initials}"
            }
            
            // Presence indicator
            if let Some(color) = presence_color {
                div {
                    class: "absolute bottom-0 right-0 rounded-full border-2 border-white {color}",
                    style: "width: {dim / 4}px; height: {dim / 4}px;",
                }
            }
        }
        
        // Tooltip (conditional)
        if props.show_tooltip {
            // Tooltip implementation
        }
    }
}

fn generate_initials(four_words: &str) -> String {
    four_words
        .split('-')
        .filter_map(|w| w.chars().next())
        .map(|c| c.to_uppercase().next().unwrap_or(c))
        .take(4)
        .collect()
}

fn generate_gradient(four_words: &str) -> String {
    // Generate deterministic gradient from four words
    let hash = simple_hash(four_words);
    let hue1 = hash % 360;
    let hue2 = (hash / 360) % 360;
    format!(
        "linear-gradient(135deg, hsl({}, 70%, 50%), hsl({}, 70%, 40%))",
        hue1, hue2
    )
}

fn get_type_border(avatar_type: &AvatarType) -> &'static str {
    match avatar_type {
        AvatarType::Organisation => "2px solid #FFD700",
        AvatarType::Project => "2px solid #C0C0C0",
        _ => "none",
    }
}

fn simple_hash(s: &str) -> u64 {
    s.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
}
```

#### Button Component

```rust
// src/components/atoms/button.rs

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
    Success,
}

#[derive(Clone, PartialEq)]
pub enum ButtonSize {
    SM,
    MD,
    LG,
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    pub children: Element,
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[props(default = ButtonSize::MD)]
    pub size: ButtonSize,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub loading: bool,
    #[props(default = false)]
    pub full_width: bool,
    #[props(default)]
    pub icon: Option<Element>,
    pub onclick: EventHandler<MouseEvent>,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let base_class = "inline-flex items-center justify-center font-medium rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2";
    
    let variant_class = match props.variant {
        ButtonVariant::Primary => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
        ButtonVariant::Secondary => "bg-gray-100 text-gray-900 hover:bg-gray-200 focus:ring-gray-500 dark:bg-gray-700 dark:text-white",
        ButtonVariant::Ghost => "text-gray-700 hover:bg-gray-100 focus:ring-gray-500 dark:text-gray-300 dark:hover:bg-gray-800",
        ButtonVariant::Danger => "bg-red-600 text-white hover:bg-red-700 focus:ring-red-500",
        ButtonVariant::Success => "bg-green-600 text-white hover:bg-green-700 focus:ring-green-500",
    };
    
    let size_class = match props.size {
        ButtonSize::SM => "px-3 py-1.5 text-sm gap-1.5",
        ButtonSize::MD => "px-4 py-2 text-base gap-2",
        ButtonSize::LG => "px-6 py-3 text-lg gap-2.5",
    };
    
    let width_class = if props.full_width { "w-full" } else { "" };
    let disabled_class = if props.disabled || props.loading { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" };
    
    rsx! {
        button {
            class: "{base_class} {variant_class} {size_class} {width_class} {disabled_class}",
            disabled: props.disabled || props.loading,
            onclick: move |e| if !props.disabled && !props.loading { props.onclick.call(e) },
            
            if props.loading {
                Spinner { size: SpinnerSize::SM }
            } else if let Some(icon) = &props.icon {
                {icon}
            }
            
            {props.children}
        }
    }
}
```

### 3.2 Molecules

#### FourWordInput Component

```rust
// src/components/molecules/four_word_input.rs

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct FourWordInputProps {
    pub value: Signal<String>,
    #[props(default = "Enter four words...".to_string())]
    pub placeholder: String,
    #[props(default = false)]
    pub show_generator: bool,
    #[props(default)]
    pub on_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_validate: Option<EventHandler<bool>>,
}

#[component]
pub fn FourWordInput(props: FourWordInputProps) -> Element {
    let mut words = use_signal(|| vec![String::new(); 4]);
    let mut is_valid = use_signal(|| false);
    let mut suggestions = use_signal(|| Vec::<String>::new());
    
    // Parse initial value into words
    use_effect(move || {
        let parts: Vec<String> = props.value.read()
            .split('-')
            .map(|s| s.to_string())
            .collect();
        if parts.len() == 4 {
            words.set(parts);
        }
    });
    
    let validate_words = move || {
        let all_filled = words.read().iter().all(|w| !w.is_empty());
        // In production, validate against four-word-networking dictionary
        is_valid.set(all_filled);
        if let Some(handler) = &props.on_validate {
            handler.call(all_filled);
        }
    };
    
    let generate_random = move |_| {
        // Call communitas-bindings to generate random words
        // For now, placeholder
        words.set(vec![
            "ocean".to_string(),
            "forest".to_string(), 
            "moon".to_string(),
            "star".to_string(),
        ]);
        validate_words();
    };
    
    rsx! {
        div {
            class: "space-y-3",
            
            // Four word inputs in a row
            div {
                class: "flex gap-2",
                
                for (idx, _word) in words.read().iter().enumerate() {
                    input {
                        class: "flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-800 dark:border-gray-600",
                        r#type: "text",
                        placeholder: match idx {
                            0 => "ocean",
                            1 => "forest",
                            2 => "moon",
                            _ => "star",
                        },
                        value: "{words.read()[idx]}",
                        oninput: move |e| {
                            let mut w = words.write();
                            w[idx] = e.value().to_lowercase();
                            drop(w);
                            
                            // Update combined value
                            let combined = words.read().join("-");
                            props.value.set(combined.clone());
                            if let Some(handler) = &props.on_change {
                                handler.call(combined);
                            }
                            validate_words();
                        },
                        onblur: move |_| validate_words(),
                    }
                }
            }
            
            // Generator button and validation
            div {
                class: "flex items-center justify-between",
                
                if props.show_generator {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::SM,
                        onclick: generate_random,
                        icon: rsx! { Icon { name: "sparkles" } },
                        "Generate Random"
                    }
                }
                
                // Validation indicator
                if *is_valid.read() {
                    div {
                        class: "flex items-center gap-1 text-green-600 text-sm",
                        Icon { name: "check-circle", size: IconSize::SM }
                        "Valid identity"
                    }
                }
            }
            
            // Autocomplete suggestions (when typing)
            if !suggestions.read().is_empty() {
                div {
                    class: "absolute z-10 w-full bg-white dark:bg-gray-800 border rounded-lg shadow-lg max-h-48 overflow-y-auto",
                    for suggestion in suggestions.read().iter() {
                        button {
                            class: "w-full px-3 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700",
                            "{suggestion}"
                        }
                    }
                }
            }
        }
    }
}
```

#### MessageBubble Component

```rust
// src/components/molecules/message_bubble.rs

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MessageBubbleProps {
    pub message: SwiftMessage,
    pub is_own: bool,
    #[props(default = false)]
    pub show_avatar: bool,
    #[props(default = false)]
    pub show_reactions: bool,
    #[props(default)]
    pub on_reply: Option<EventHandler<String>>,
    #[props(default)]
    pub on_react: Option<EventHandler<String>>,
}

#[component]
pub fn MessageBubble(props: MessageBubbleProps) -> Element {
    let alignment = if props.is_own { "justify-end" } else { "justify-start" };
    let bubble_color = if props.is_own {
        "bg-blue-600 text-white"
    } else {
        "bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white"
    };
    let bubble_radius = if props.is_own {
        "rounded-2xl rounded-br-md"
    } else {
        "rounded-2xl rounded-bl-md"
    };
    
    rsx! {
        div {
            class: "flex {alignment} gap-2 group",
            
            // Avatar (for received messages)
            if !props.is_own && props.show_avatar {
                FourWordAvatar {
                    four_words: props.message.author.clone(),
                    size: AvatarSize::SM,
                }
            }
            
            div {
                class: "max-w-xs md:max-w-md lg:max-w-lg",
                
                // Author name (for group messages)
                if !props.is_own {
                    p {
                        class: "text-xs text-gray-500 mb-1 ml-1",
                        "{short_four_words(&props.message.author)}"
                    }
                }
                
                // Message bubble
                div {
                    class: "relative px-4 py-2 {bubble_color} {bubble_radius} shadow-sm",
                    
                    // Message text
                    p {
                        class: "whitespace-pre-wrap break-words",
                        "{props.message.text}"
                    }
                    
                    // Timestamp
                    p {
                        class: "text-xs opacity-70 mt-1 text-right",
                        "{format_timestamp(props.message.created_at)}"
                    }
                }
                
                // Reactions (placeholder)
            }
            
            // Quick actions (on hover)
            div {
                class: "hidden group-hover:flex items-center gap-1",
                
                IconButton {
                    icon: "reply",
                    size: IconSize::SM,
                    onclick: move |_| if let Some(h) = &props.on_reply { h.call(props.message.id.clone()) },
                }
                IconButton {
                    icon: "emoji-happy",
                    size: IconSize::SM,
                    onclick: move |_| if let Some(h) = &props.on_react { h.call(props.message.id.clone()) },
                }
            }
        }
    }
}

fn short_four_words(four_words: &str) -> String {
    let words: Vec<&str> = four_words.split('-').collect();
    if words.len() >= 2 {
        format!("{} {}", words[0], words[1])
    } else {
        four_words.replace('-', " ")
    }
}

fn format_timestamp(ts: i64) -> String {
    // Format timestamp as "HH:MM" or "Yesterday" etc.
    "12:34".to_string()
}
```

---

## 4. Theming System

### 4.1 Color Palette

```rust
// src/theme/colors.rs

pub struct ColorPalette {
    // Primary brand colors
    pub primary_50: &'static str,
    pub primary_100: &'static str,
    pub primary_500: &'static str,
    pub primary_600: &'static str,
    pub primary_700: &'static str,
    
    // Neutral colors
    pub gray_50: &'static str,
    pub gray_100: &'static str,
    pub gray_200: &'static str,
    pub gray_300: &'static str,
    pub gray_400: &'static str,
    pub gray_500: &'static str,
    pub gray_600: &'static str,
    pub gray_700: &'static str,
    pub gray_800: &'static str,
    pub gray_900: &'static str,
    
    // Semantic colors
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,
    
    // Background colors
    pub bg_primary: &'static str,
    pub bg_secondary: &'static str,
    pub bg_tertiary: &'static str,
    
    // Text colors
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    
    // Border colors
    pub border_light: &'static str,
    pub border_default: &'static str,
}

pub const LIGHT_PALETTE: ColorPalette = ColorPalette {
    primary_50: "#EFF6FF",
    primary_100: "#DBEAFE",
    primary_500: "#3B82F6",
    primary_600: "#2563EB",
    primary_700: "#1D4ED8",
    
    gray_50: "#F9FAFB",
    gray_100: "#F3F4F6",
    gray_200: "#E5E7EB",
    gray_300: "#D1D5DB",
    gray_400: "#9CA3AF",
    gray_500: "#6B7280",
    gray_600: "#4B5563",
    gray_700: "#374151",
    gray_800: "#1F2937",
    gray_900: "#111827",
    
    success: "#22C55E",
    warning: "#F59E0B",
    error: "#EF4444",
    info: "#3B82F6",
    
    bg_primary: "#FFFFFF",
    bg_secondary: "#F9FAFB",
    bg_tertiary: "#F3F4F6",
    
    text_primary: "#111827",
    text_secondary: "#4B5563",
    text_muted: "#9CA3AF",
    
    border_light: "#E5E7EB",
    border_default: "#D1D5DB",
};

pub const DARK_PALETTE: ColorPalette = ColorPalette {
    primary_50: "#1E3A5F",
    primary_100: "#1E40AF",
    primary_500: "#3B82F6",
    primary_600: "#60A5FA",
    primary_700: "#93C5FD",
    
    gray_50: "#030712",
    gray_100: "#111827",
    gray_200: "#1F2937",
    gray_300: "#374151",
    gray_400: "#4B5563",
    gray_500: "#6B7280",
    gray_600: "#9CA3AF",
    gray_700: "#D1D5DB",
    gray_800: "#E5E7EB",
    gray_900: "#F9FAFB",
    
    success: "#22C55E",
    warning: "#F59E0B",
    error: "#EF4444",
    info: "#60A5FA",
    
    bg_primary: "#111827",
    bg_secondary: "#1F2937",
    bg_tertiary: "#374151",
    
    text_primary: "#F9FAFB",
    text_secondary: "#D1D5DB",
    text_muted: "#6B7280",
    
    border_light: "#374151",
    border_default: "#4B5563",
};
```

### 4.2 Theme Provider

```rust
// src/theme/mod.rs

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: &'static ColorPalette,
}

impl Theme {
    pub fn new(mode: ThemeMode) -> Self {
        let colors = match mode {
            ThemeMode::Light => &LIGHT_PALETTE,
            ThemeMode::Dark => &DARK_PALETTE,
            ThemeMode::System => {
                // Detect system preference
                if is_dark_mode_preferred() {
                    &DARK_PALETTE
                } else {
                    &LIGHT_PALETTE
                }
            }
        };
        Self { mode, colors }
    }
    
    pub fn class(&self) -> &'static str {
        match self.mode {
            ThemeMode::Dark => "dark",
            _ => "",
        }
    }
}

#[component]
pub fn ThemeProvider(children: Element) -> Element {
    let mut theme = use_signal(|| Theme::new(ThemeMode::System));
    
    // Provide theme context
    use_context_provider(|| theme);
    
    rsx! {
        div {
            class: "{theme.read().class()}",
            data_theme: match theme.read().mode {
                ThemeMode::Light => "light",
                ThemeMode::Dark => "dark",
                ThemeMode::System => "system",
            },
            
            {children}
        }
    }
}

// Hook for components to access theme
pub fn use_theme() -> Signal<Theme> {
    use_context::<Signal<Theme>>()
}
```

---

## 5. Navigation Patterns

### 5.1 Router Configuration

```rust
// src/router.rs

use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    // Auth routes
    #[route("/")]
    Welcome,
    #[route("/login")]
    Login,
    #[route("/create-identity")]
    CreateIdentity,
    #[route("/vault-selection")]
    VaultSelection,
    
    // Main routes (require auth)
    #[route("/app")]
    #[layout(MainLayout)]
    Home,
    
    #[route("/app/entity/:id")]
    EntityDetail { id: String },
    
    #[route("/app/contact/:four_words")]
    ContactChat { four_words: String },
    
    #[route("/app/call/:peer_four_words")]
    Call { peer_four_words: String },
    
    #[route("/app/settings")]
    Settings,
    
    #[route("/app/settings/network")]
    NetworkSettings,
}

#[component]
pub fn AppRouter() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
```

### 5.2 Navigation State

```rust
// src/hooks/use_navigation.rs

#[derive(Clone, PartialEq)]
pub enum ActiveView {
    Home,
    Chat { entity_type: String, entity_id: String, entity_name: String },
    ContactChat { four_words: String, display_name: Option<String> },
    Drive { entity_type: String, entity_id: String },
    Call { peer_four_words: String },
    Project { project_id: String },
    Settings,
}

pub fn use_navigation() -> UseNavigation {
    let nav = use_navigator();
    let active_view = use_signal(|| ActiveView::Home);
    
    UseNavigation {
        navigator: nav,
        active_view,
    }
}

pub struct UseNavigation {
    navigator: Navigator,
    pub active_view: Signal<ActiveView>,
}

impl UseNavigation {
    pub fn go_home(&self) {
        self.active_view.set(ActiveView::Home);
        self.navigator.push(Route::Home);
    }
    
    pub fn open_entity(&self, entity: &SwiftEntity) {
        self.active_view.set(ActiveView::Chat {
            entity_type: entity.entity_type.to_string(),
            entity_id: entity.id.clone(),
            entity_name: entity.name.clone(),
        });
        self.navigator.push(Route::EntityDetail { id: entity.id.clone() });
    }
    
    pub fn open_contact_chat(&self, contact: &ContactItem) {
        self.active_view.set(ActiveView::ContactChat {
            four_words: contact.four_words.clone(),
            display_name: contact.display_name.clone(),
        });
        self.navigator.push(Route::ContactChat { 
            four_words: contact.four_words.clone() 
        });
    }
    
    pub fn start_call(&self, peer_four_words: &str) {
        self.active_view.set(ActiveView::Call {
            peer_four_words: peer_four_words.to_string(),
        });
        self.navigator.push(Route::Call { 
            peer_four_words: peer_four_words.to_string() 
        });
    }
}
```

---

## 6. Platform-Specific Adaptations

### 6.1 Platform Detection

```rust
// src/platform/mod.rs

#[derive(Clone, Copy, PartialEq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
    IOS,
    Android,
    Web,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        
        #[cfg(target_os = "ios")]
        return Platform::IOS;
        
        #[cfg(target_os = "android")]
        return Platform::Android;
        
        #[cfg(target_arch = "wasm32")]
        return Platform::Web;
    }
    
    pub fn is_mobile(&self) -> bool {
        matches!(self, Platform::IOS | Platform::Android)
    }
    
    pub fn is_desktop(&self) -> bool {
        matches!(self, Platform::MacOS | Platform::Windows | Platform::Linux)
    }
    
    pub fn supports_window_controls(&self) -> bool {
        matches!(self, Platform::MacOS)
    }
}

pub fn use_platform() -> Platform {
    Platform::current()
}
```

### 6.2 Desktop Window Chrome (macOS)

```rust
// src/platform/desktop.rs

#[component]
pub fn DesktopWindowChrome(children: Element) -> Element {
    let platform = use_platform();
    
    rsx! {
        div {
            class: "h-screen flex flex-col",
            
            // macOS traffic light padding
            if platform == Platform::MacOS {
                div {
                    class: "h-7 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700",
                    // Titlebar area - traffic lights go here
                    style: "-webkit-app-region: drag;",
                }
            }
            
            // Main content
            div {
                class: "flex-1 overflow-hidden",
                {children}
            }
        }
    }
}
```

### 6.3 Mobile Tab Bar

```rust
// src/platform/mobile.rs

#[component]
pub fn MobileTabBar() -> Element {
    let nav = use_navigation();
    
    let tabs = vec![
        ("Chats", "chat-bubble-left-right", ActiveView::Home),
        ("Contacts", "users", ActiveView::Home),
        ("Files", "folder", ActiveView::Home),
        ("Settings", "cog-6-tooth", ActiveView::Settings),
    ];
    
    rsx! {
        nav {
            class: "fixed bottom-0 left-0 right-0 bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-700 safe-area-inset-bottom",
            
            div {
                class: "flex justify-around items-center h-16",
                
                for (label, icon, view) in tabs {
                    button {
                        class: "flex flex-col items-center gap-1 px-4 py-2",
                        onclick: move |_| nav.active_view.set(view.clone()),
                        
                        Icon {
                            name: icon,
                            size: IconSize::MD,
                            class: if *nav.active_view.read() == view {
                                "text-blue-600"
                            } else {
                                "text-gray-500"
                            },
                        }
                        span {
                            class: if *nav.active_view.read() == view {
                                "text-xs font-medium text-blue-600"
                            } else {
                                "text-xs text-gray-500"
                            },
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
```

### 6.4 Responsive Layout

```rust
// src/layouts/main_layout.rs

#[component]
pub fn MainLayout() -> Element {
    let platform = use_platform();
    let state = use_context::<AppState>();
    
    if platform.is_mobile() {
        // Mobile: Full-screen with tab navigation
        rsx! {
            div {
                class: "h-screen flex flex-col",
                
                // Content area (scrollable)
                div {
                    class: "flex-1 overflow-hidden pb-16", // padding for tab bar
                    Outlet::<Route> {}
                }
                
                // Tab bar
                MobileTabBar {}
            }
        }
    } else {
        // Desktop: Sidebar + Detail pane
        rsx! {
            DesktopWindowChrome {
                div {
                    class: "h-full flex",
                    
                    // Sidebar (collapsible on smaller screens)
                    aside {
                        class: "w-64 xl:w-80 flex-shrink-0 hidden md:flex flex-col",
                        
                        ProfileHeader {}
                        
                        Sidebar {
                            selected_entity: state.selected_entity,
                            on_select_entity: move |e| state.select_entity(e),
                            on_select_contact: move |c| state.select_contact(c),
                            on_create_entity: move |ctx| state.show_create_modal(ctx),
                        }
                    }
                    
                    // Detail pane
                    main {
                        class: "flex-1 overflow-hidden",
                        Outlet::<Route> {}
                    }
                }
            }
        }
    }
}
```

---

## 7. State Management

### 7.1 Global App State

```rust
// src/state.rs

use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    // Core
    pub client: Signal<Option<CommunitasClient>>,
    
    // Authentication
    pub four_words: Signal<String>,
    pub display_name: Signal<String>,
    pub is_authenticated: Signal<bool>,
    
    // Network
    pub is_networking: Signal<bool>,
    pub connection_identity: Signal<Option<String>>,
    
    // Data
    pub entities: Signal<Vec<SwiftEntity>>,
    pub contacts: Signal<Vec<ContactItem>>,
    pub messages: Signal<HashMap<String, Vec<SwiftMessage>>>,
    
    // UI State
    pub selected_entity: Signal<Option<SwiftEntity>>,
    pub active_modal: Signal<Option<ModalType>>,
    pub error_message: Signal<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: Signal::new(None),
            four_words: Signal::new(String::new()),
            display_name: Signal::new(String::new()),
            is_authenticated: Signal::new(false),
            is_networking: Signal::new(false),
            connection_identity: Signal::new(None),
            entities: Signal::new(Vec::new()),
            contacts: Signal::new(Vec::new()),
            messages: Signal::new(HashMap::new()),
            selected_entity: Signal::new(None),
            active_modal: Signal::new(None),
            error_message: Signal::new(None),
        }
    }
    
    // Authentication methods
    pub fn login(&self, four_words: &str, password: &str) -> Result<(), ClientError> {
        // Implementation using communitas-bindings
        todo!()
    }
    
    pub fn logout(&self) {
        self.is_authenticated.set(false);
        self.client.set(None);
    }
    
    // Entity methods
    pub fn load_entities(&self) {
        if let Some(client) = self.client.read().as_ref() {
            match client.entity_list() {
                Ok(entities) => self.entities.set(entities),
                Err(e) => self.error_message.set(Some(e.to_string())),
            }
        }
    }
    
    pub fn create_entity(&self, name: &str, entity_type: SwiftEntityType, description: Option<&str>, parent_org_id: Option<&str>) {
        if let Some(client) = self.client.read().as_ref() {
            match client.entity_create(name, entity_type, description, parent_org_id) {
                Ok(_) => self.load_entities(),
                Err(e) => self.error_message.set(Some(e.to_string())),
            }
        }
    }
    
    // Messaging methods
    pub fn load_messages(&self, entity_id: &str) {
        if let Some(client) = self.client.read().as_ref() {
            match client.message_get_for_entity(entity_id, Some(100), None) {
                Ok(msgs) => {
                    let mut map = self.messages.write();
                    map.insert(entity_id.to_string(), msgs);
                }
                Err(e) => self.error_message.set(Some(e.to_string())),
            }
        }
    }
    
    pub fn send_message(&self, entity_id: &str, text: &str, reply_to_id: Option<&str>) {
        if let Some(client) = self.client.read().as_ref() {
            match client.message_send(entity_id, text, reply_to_id) {
                Ok(_) => self.load_messages(entity_id),
                Err(e) => self.error_message.set(Some(e.to_string())),
            }
        }
    }
    
    // Network methods
    pub fn start_networking(&self, port: Option<u16>) {
        if let Some(client) = self.client.read().as_ref() {
            match client.gossip_start(port) {
                Ok(identity) => {
                    self.is_networking.set(true);
                    self.connection_identity.set(Some(identity));
                }
                Err(e) => self.error_message.set(Some(e.to_string())),
            }
        }
    }
    
    pub fn stop_networking(&self) {
        if let Some(client) = self.client.read().as_ref() {
            let _ = client.gossip_stop();
            self.is_networking.set(false);
            self.connection_identity.set(None);
        }
    }
}

// Provider component
#[component]
pub fn StateProvider(children: Element) -> Element {
    let state = AppState::new();
    use_context_provider(|| state.clone());
    
    rsx! {
        {children}
    }
}

// Hook to access state
pub fn use_app_state() -> AppState {
    use_context::<AppState>()
}
```

---

## 8. Implementation Priority

### Phase 1: Foundation (Week 1-2)

1. **Project Setup**
   - Create `communitas-dioxus` crate
   - Configure Dioxus.toml for all platforms
   - Set up Tailwind CSS
   - Integrate communitas-bindings

2. **Core Components**
   - ThemeProvider
   - Button, Input, Icon atoms
   - FourWordAvatar
   - FourWordInput

3. **Basic Layouts**
   - AuthLayout
   - MainLayout (desktop only)

### Phase 2: Authentication (Week 3-4)

1. **Auth Screens**
   - WelcomeScreen
   - LoginScreen
   - CreateIdentityScreen
   - VaultSelectionScreen

2. **Auth State**
   - use_auth hook
   - Session management
   - Passkey integration

### Phase 3: Core Features (Week 5-6)

1. **Sidebar & Navigation**
   - Full Sidebar with sections
   - OrganisationNode
   - ContactRow
   - EntityRow

2. **Chat Features**
   - MessageBubble
   - MessageComposer
   - ChatPanel
   - ContactChatView

3. **Entity Management**
   - CreateEntityModal
   - EntityDetailPane
   - Member management

### Phase 4: Advanced Features (Week 7-8)

1. **Virtual Disks**
   - DrivePanel
   - FileRow
   - Upload/download

2. **Documents**
   - DocumentEditor
   - CRDT sync

3. **Calls (WebRTC)**
   - CallView
   - IncomingCallModal
   - MediaControls

### Phase 5: Mobile & Polish (Week 9-10)

1. **Mobile Layout**
   - MobileTabBar
   - Touch gestures
   - Platform-specific UI

2. **Polish**
   - Animations
   - Loading states
   - Error handling
   - Accessibility

---

## 9. Critical Files Reference

### Critical Files for Implementation

| File | Purpose | Priority |
|------|---------|----------|
| `communitas-bindings/src/lib.rs` | UniFFI bindings with all Swift types and 98+ API methods | Critical |
| `communitas-swift/APP_SPECIFICATION.md` | Complete app specification with screens and flows | Critical |
| `communitas-swift/CommunitasApp/Sources/AppState.swift` | SwiftUI state management patterns | High |
| `communitas-swift/CommunitasApp/Sources/ContentView.swift` | SwiftUI component structure | High |
| `communitas-swift/CommunitasApp/Sources/SidebarView.swift` | Sidebar organization patterns | High |
| `src/components/unified/FourWordAvatar.tsx` | React avatar with gradient generation | Medium |
| `communitas-tui/src/components/avatar.rs` | Rust TUI avatar implementation | Medium |

### Key Data Types from communitas-bindings

- `SwiftUserProfile` - User identity and device info
- `SwiftEntity` - Organisation/Project/Group/Channel
- `SwiftMessage` - Chat message with reactions
- `SwiftContactInfo` - Contact with presence and endpoint tracking
- `SwiftPresenceInfo` - Online status
- `SwiftDocumentInfo` - CRDT document metadata
- `SwiftFileInfo` - Virtual disk file
- `SwiftCallState` - WebRTC call state
- `ClientError` - Error enum for all operations

### API Method Categories

1. **Auth (15 methods)**: Vault management, login, passkeys
2. **Entity (10 methods)**: Create/manage entities and members
3. **Messaging (7 methods)**: Send/receive messages
4. **Document (9 methods)**: CRDT document operations
5. **Gossip/Network (18 methods)**: P2P networking, contacts
6. **Presence (5 methods)**: Online status beacons
7. **Disk (9 methods)**: Virtual disk file operations
8. **WebRTC (11 methods)**: Voice/video calls

---

## Key Design Decisions

1. **Atomic Design Pattern**: Components organized as atoms -> molecules -> organisms for maximum reuse

2. **Signal-based Reactivity**: Using Dioxus Signals for fine-grained reactivity

3. **Platform-First Design**: Desktop and mobile share component library but have distinct layouts

4. **Offline-First Architecture**: All state changes are local-first with P2P sync

5. **Four-Word Identity Throughout**: All user-facing identifiers use four-word format with spaces for readability

6. **Theme System**: CSS variables + Tailwind for light/dark modes with system preference detection

7. **Accessibility**: ARIA labels, keyboard navigation, focus management built into all interactive components
