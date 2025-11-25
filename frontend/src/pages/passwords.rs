// Passwords Page - Hudu-style password management
// Features: Phonetic spelling, show/hide, OTP, notes, secure sharing

use gloo_timers::callback::Interval;
use yew::prelude::*;
use crate::services::{self, ApiResult, PaginatedResponse};
use serde::{Deserialize, Serialize};

// NATO phonetic alphabet mapping
fn get_phonetic(c: char) -> &'static str {
    match c.to_ascii_lowercase() {
        'a' => "ALFA",
        'b' => "BRAVO",
        'c' => "CHARLIE",
        'd' => "DELTA",
        'e' => "ECHO",
        'f' => "FOXTROT",
        'g' => "GOLF",
        'h' => "HOTEL",
        'i' => "INDIA",
        'j' => "JULIET",
        'k' => "KILO",
        'l' => "LIMA",
        'm' => "MIKE",
        'n' => "NOVEMBER",
        'o' => "OSCAR",
        'p' => "PAPA",
        'q' => "QUEBEC",
        'r' => "ROMEO",
        's' => "SIERRA",
        't' => "TANGO",
        'u' => "UNIFORM",
        'v' => "VICTOR",
        'w' => "WHISKEY",
        'x' => "XRAY",
        'y' => "YANKEE",
        'z' => "ZULU",
        '0' => "Zero",
        '1' => "One",
        '2' => "Two",
        '3' => "Three",
        '4' => "Four",
        '5' => "Five",
        '6' => "Six",
        '7' => "Seven",
        '8' => "Eight",
        '9' => "Nine",
        '!' => "Exclamation",
        '@' => "At",
        '#' => "Hash",
        '$' => "Dollar",
        '%' => "Percent",
        '^' => "Caret",
        '&' => "Ampersand",
        '*' => "Asterisk",
        '(' => "Left Paren",
        ')' => "Right Paren",
        '-' => "Dash",
        '_' => "Underscore",
        '=' => "Equals",
        '+' => "Plus",
        '[' => "Left Bracket",
        ']' => "Right Bracket",
        '{' => "Left Brace",
        '}' => "Right Brace",
        '|' => "Pipe",
        '\\' => "Backslash",
        ':' => "Colon",
        ';' => "Semicolon",
        '\'' => "Quote",
        '"' => "Double Quote",
        '<' => "Less Than",
        '>' => "Greater Than",
        ',' => "Comma",
        '.' => "Period",
        '/' => "Slash",
        '?' => "Question",
        '`' => "Backtick",
        '~' => "Tilde",
        ' ' => "Space",
        _ => "Symbol",
    }
}

fn get_char_type(c: char) -> &'static str {
    if c.is_ascii_digit() {
        "is-number"
    } else if c.is_ascii_uppercase() {
        "is-upper"
    } else if c.is_ascii_lowercase() {
        ""
    } else {
        "is-special"
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Password {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub otp_secret: Option<String>,
    pub notes: Option<String>,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub folder: Option<String>,
    pub last_changed: String,
    pub created_at: String,
    pub dark_web_alert: bool,
    pub secure_password: bool,
}

#[derive(Clone, PartialEq)]
enum ViewMode {
    List,
    Detail(String), // Password ID
}

#[function_component(PasswordsPage)]
pub fn passwords_page() -> Html {
    let passwords = use_state(|| None::<Vec<Password>>);
    let selected_password = use_state(|| None::<Password>);
    let show_password = use_state(|| false);
    let auto_hide_countdown = use_state(|| 0i32);
    let search_query = use_state(|| String::new());
    let show_create_modal = use_state(|| false);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Auto-hide password after 30 seconds when revealed
    {
        let show_password = show_password.clone();
        let auto_hide_countdown = auto_hide_countdown.clone();

        use_effect_with(*show_password, move |visible| {
            if *visible {
                auto_hide_countdown.set(30);
                let countdown = auto_hide_countdown.clone();
                let show = show_password.clone();

                let interval = Interval::new(1000, move || {
                    let current = *countdown;
                    if current > 0 {
                        countdown.set(current - 1);
                    } else {
                        show.set(false);
                    }
                });

                return move || drop(interval);
            }
            || ()
        });
    }

    // Fetch passwords on mount
    {
        let passwords = passwords.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Mock data for now - replace with actual API call
                let mock_passwords = vec![
                    Password {
                        id: "1".to_string(),
                        name: "Admin Portal".to_string(),
                        username: Some("admin@company.com".to_string()),
                        password: "Tr0ub4dor&3".to_string(),
                        url: Some("https://admin.company.com".to_string()),
                        otp_secret: Some("JBSWY3DPEHPK3PXP".to_string()),
                        notes: Some("Main admin account for company portal".to_string()),
                        client_id: Some("c1".to_string()),
                        client_name: Some("Acme Corp".to_string()),
                        folder: Some("Admin Accounts".to_string()),
                        last_changed: "2024-01-15T10:30:00Z".to_string(),
                        created_at: "2023-06-01T08:00:00Z".to_string(),
                        dark_web_alert: false,
                        secure_password: true,
                    },
                    Password {
                        id: "2".to_string(),
                        name: "VPN Access".to_string(),
                        username: Some("vpnuser".to_string()),
                        password: "SecureVPN#2024!".to_string(),
                        url: Some("https://vpn.company.com".to_string()),
                        otp_secret: None,
                        notes: Some("Use with company VPN client".to_string()),
                        client_id: Some("c1".to_string()),
                        client_name: Some("Acme Corp".to_string()),
                        folder: Some("Network".to_string()),
                        last_changed: "2024-02-20T14:00:00Z".to_string(),
                        created_at: "2023-08-15T09:00:00Z".to_string(),
                        dark_web_alert: false,
                        secure_password: true,
                    },
                    Password {
                        id: "3".to_string(),
                        name: "Cloud Services".to_string(),
                        username: Some("cloud.admin@company.com".to_string()),
                        password: "Cl0ud$3rv!c3".to_string(),
                        url: Some("https://cloud.company.com".to_string()),
                        otp_secret: Some("GEZDGNBVGY3TQOJQ".to_string()),
                        notes: Some("AWS root account - use sparingly".to_string()),
                        client_id: Some("c2".to_string()),
                        client_name: Some("TechStart Inc".to_string()),
                        folder: Some("Cloud".to_string()),
                        last_changed: "2024-03-01T09:00:00Z".to_string(),
                        created_at: "2023-09-01T10:00:00Z".to_string(),
                        dark_web_alert: true,
                        secure_password: false,
                    },
                ];

                passwords.set(Some(mock_passwords));
                loading.set(false);
            });
            || ()
        });
    }

    let toggle_password_visibility = {
        let show_password = show_password.clone();
        Callback::from(move |_| {
            show_password.set(!*show_password);
        })
    };

    let hide_password_now = {
        let show_password = show_password.clone();
        Callback::from(move |_| {
            show_password.set(false);
        })
    };

    let on_password_select = {
        let selected_password = selected_password.clone();
        let show_password = show_password.clone();
        Callback::from(move |password: Password| {
            selected_password.set(Some(password));
            show_password.set(false);
        })
    };

    let on_search = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value());
        })
    };

    // Filter passwords by search query
    let filtered_passwords = passwords.as_ref().map(|list| {
        let query = search_query.to_lowercase();
        if query.is_empty() {
            list.clone()
        } else {
            list.iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&query)
                        || p.username.as_ref().map(|u| u.to_lowercase().contains(&query)).unwrap_or(false)
                        || p.client_name.as_ref().map(|c| c.to_lowercase().contains(&query)).unwrap_or(false)
                })
                .cloned()
                .collect()
        }
    });

    html! {
        <div class="flex h-full" style="background-color: var(--bg-primary);">
            // Left Panel - Password List
            <div class="w-96 flex-shrink-0 border-r flex flex-col" style="border-color: var(--border-primary); background-color: var(--bg-secondary);">
                // Header
                <div class="p-4 border-b" style="border-color: var(--border-primary);">
                    <div class="flex items-center justify-between mb-4">
                        <h1 class="text-xl font-semibold" style="color: var(--fg-primary);">{"Passwords"}</h1>
                        <button
                            onclick={Callback::from(move |_| {})}
                            class="flex items-center space-x-1 px-3 py-1.5 rounded-lg text-sm font-medium"
                            style="background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                            </svg>
                            <span>{"New"}</span>
                        </button>
                    </div>

                    // Search
                    <div class="relative">
                        <svg class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                        </svg>
                        <input
                            type="text"
                            placeholder="Search passwords..."
                            oninput={on_search}
                            class="w-full pl-10 pr-4 py-2 rounded-lg text-sm"
                            style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                        />
                    </div>
                </div>

                // Password List
                <div class="flex-1 overflow-y-auto">
                    if *loading {
                        <div class="p-4 text-center" style="color: var(--fg-muted);">
                            {"Loading passwords..."}
                        </div>
                    } else if let Some(passwords) = &filtered_passwords {
                        { for passwords.iter().map(|password| {
                            let p = password.clone();
                            let on_select = on_password_select.clone();
                            let is_selected = selected_password.as_ref().map(|s| s.id == password.id).unwrap_or(false);

                            html! {
                                <PasswordListItem
                                    password={p.clone()}
                                    selected={is_selected}
                                    on_click={Callback::from(move |_| on_select.emit(p.clone()))}
                                />
                            }
                        })}
                    } else {
                        <div class="p-4 text-center" style="color: var(--fg-muted);">
                            {"No passwords found"}
                        </div>
                    }
                </div>
            </div>

            // Right Panel - Password Detail
            <div class="flex-1 overflow-y-auto" style="background-color: var(--bg-primary);">
                if let Some(password) = (*selected_password).clone() {
                    <PasswordDetail
                        password={password}
                        show_password={*show_password}
                        countdown={*auto_hide_countdown}
                        on_toggle_visibility={toggle_password_visibility.clone()}
                        on_hide_now={hide_password_now.clone()}
                    />
                } else {
                    <div class="h-full flex items-center justify-center">
                        <div class="text-center">
                            <svg class="w-16 h-16 mx-auto mb-4" style="color: var(--fg-dimmed);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/>
                            </svg>
                            <p style="color: var(--fg-muted);">{"Select a password to view details"}</p>
                        </div>
                    </div>
                }
            </div>
        </div>
    }
}

// ===== Password List Item Component =====

#[derive(Properties, PartialEq)]
struct PasswordListItemProps {
    password: Password,
    selected: bool,
    on_click: Callback<()>,
}

#[function_component(PasswordListItem)]
fn password_list_item(props: &PasswordListItemProps) -> Html {
    let onclick = {
        let on_click = props.on_click.clone();
        Callback::from(move |_| on_click.emit(()))
    };

    let bg_style = if props.selected {
        "background-color: var(--bg-highlight);"
    } else {
        ""
    };

    html! {
        <div
            {onclick}
            class="px-4 py-3 cursor-pointer border-b hover:bg-gray-700/50"
            style={format!("border-color: var(--border-primary); {}", bg_style)}
        >
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background-color: var(--accent-blue-dark);">
                        <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/>
                        </svg>
                    </div>
                    <div>
                        <div class="font-medium" style="color: var(--fg-primary);">{&props.password.name}</div>
                        <div class="text-sm" style="color: var(--fg-muted);">
                            {props.password.username.as_deref().unwrap_or("-")}
                        </div>
                    </div>
                </div>
                <div class="flex items-center space-x-2">
                    if props.password.dark_web_alert {
                        <span class="px-2 py-0.5 text-xs rounded" style="background-color: var(--color-error); color: white;">
                            {"Alert"}
                        </span>
                    }
                    if props.password.otp_secret.is_some() {
                        <svg class="w-4 h-4" style="color: var(--color-success);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
                        </svg>
                    }
                </div>
            </div>
            if let Some(client) = &props.password.client_name {
                <div class="mt-1 text-xs" style="color: var(--fg-dimmed);">
                    {client}
                </div>
            }
        </div>
    }
}

// ===== Password Detail Component =====

#[derive(Properties, PartialEq)]
struct PasswordDetailProps {
    password: Password,
    show_password: bool,
    countdown: i32,
    on_toggle_visibility: Callback<()>,
    on_hide_now: Callback<()>,
}

#[function_component(PasswordDetail)]
fn password_detail(props: &PasswordDetailProps) -> Html {
    let copy_to_clipboard = |text: String| {
        if let Some(window) = web_sys::window() {
            if let Some(navigator) = window.navigator().clipboard() {
                let _ = navigator.write_text(&text);
            }
        }
    };

    let copy_username = {
        let username = props.password.username.clone().unwrap_or_default();
        Callback::from(move |_| {
            copy_to_clipboard(username.clone());
        })
    };

    let copy_password = {
        let password = props.password.password.clone();
        Callback::from(move |_| {
            copy_to_clipboard(password.clone());
        })
    };

    let on_toggle = {
        let cb = props.on_toggle_visibility.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let on_hide = {
        let cb = props.on_hide_now.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <div class="p-6 max-w-4xl">
            // Header
            <div class="flex items-center justify-between mb-6">
                <div class="flex items-center space-x-4">
                    <div class="w-12 h-12 rounded-lg flex items-center justify-center" style="background-color: var(--accent-blue-dark);">
                        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/>
                        </svg>
                    </div>
                    <div>
                        <h2 class="text-2xl font-semibold" style="color: var(--fg-primary);">{&props.password.name}</h2>
                        <div class="flex items-center space-x-4 mt-1">
                            if props.password.dark_web_alert {
                                <span class="flex items-center text-sm" style="color: var(--color-error);">
                                    <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                                    </svg>
                                    {"Dark Web Alert"}
                                </span>
                            }
                            if props.password.secure_password {
                                <span class="flex items-center text-sm" style="color: var(--color-success);">
                                    <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
                                    </svg>
                                    {"Secure Password"}
                                </span>
                            }
                        </div>
                    </div>
                </div>

                <div class="flex items-center space-x-2">
                    <button class="p-2 rounded-lg hover:bg-gray-700" style="color: var(--fg-muted);">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                        </svg>
                    </button>
                    <button class="p-2 rounded-lg hover:bg-gray-700" style="color: var(--fg-muted);">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"/>
                        </svg>
                    </button>
                </div>
            </div>

            // Status Bar
            <div class="flex items-center space-x-6 mb-6 text-sm" style="color: var(--fg-muted);">
                <div>
                    <span>{"Last Changed: "}</span>
                    <span style="color: var(--fg-secondary);">{"less than a minute ago"}</span>
                </div>
            </div>

            // Overview Section
            <div class="rounded-lg p-6 mb-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Overview"}</h3>

                // Username Field
                <div class="mb-4">
                    <label class="block text-sm mb-2" style="color: var(--fg-muted);">{"Username"}</label>
                    <div class="flex items-center">
                        <input
                            type="text"
                            value={props.password.username.clone().unwrap_or_default()}
                            readonly=true
                            class="flex-1 px-4 py-2 rounded-l-lg font-mono"
                            style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                        />
                        <button
                            onclick={copy_username}
                            class="px-4 py-2 rounded-r-lg border-l-0"
                            style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-muted);"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                            </svg>
                        </button>
                    </div>
                </div>

                // Password Field
                <div class="mb-4">
                    <label class="block text-sm mb-2" style="color: var(--fg-muted);">{"Password"}</label>
                    <div class="flex items-center">
                        <input
                            type={if props.show_password { "text" } else { "password" }}
                            value={props.password.password.clone()}
                            readonly=true
                            class="flex-1 px-4 py-2 rounded-l-lg font-mono"
                            style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                        />
                        <button
                            onclick={on_toggle}
                            class="px-4 py-2 border-l-0"
                            style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-muted);"
                        >
                            if props.show_password {
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"/>
                                </svg>
                            } else {
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
                                </svg>
                            }
                        </button>
                        <button
                            onclick={copy_password}
                            class="px-4 py-2 rounded-r-lg border-l-0"
                            style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-muted);"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                            </svg>
                        </button>
                    </div>

                    // Auto-hide countdown
                    if props.show_password && props.countdown > 0 {
                        <div class="mt-2 flex items-center text-sm" style="color: var(--fg-muted);">
                            <span>{format!("Hides in {} seconds ", props.countdown)}</span>
                            <button
                                onclick={on_hide}
                                class="ml-2 underline"
                                style="color: var(--accent-primary);"
                            >
                                {"(hide now)"}
                            </button>
                        </div>
                    }
                </div>

                // Phonetic Display (only when password is visible)
                if props.show_password {
                    <PhoneticDisplay password={props.password.password.clone()} />
                }
            </div>

            // OTP Section (if available)
            if props.password.otp_secret.is_some() {
                <div class="rounded-lg p-6 mb-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"One-Time Password (OTP)"}</h3>
                    <OtpDisplay secret={props.password.otp_secret.clone().unwrap_or_default()} />
                </div>
            }

            // Share Password Via Link
            <div class="rounded-lg p-6 mb-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Share Password Via Link"}</h3>

                <div class="mb-4">
                    <label class="block text-sm mb-2" style="color: var(--fg-muted);">
                        {"Expires In"}
                        <span style="color: var(--color-error);">{"*"}</span>
                    </label>
                    <select class="w-full px-4 py-2 rounded-lg" style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);">
                        <option>{"30 minutes"}</option>
                        <option>{"1 hour"}</option>
                        <option>{"4 hours"}</option>
                        <option>{"24 hours"}</option>
                    </select>
                </div>

                <div class="mb-4">
                    <label class="block text-sm mb-2" style="color: var(--fg-muted);">{"Message"}</label>
                    <textarea
                        rows="3"
                        class="w-full px-4 py-2 rounded-lg"
                        style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                        placeholder="Optional message to recipient..."
                    />
                </div>

                <div class="flex items-center space-x-4 mb-4">
                    <label class="flex items-center space-x-2 cursor-pointer">
                        <input type="checkbox" class="rounded" />
                        <span class="text-sm" style="color: var(--fg-secondary);">{"Include Username"}</span>
                    </label>
                </div>

                <button
                    class="px-4 py-2 rounded-lg font-medium"
                    style="background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                >
                    {"Generate Share Link"}
                </button>
            </div>

            // Notes Section
            if let Some(notes) = &props.password.notes {
                <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Notes"}</h3>
                    <div class="whitespace-pre-wrap" style="color: var(--fg-secondary);">
                        {notes}
                    </div>
                </div>
            }
        </div>
    }
}

// ===== Phonetic Display Component =====

#[derive(Properties, PartialEq)]
struct PhoneticDisplayProps {
    password: String,
}

#[function_component(PhoneticDisplay)]
fn phonetic_display(props: &PhoneticDisplayProps) -> Html {
    html! {
        <div class="mt-4 p-4 rounded-lg" style="background-color: var(--bg-tertiary);">
            <div class="flex flex-wrap gap-3">
                { for props.password.chars().map(|c| {
                    let char_type = get_char_type(c);
                    let phonetic = get_phonetic(c);

                    html! {
                        <div class="flex flex-col items-center min-w-[2.5rem]">
                            <span
                                class={format!("text-2xl font-semibold font-mono {}", char_type)}
                                style={match char_type {
                                    "is-number" => "color: var(--syntax-number);",
                                    "is-upper" => "color: var(--accent-primary);",
                                    "is-special" => "color: var(--color-error);",
                                    _ => "color: var(--fg-primary);",
                                }}
                            >
                                {c}
                            </span>
                            <span class="text-xs mt-1" style="color: var(--fg-muted);">
                                {phonetic.to_lowercase()}
                            </span>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

// ===== OTP Display Component =====

#[derive(Properties, PartialEq)]
struct OtpDisplayProps {
    secret: String,
}

#[function_component(OtpDisplay)]
fn otp_display(props: &OtpDisplayProps) -> Html {
    // In a real app, this would generate actual TOTP codes
    // For now, showing a placeholder
    let otp_code = use_state(|| "123 456".to_string());
    let time_remaining = use_state(|| 30);

    // Countdown timer effect
    {
        let time_remaining = time_remaining.clone();
        use_effect_with((), move |_| {
            let interval = Interval::new(1000, move || {
                let current = *time_remaining;
                if current > 0 {
                    time_remaining.set(current - 1);
                } else {
                    time_remaining.set(30);
                }
            });
            move || drop(interval)
        });
    }

    let copy_otp = {
        let code = otp_code.clone();
        Callback::from(move |_| {
            if let Some(window) = web_sys::window() {
                if let Some(navigator) = window.navigator().clipboard() {
                    let _ = navigator.write_text(&code.replace(' ', ""));
                }
            }
        })
    };

    html! {
        <div class="flex items-center space-x-4">
            <div class="flex-1">
                <div class="flex items-center space-x-4">
                    <span
                        class="text-3xl font-mono font-bold tracking-widest"
                        style="color: var(--color-success-muted);"
                    >
                        {&*otp_code}
                    </span>
                    <button
                        onclick={copy_otp}
                        class="p-2 rounded-lg hover:bg-gray-700"
                        style="color: var(--fg-muted);"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                        </svg>
                    </button>
                </div>
                <div class="mt-2 flex items-center space-x-2">
                    <div class="w-32 h-1 rounded-full overflow-hidden" style="background-color: var(--bg-highlight);">
                        <div
                            class="h-full transition-all duration-1000"
                            style={format!(
                                "width: {}%; background-color: {};",
                                (*time_remaining as f32 / 30.0 * 100.0) as i32,
                                if *time_remaining < 10 { "var(--color-error)" } else { "var(--color-success)" }
                            )}
                        />
                    </div>
                    <span class="text-sm" style="color: var(--fg-muted);">
                        {format!("{}s", *time_remaining)}
                    </span>
                </div>
            </div>
        </div>
    }
}
