// Resolve MSP Platform - Theme System
// Tokyo Night (Night, Storm, Moon) + Dracula themes

use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

const THEME_STORAGE_KEY: &str = "resolve_theme";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    TokyoNight,
    TokyoStorm,
    TokyoMoon,
    Dracula,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::TokyoNight => "tokyo-night",
            Theme::TokyoStorm => "tokyo-storm",
            Theme::TokyoMoon => "tokyo-moon",
            Theme::Dracula => "dracula",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::TokyoNight => "Tokyo Night",
            Theme::TokyoStorm => "Tokyo Night Storm",
            Theme::TokyoMoon => "Tokyo Night Moon",
            Theme::Dracula => "Dracula",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Theme::TokyoNight => "The classic dark theme with deep blues",
            Theme::TokyoStorm => "A slightly lighter variant with storm blue tones",
            Theme::TokyoMoon => "Softer, moonlit purple-blue palette",
            Theme::Dracula => "Classic purple and pink vampire aesthetics",
        }
    }

    pub fn all() -> [Theme; 4] {
        [
            Theme::TokyoNight,
            Theme::TokyoStorm,
            Theme::TokyoMoon,
            Theme::Dracula,
        ]
    }

    pub fn from_str(s: &str) -> Option<Theme> {
        match s {
            "tokyo-night" => Some(Theme::TokyoNight),
            "tokyo-storm" => Some(Theme::TokyoStorm),
            "tokyo-moon" => Some(Theme::TokyoMoon),
            "dracula" => Some(Theme::Dracula),
            _ => None,
        }
    }

    /// Primary accent color for this theme (for preview swatches)
    pub fn accent_color(&self) -> &'static str {
        match self {
            Theme::TokyoNight => "#7aa2f7",
            Theme::TokyoStorm => "#7aa2f7",
            Theme::TokyoMoon => "#82aaff",
            Theme::Dracula => "#bd93f9",
        }
    }

    /// Background color for this theme (for preview)
    pub fn bg_color(&self) -> &'static str {
        match self {
            Theme::TokyoNight => "#1a1b26",
            Theme::TokyoStorm => "#24283b",
            Theme::TokyoMoon => "#222436",
            Theme::Dracula => "#282a36",
        }
    }
}

/// Apply theme to the document
pub fn apply_theme(theme: Theme) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(root) = document.document_element() {
                if let Ok(html) = root.dyn_into::<HtmlElement>() {
                    html.set_attribute("data-theme", theme.as_str()).ok();
                }
            }
        }
    }
}

/// Load theme from local storage or return default
pub fn load_theme() -> Theme {
    LocalStorage::get::<String>(THEME_STORAGE_KEY)
        .ok()
        .and_then(|s| Theme::from_str(&s))
        .unwrap_or_default()
}

/// Save theme to local storage
pub fn save_theme(theme: Theme) {
    let _ = LocalStorage::set(THEME_STORAGE_KEY, theme.as_str());
}

// ===== Theme Context =====

#[derive(Clone, PartialEq)]
pub struct ThemeContext {
    pub theme: Theme,
    pub set_theme: Callback<Theme>,
}

#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    pub children: Html,
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProviderProps) -> Html {
    let theme = use_state(|| load_theme());

    // Apply theme on mount and when it changes
    {
        let theme = theme.clone();
        use_effect_with((*theme).clone(), move |theme| {
            apply_theme(*theme);
            || ()
        });
    }

    let set_theme = {
        let theme = theme.clone();
        Callback::from(move |new_theme: Theme| {
            save_theme(new_theme);
            theme.set(new_theme);
        })
    };

    let ctx = ThemeContext {
        theme: *theme,
        set_theme,
    };

    html! {
        <ContextProvider<ThemeContext> context={ctx}>
            { props.children.clone() }
        </ContextProvider<ThemeContext>>
    }
}

/// Hook to access theme context
#[hook]
pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>().expect("ThemeContext not found")
}

// ===== Theme Selector Component =====

#[derive(Properties, PartialEq)]
pub struct ThemeSelectorProps {
    #[prop_or_default]
    pub compact: bool,
}

#[function_component(ThemeSelector)]
pub fn theme_selector(props: &ThemeSelectorProps) -> Html {
    let theme_ctx = use_theme();
    let show_dropdown = use_state(|| false);

    let toggle_dropdown = {
        let show_dropdown = show_dropdown.clone();
        Callback::from(move |_| show_dropdown.set(!*show_dropdown))
    };

    let close_dropdown = {
        let show_dropdown = show_dropdown.clone();
        Callback::from(move |_| show_dropdown.set(false))
    };

    if props.compact {
        html! {
            <div class="relative">
                <button
                    onclick={toggle_dropdown}
                    class="flex items-center space-x-2 px-3 py-2 rounded-lg bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm"
                >
                    <div
                        class="w-4 h-4 rounded-full"
                        style={format!("background-color: {}", theme_ctx.theme.accent_color())}
                    />
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                    </svg>
                </button>

                if *show_dropdown {
                    <div class="absolute right-0 mt-2 w-56 bg-gray-800 rounded-lg shadow-lg border border-gray-700 py-2 z-50">
                        { for Theme::all().iter().map(|t| {
                            let theme = *t;
                            let theme_ctx = theme_ctx.clone();
                            let close = close_dropdown.clone();
                            let is_selected = theme_ctx.theme == theme;

                            html! {
                                <button
                                    onclick={Callback::from(move |_| {
                                        theme_ctx.set_theme.emit(theme);
                                        close.emit(());
                                    })}
                                    class={format!(
                                        "w-full flex items-center px-4 py-2 text-left hover:bg-gray-700 {}",
                                        if is_selected { "bg-gray-700" } else { "" }
                                    )}
                                >
                                    <div
                                        class="w-4 h-4 rounded-full mr-3"
                                        style={format!("background-color: {}", theme.accent_color())}
                                    />
                                    <span class="text-gray-200 text-sm">{theme.display_name()}</span>
                                    if is_selected {
                                        <svg class="w-4 h-4 ml-auto text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                                        </svg>
                                    }
                                </button>
                            }
                        })}
                    </div>
                }
            </div>
        }
    } else {
        html! {
            <div class="space-y-3">
                <h3 class="text-lg font-medium text-white">{"Theme"}</h3>
                <div class="grid grid-cols-2 gap-3">
                    { for Theme::all().iter().map(|t| {
                        let theme = *t;
                        let theme_ctx = theme_ctx.clone();
                        let is_selected = theme_ctx.theme == theme;

                        html! {
                            <button
                                onclick={Callback::from(move |_| {
                                    theme_ctx.set_theme.emit(theme);
                                })}
                                class={format!(
                                    "relative p-4 rounded-lg border-2 transition-all {}",
                                    if is_selected {
                                        "border-blue-500 ring-2 ring-blue-500/20"
                                    } else {
                                        "border-gray-700 hover:border-gray-600"
                                    }
                                )}
                                style={format!("background-color: {}", theme.bg_color())}
                            >
                                <div class="flex items-center space-x-3">
                                    <div
                                        class="w-6 h-6 rounded-full"
                                        style={format!("background-color: {}", theme.accent_color())}
                                    />
                                    <div class="text-left">
                                        <div class="text-sm font-medium text-gray-200">{theme.display_name()}</div>
                                        <div class="text-xs text-gray-500">{theme.description()}</div>
                                    </div>
                                </div>
                                if is_selected {
                                    <div class="absolute top-2 right-2">
                                        <svg class="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                                        </svg>
                                    </div>
                                }
                            </button>
                        }
                    })}
                </div>
            </div>
        }
    }
}

// ===== Color Palette Display (for settings page) =====

#[function_component(ColorPalette)]
pub fn color_palette() -> Html {
    let theme_ctx = use_theme();

    let colors = match theme_ctx.theme {
        Theme::TokyoNight | Theme::TokyoStorm => vec![
            ("Blue", "#7aa2f7"),
            ("Cyan", "#7dcfff"),
            ("Teal", "#73daca"),
            ("Green", "#9ece6a"),
            ("Yellow", "#e0af68"),
            ("Orange", "#ff9e64"),
            ("Red", "#f7768e"),
            ("Purple", "#bb9af7"),
        ],
        Theme::TokyoMoon => vec![
            ("Blue", "#82aaff"),
            ("Cyan", "#86e1fc"),
            ("Teal", "#4fd6be"),
            ("Green", "#c3e88d"),
            ("Yellow", "#ffc777"),
            ("Orange", "#ff966c"),
            ("Red", "#ff757f"),
            ("Purple", "#c099ff"),
        ],
        Theme::Dracula => vec![
            ("Purple", "#bd93f9"),
            ("Cyan", "#8be9fd"),
            ("Green", "#50fa7b"),
            ("Yellow", "#f1fa8c"),
            ("Orange", "#ffb86c"),
            ("Red", "#ff5555"),
            ("Pink", "#ff79c6"),
            ("Comment", "#6272a4"),
        ],
    };

    html! {
        <div class="space-y-3">
            <h4 class="text-sm font-medium text-gray-400">{"Color Palette"}</h4>
            <div class="flex flex-wrap gap-2">
                { for colors.iter().map(|(name, color)| {
                    html! {
                        <div class="flex items-center space-x-2 px-3 py-1.5 rounded bg-gray-800">
                            <div
                                class="w-4 h-4 rounded"
                                style={format!("background-color: {}", color)}
                            />
                            <span class="text-xs text-gray-400">{name}</span>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
