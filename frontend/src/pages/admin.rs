// Admin/Settings Page with Theme Selector

use yew::prelude::*;
use crate::theme::{ThemeSelector, ColorPalette, use_theme};

#[derive(Clone, Copy, PartialEq)]
enum SettingsTab {
    Appearance,
    General,
    Security,
    Integrations,
    Users,
    Billing,
}

#[function_component(AdminPage)]
pub fn admin_page() -> Html {
    let active_tab = use_state(|| SettingsTab::Appearance);
    let theme_ctx = use_theme();

    let set_tab = |tab: SettingsTab| {
        let active_tab = active_tab.clone();
        Callback::from(move |_| active_tab.set(tab))
    };

    let tab_class = |tab: SettingsTab| -> String {
        let base = "px-4 py-2 text-sm font-medium rounded-lg transition-colors";
        if *active_tab == tab {
            format!("{} bg-blue-600 text-white", base)
        } else {
            format!("{} text-gray-400 hover:text-white hover:bg-gray-700", base)
        }
    };

    html! {
        <div class="p-6" style="background-color: var(--bg-primary); min-height: 100vh;">
            <div class="max-w-6xl mx-auto">
                // Header
                <div class="mb-8">
                    <h1 class="text-2xl font-bold" style="color: var(--fg-primary);">{"Settings"}</h1>
                    <p class="mt-1" style="color: var(--fg-muted);">{"Manage your workspace preferences and configurations"}</p>
                </div>

                <div class="flex gap-8">
                    // Sidebar Navigation
                    <div class="w-48 flex-shrink-0">
                        <nav class="space-y-1">
                            <button onclick={set_tab(SettingsTab::Appearance)} class={tab_class(SettingsTab::Appearance)}>
                                <div class="flex items-center space-x-2">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01"/>
                                    </svg>
                                    <span>{"Appearance"}</span>
                                </div>
                            </button>
                            <button onclick={set_tab(SettingsTab::General)} class={tab_class(SettingsTab::General)}>
                                <div class="flex items-center space-x-2">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                    </svg>
                                    <span>{"General"}</span>
                                </div>
                            </button>
                            <button onclick={set_tab(SettingsTab::Security)} class={tab_class(SettingsTab::Security)}>
                                <div class="flex items-center space-x-2">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
                                    </svg>
                                    <span>{"Security"}</span>
                                </div>
                            </button>
                            <button onclick={set_tab(SettingsTab::Integrations)} class={tab_class(SettingsTab::Integrations)}>
                                <div class="flex items-center space-x-2">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z"/>
                                    </svg>
                                    <span>{"Integrations"}</span>
                                </div>
                            </button>
                            <button onclick={set_tab(SettingsTab::Users)} class={tab_class(SettingsTab::Users)}>
                                <div class="flex items-center space-x-2">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"/>
                                    </svg>
                                    <span>{"Users"}</span>
                                </div>
                            </button>
                            <button onclick={set_tab(SettingsTab::Billing)} class={tab_class(SettingsTab::Billing)}>
                                <div class="flex items-center space-x-2">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z"/>
                                    </svg>
                                    <span>{"Billing"}</span>
                                </div>
                            </button>
                        </nav>
                    </div>

                    // Main Content
                    <div class="flex-1">
                        {match *active_tab {
                            SettingsTab::Appearance => html! { <AppearanceSettings /> },
                            SettingsTab::General => html! { <GeneralSettings /> },
                            SettingsTab::Security => html! { <SecuritySettings /> },
                            SettingsTab::Integrations => html! { <IntegrationsSettings /> },
                            SettingsTab::Users => html! { <UsersSettings /> },
                            SettingsTab::Billing => html! { <BillingSettings /> },
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

// ===== Appearance Settings =====

#[function_component(AppearanceSettings)]
fn appearance_settings() -> Html {
    html! {
        <div class="space-y-8">
            // Theme Section
            <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <ThemeSelector compact={false} />
                <div class="mt-6">
                    <ColorPalette />
                </div>
            </div>

            // Font Settings
            <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Font Settings"}</h3>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm mb-2" style="color: var(--fg-muted);">{"UI Font"}</label>
                        <select class="w-full px-4 py-2 rounded-lg" style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);">
                            <option>{"Inter"}</option>
                            <option>{"System Default"}</option>
                        </select>
                    </div>
                    <div>
                        <label class="block text-sm mb-2" style="color: var(--fg-muted);">{"Monospace Font"}</label>
                        <select class="w-full px-4 py-2 rounded-lg" style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);">
                            <option>{"Fira Code"}</option>
                            <option>{"JetBrains Mono"}</option>
                            <option>{"Cascadia Code"}</option>
                        </select>
                    </div>
                </div>

                <div class="mt-4">
                    <label class="flex items-center space-x-2 cursor-pointer">
                        <input type="checkbox" class="rounded" checked=true />
                        <span class="text-sm" style="color: var(--fg-secondary);">{"Enable font ligatures"}</span>
                    </label>
                </div>
            </div>

            // Display Preferences
            <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Display Preferences"}</h3>

                <div class="space-y-4">
                    <label class="flex items-center justify-between cursor-pointer">
                        <div>
                            <span class="text-sm font-medium" style="color: var(--fg-primary);">{"Compact Mode"}</span>
                            <p class="text-xs" style="color: var(--fg-muted);">{"Reduce spacing and padding throughout the UI"}</p>
                        </div>
                        <input type="checkbox" class="rounded" />
                    </label>

                    <label class="flex items-center justify-between cursor-pointer">
                        <div>
                            <span class="text-sm font-medium" style="color: var(--fg-primary);">{"Animate Transitions"}</span>
                            <p class="text-xs" style="color: var(--fg-muted);">{"Enable smooth transitions and animations"}</p>
                        </div>
                        <input type="checkbox" class="rounded" checked=true />
                    </label>

                    <label class="flex items-center justify-between cursor-pointer">
                        <div>
                            <span class="text-sm font-medium" style="color: var(--fg-primary);">{"Show Sidebar by Default"}</span>
                            <p class="text-xs" style="color: var(--fg-muted);">{"Keep the navigation sidebar expanded"}</p>
                        </div>
                        <input type="checkbox" class="rounded" checked=true />
                    </label>
                </div>
            </div>
        </div>
    }
}

// ===== General Settings =====

#[function_component(GeneralSettings)]
fn general_settings() -> Html {
    html! {
        <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
            <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"General Settings"}</h3>

            <div class="space-y-4">
                <div>
                    <label class="block text-sm mb-2" style="color: var(--fg-muted);">{"Company Name"}</label>
                    <input
                        type="text"
                        value="Resolve MSP"
                        class="w-full px-4 py-2 rounded-lg"
                        style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                    />
                </div>

                <div>
                    <label class="block text-sm mb-2" style="color: var(--fg-muted);">{"Timezone"}</label>
                    <select class="w-full px-4 py-2 rounded-lg" style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);">
                        <option>{"(UTC-05:00) Eastern Time"}</option>
                        <option>{"(UTC-06:00) Central Time"}</option>
                        <option>{"(UTC-07:00) Mountain Time"}</option>
                        <option>{"(UTC-08:00) Pacific Time"}</option>
                    </select>
                </div>

                <div>
                    <label class="block text-sm mb-2" style="color: var(--fg-muted);">{"Date Format"}</label>
                    <select class="w-full px-4 py-2 rounded-lg" style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);">
                        <option>{"MM/DD/YYYY"}</option>
                        <option>{"DD/MM/YYYY"}</option>
                        <option>{"YYYY-MM-DD"}</option>
                    </select>
                </div>
            </div>

            <div class="mt-6 pt-6 border-t" style="border-color: var(--border-primary);">
                <button
                    class="px-4 py-2 rounded-lg font-medium"
                    style="background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                >
                    {"Save Changes"}
                </button>
            </div>
        </div>
    }
}

// ===== Security Settings =====

#[function_component(SecuritySettings)]
fn security_settings() -> Html {
    html! {
        <div class="space-y-6">
            <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Password Policy"}</h3>

                <div class="space-y-4">
                    <label class="flex items-center space-x-3 cursor-pointer">
                        <input type="checkbox" class="rounded" checked=true />
                        <span class="text-sm" style="color: var(--fg-secondary);">{"Require minimum 12 characters"}</span>
                    </label>
                    <label class="flex items-center space-x-3 cursor-pointer">
                        <input type="checkbox" class="rounded" checked=true />
                        <span class="text-sm" style="color: var(--fg-secondary);">{"Require uppercase and lowercase letters"}</span>
                    </label>
                    <label class="flex items-center space-x-3 cursor-pointer">
                        <input type="checkbox" class="rounded" checked=true />
                        <span class="text-sm" style="color: var(--fg-secondary);">{"Require numbers"}</span>
                    </label>
                    <label class="flex items-center space-x-3 cursor-pointer">
                        <input type="checkbox" class="rounded" checked=true />
                        <span class="text-sm" style="color: var(--fg-secondary);">{"Require special characters"}</span>
                    </label>
                </div>
            </div>

            <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Two-Factor Authentication"}</h3>
                <p class="text-sm mb-4" style="color: var(--fg-muted);">{"Require 2FA for all users"}</p>

                <label class="flex items-center justify-between cursor-pointer">
                    <span class="text-sm" style="color: var(--fg-secondary);">{"Enforce 2FA"}</span>
                    <input type="checkbox" class="rounded" />
                </label>
            </div>
        </div>
    }
}

// ===== Integrations Settings =====

#[function_component(IntegrationsSettings)]
fn integrations_settings() -> Html {
    let integrations = vec![
        ("Microsoft 365", "Connected", "microsoft", true),
        ("Azure AD", "Connected", "azure", true),
        ("Bitwarden", "Not Connected", "bitwarden", false),
        ("Slack", "Not Connected", "slack", false),
        ("Teams", "Connected", "teams", true),
    ];

    html! {
        <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
            <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Integrations"}</h3>

            <div class="space-y-4">
                { for integrations.iter().map(|(name, status, _icon, connected)| {
                    html! {
                        <div class="flex items-center justify-between p-4 rounded-lg" style="background-color: var(--bg-tertiary);">
                            <div class="flex items-center space-x-3">
                                <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background-color: var(--bg-highlight);">
                                    <svg class="w-5 h-5" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z"/>
                                    </svg>
                                </div>
                                <div>
                                    <div class="font-medium" style="color: var(--fg-primary);">{name}</div>
                                    <div class="text-sm" style={if *connected { "color: var(--color-success);" } else { "color: var(--fg-muted);" }}>
                                        {status}
                                    </div>
                                </div>
                            </div>
                            <button
                                class="px-4 py-2 rounded-lg text-sm font-medium"
                                style={if *connected {
                                    "background-color: var(--button-secondary-bg); color: var(--fg-secondary);"
                                } else {
                                    "background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                                }}
                            >
                                {if *connected { "Configure" } else { "Connect" }}
                            </button>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

// ===== Users Settings =====

#[function_component(UsersSettings)]
fn users_settings() -> Html {
    let users = vec![
        ("John Doe", "john@company.com", "Admin", true),
        ("Jane Smith", "jane@company.com", "Technician", true),
        ("Bob Wilson", "bob@company.com", "Technician", false),
    ];

    html! {
        <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-medium" style="color: var(--fg-primary);">{"Team Members"}</h3>
                <button
                    class="px-4 py-2 rounded-lg text-sm font-medium"
                    style="background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                >
                    {"Invite User"}
                </button>
            </div>

            <table class="w-full">
                <thead>
                    <tr>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted); border-bottom: 1px solid var(--border-primary);">{"Name"}</th>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted); border-bottom: 1px solid var(--border-primary);">{"Email"}</th>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted); border-bottom: 1px solid var(--border-primary);">{"Role"}</th>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted); border-bottom: 1px solid var(--border-primary);">{"Status"}</th>
                        <th class="py-3 px-4" style="border-bottom: 1px solid var(--border-primary);"></th>
                    </tr>
                </thead>
                <tbody>
                    { for users.iter().map(|(name, email, role, active)| {
                        html! {
                            <tr>
                                <td class="py-3 px-4" style="color: var(--fg-primary); border-bottom: 1px solid var(--border-primary);">{name}</td>
                                <td class="py-3 px-4" style="color: var(--fg-secondary); border-bottom: 1px solid var(--border-primary);">{email}</td>
                                <td class="py-3 px-4" style="color: var(--fg-secondary); border-bottom: 1px solid var(--border-primary);">{role}</td>
                                <td class="py-3 px-4" style="border-bottom: 1px solid var(--border-primary);">
                                    <span
                                        class="px-2 py-1 text-xs rounded"
                                        style={if *active {
                                            "background-color: var(--color-success); color: var(--bg-primary);"
                                        } else {
                                            "background-color: var(--fg-dimmed); color: white;"
                                        }}
                                    >
                                        {if *active { "Active" } else { "Inactive" }}
                                    </span>
                                </td>
                                <td class="py-3 px-4" style="border-bottom: 1px solid var(--border-primary);">
                                    <button class="p-1 rounded hover:bg-gray-700" style="color: var(--fg-muted);">
                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"/>
                                        </svg>
                                    </button>
                                </td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        </div>
    }
}

// ===== Billing Settings =====

#[function_component(BillingSettings)]
fn billing_settings() -> Html {
    html! {
        <div class="space-y-6">
            <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Current Plan"}</h3>

                <div class="flex items-center justify-between p-4 rounded-lg" style="background-color: var(--bg-tertiary);">
                    <div>
                        <div class="font-semibold" style="color: var(--fg-primary);">{"Professional"}</div>
                        <div class="text-sm" style="color: var(--fg-muted);">{"$99/month • Unlimited users"}</div>
                    </div>
                    <button
                        class="px-4 py-2 rounded-lg text-sm font-medium"
                        style="background-color: var(--button-secondary-bg); color: var(--fg-secondary);"
                    >
                        {"Upgrade Plan"}
                    </button>
                </div>
            </div>

            <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Payment Method"}</h3>

                <div class="flex items-center space-x-4 p-4 rounded-lg" style="background-color: var(--bg-tertiary);">
                    <div class="w-12 h-8 rounded flex items-center justify-center" style="background-color: var(--bg-highlight);">
                        <svg class="w-8 h-5" style="color: var(--accent-primary);" viewBox="0 0 32 20" fill="currentColor">
                            <rect width="32" height="20" rx="2" fill="currentColor" opacity="0.2"/>
                            <text x="4" y="14" font-size="8" fill="currentColor">{"VISA"}</text>
                        </svg>
                    </div>
                    <div>
                        <div class="font-medium" style="color: var(--fg-primary);">{"•••• •••• •••• 4242"}</div>
                        <div class="text-sm" style="color: var(--fg-muted);">{"Expires 12/25"}</div>
                    </div>
                    <button class="ml-auto text-sm" style="color: var(--accent-primary);">{"Update"}</button>
                </div>
            </div>
        </div>
    }
}
