use yew::prelude::*;
use yew_router::prelude::*;
use super::{time_tracker::TimeTracker, AuthContext};
use crate::theme::{ThemeSelector, use_theme};

// Define Route here for now, will be moved to a routes module later
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Dashboard,
    #[at("/login")]
    Login,
    #[at("/service-desk")]
    ServiceDesk,
    #[at("/clients")]
    Clients,
    #[at("/tickets")]
    Tickets,
    #[at("/tickets/:id")]
    TicketDetail { id: String },
    #[at("/time")]
    TimeTracking,
    #[at("/assets")]
    Assets,
    #[at("/invoices")]
    Invoices,
    #[at("/projects")]
    Projects,
    #[at("/kb")]
    KnowledgeBase,
    #[at("/passwords")]
    Passwords,
    #[at("/reports")]
    Reports,
    #[at("/admin")]
    Admin,
    #[at("/m365")]
    M365,
    #[at("/azure")]
    Azure,
    #[at("/bitwarden")]
    Bitwarden,
    #[at("/network")]
    Network,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Html,
}

/// BMS-style dark theme layout with persistent header timer
#[function_component(Layout)]
pub fn layout(props: &LayoutProps) -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthContext not found");
    let _theme_ctx = use_theme(); // Access theme context for reactivity
    let current_route = use_route::<Route>().unwrap_or(Route::Dashboard);
    let sidebar_collapsed = use_state(|| false);
    let show_notifications = use_state(|| false);
    let show_user_menu = use_state(|| false);

    // Toggle sidebar
    let toggle_sidebar = {
        let sidebar_collapsed = sidebar_collapsed.clone();
        Callback::from(move |_| sidebar_collapsed.set(!*sidebar_collapsed))
    };

    // Toggle notifications
    let toggle_notifications = {
        let show_notifications = show_notifications.clone();
        Callback::from(move |_| show_notifications.set(!*show_notifications))
    };

    // Toggle user menu
    let toggle_user_menu = {
        let show_user_menu = show_user_menu.clone();
        Callback::from(move |_| show_user_menu.set(!*show_user_menu))
    };

    let is_nav_active = |route: &Route| -> &'static str {
        if route == &current_route {
            "bg-blue-600 text-white"
        } else {
            "text-gray-300 hover:bg-gray-700 hover:text-white"
        }
    };

    let sidebar_width = if *sidebar_collapsed { "w-16" } else { "w-64" };

    html! {
        <div class="min-h-screen bg-gray-900 flex flex-col">
            // ===== TOP HEADER BAR (BMS Style - Always Visible) =====
            <header class="bg-gray-800 border-b border-gray-700 h-14 flex-shrink-0 z-50">
                <div class="h-full flex items-center justify-between px-4">
                    // Left: Logo and main navigation tabs
                    <div class="flex items-center space-x-6">
                        // Logo
                        <div class="flex items-center space-x-2">
                            <div class="w-8 h-8 bg-blue-500 rounded flex items-center justify-center">
                                <span class="text-white font-bold text-lg">{"R"}</span>
                            </div>
                            if !*sidebar_collapsed {
                                <span class="text-white font-semibold text-lg">{"Resolve"}</span>
                            }
                        </div>

                        // Main Navigation Tabs (BMS Style)
                        <nav class="hidden lg:flex items-center space-x-1">
                            <NavTab route={Route::Dashboard} label="Home" current={current_route.clone()} />
                            <NavTab route={Route::ServiceDesk} label="Service Desk" current={current_route.clone()} />
                            <NavTab route={Route::Clients} label="CRM" current={current_route.clone()} />
                            <NavTab route={Route::Invoices} label="Finance" current={current_route.clone()} />
                            <NavTab route={Route::Projects} label="Projects" current={current_route.clone()} />
                            <NavTab route={Route::Reports} label="Reports" current={current_route.clone()} />
                            <NavTab route={Route::Admin} label="Admin" current={current_route.clone()} />
                        </nav>
                    </div>

                    // Right: Timer Widget, Search, Notifications, User
                    <div class="flex items-center space-x-4">
                        // ===== PERSISTENT TIMER WIDGET (KEY FEATURE) =====
                        <div class="flex items-center bg-gray-700 rounded-lg px-3 py-1.5">
                            <TimeTracker compact={true} header_mode={true} />
                        </div>

                        // Global Search (Cmd+K)
                        <button class="hidden md:flex items-center space-x-2 bg-gray-700 hover:bg-gray-600 rounded-lg px-3 py-1.5 text-gray-300 text-sm">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                            </svg>
                            <span>{"Search..."}</span>
                            <kbd class="bg-gray-600 px-1.5 py-0.5 rounded text-xs">{"⌘K"}</kbd>
                        </button>

                        // New Ticket Button (Quick Action)
                        <Link<Route> to={Route::Tickets}
                            classes="bg-blue-600 hover:bg-blue-700 text-white px-4 py-1.5 rounded-lg text-sm font-medium flex items-center space-x-1">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                            </svg>
                            <span>{"New Ticket"}</span>
                        </Link<Route>>

                        // Theme Selector
                        <ThemeSelector compact={true} />

                        // Notifications
                        <div class="relative">
                            <button
                                onclick={toggle_notifications}
                                class="text-gray-300 hover:text-white p-2 rounded-lg hover:bg-gray-700 relative"
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
                                </svg>
                                // Notification badge
                                <span class="absolute top-1 right-1 w-2 h-2 bg-red-500 rounded-full"></span>
                            </button>

                            // Notifications dropdown
                            if *show_notifications {
                                <div class="absolute right-0 mt-2 w-80 bg-gray-800 rounded-lg shadow-lg border border-gray-700 py-2">
                                    <div class="px-4 py-2 border-b border-gray-700">
                                        <h3 class="text-white font-medium">{"Notifications"}</h3>
                                    </div>
                                    <div class="max-h-96 overflow-y-auto">
                                        <NotificationItem
                                            title="Ticket #1234 Updated"
                                            message="John replied to your ticket"
                                            time="5 min ago"
                                        />
                                        <NotificationItem
                                            title="SLA Breach Warning"
                                            message="Ticket #1235 is approaching SLA deadline"
                                            time="15 min ago"
                                        />
                                    </div>
                                </div>
                            }
                        </div>

                        // User Menu
                        <div class="relative">
                            <button
                                onclick={toggle_user_menu}
                                class="flex items-center space-x-2 text-gray-300 hover:text-white"
                            >
                                <div class="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                                    <span class="text-white text-sm font-medium">
                                        {auth_ctx.user.as_ref().map(|u| u.first_name.chars().next().unwrap_or('U')).unwrap_or('U')}
                                    </span>
                                </div>
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                                </svg>
                            </button>

                            if *show_user_menu {
                                <div class="absolute right-0 mt-2 w-56 bg-gray-800 rounded-lg shadow-lg border border-gray-700 py-2">
                                    <div class="px-4 py-3 border-b border-gray-700">
                                        <p class="text-white font-medium">
                                            {auth_ctx.user.as_ref().map(|u| format!("{} {}", u.first_name, u.last_name)).unwrap_or_else(|| "User".to_string())}
                                        </p>
                                        <p class="text-gray-400 text-sm">
                                            {auth_ctx.user.as_ref().map(|u| u.email.clone()).unwrap_or_default()}
                                        </p>
                                    </div>
                                    <div class="py-1">
                                        <a href="#" class="block px-4 py-2 text-gray-300 hover:bg-gray-700">{"My Profile"}</a>
                                        <a href="#" class="block px-4 py-2 text-gray-300 hover:bg-gray-700">{"Settings"}</a>
                                        <a href="#" class="block px-4 py-2 text-gray-300 hover:bg-gray-700">{"API Keys"}</a>
                                    </div>
                                    <div class="border-t border-gray-700 py-1">
                                        <button
                                            onclick={auth_ctx.logout.reform(|_| ())}
                                            class="w-full text-left px-4 py-2 text-red-400 hover:bg-gray-700"
                                        >
                                            {"Sign Out"}
                                        </button>
                                    </div>
                                </div>
                            }
                        </div>
                    </div>
                </div>
            </header>

            // ===== MAIN CONTENT AREA =====
            <div class="flex flex-1 overflow-hidden">
                // Left Sidebar (Collapsible)
                <aside class={format!("bg-gray-800 border-r border-gray-700 flex-shrink-0 transition-all duration-200 {}", sidebar_width)}>
                    <div class="h-full flex flex-col">
                        // Sidebar toggle
                        <div class="p-2 border-b border-gray-700">
                            <button
                                onclick={toggle_sidebar}
                                class="w-full text-gray-400 hover:text-white p-2 rounded hover:bg-gray-700"
                            >
                                if *sidebar_collapsed {
                                    <svg class="w-5 h-5 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7"/>
                                    </svg>
                                } else {
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7"/>
                                    </svg>
                                }
                            </button>
                        </div>

                        // Context-sensitive navigation
                        <nav class="flex-1 overflow-y-auto py-4">
                            <SidebarSection title="Service Desk" collapsed={*sidebar_collapsed}>
                                <SidebarLink route={Route::Tickets} icon="ticket" label="All Tickets" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                                <SidebarLink route={Route::Tickets} icon="user" label="My Tickets" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                                <SidebarLink route={Route::Tickets} icon="clock" label="Overdue" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                            </SidebarSection>

                            <SidebarSection title="Documentation" collapsed={*sidebar_collapsed}>
                                <SidebarLink route={Route::KnowledgeBase} icon="book" label="Knowledge Base" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                                <SidebarLink route={Route::Passwords} icon="key" label="Passwords" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                                <SidebarLink route={Route::Assets} icon="server" label="Assets" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                            </SidebarSection>

                            <SidebarSection title="Integrations" collapsed={*sidebar_collapsed}>
                                <SidebarLink route={Route::M365} icon="microsoft" label="Microsoft 365" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                                <SidebarLink route={Route::Azure} icon="cloud" label="Azure" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                                <SidebarLink route={Route::Network} icon="network" label="Network" collapsed={*sidebar_collapsed} current={current_route.clone()} />
                            </SidebarSection>
                        </nav>
                    </div>
                </aside>

                // Main Content
                <main class="flex-1 overflow-auto bg-gray-900">
                    { props.children.clone() }
                </main>
            </div>
        </div>
    }
}

// ===== HELPER COMPONENTS =====

#[derive(Properties, PartialEq)]
struct NavTabProps {
    route: Route,
    label: &'static str,
    current: Route,
}

#[function_component(NavTab)]
fn nav_tab(props: &NavTabProps) -> Html {
    let is_active = props.route == props.current;
    let classes = if is_active {
        "px-3 py-2 text-sm font-medium text-white border-b-2 border-blue-500"
    } else {
        "px-3 py-2 text-sm font-medium text-gray-300 hover:text-white border-b-2 border-transparent hover:border-gray-500"
    };

    html! {
        <Link<Route> to={props.route.clone()} classes={classes}>
            {props.label}
        </Link<Route>>
    }
}

#[derive(Properties, PartialEq)]
struct SidebarSectionProps {
    title: &'static str,
    collapsed: bool,
    children: Html,
}

#[function_component(SidebarSection)]
fn sidebar_section(props: &SidebarSectionProps) -> Html {
    html! {
        <div class="mb-6">
            if !props.collapsed {
                <h3 class="px-4 text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                    {props.title}
                </h3>
            }
            <div class="space-y-1 px-2">
                {props.children.clone()}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct SidebarLinkProps {
    route: Route,
    icon: &'static str,
    label: &'static str,
    collapsed: bool,
    current: Route,
}

#[function_component(SidebarLink)]
fn sidebar_link(props: &SidebarLinkProps) -> Html {
    let is_active = props.route == props.current;
    let classes = if is_active {
        "flex items-center px-3 py-2 rounded-lg bg-blue-600 text-white"
    } else {
        "flex items-center px-3 py-2 rounded-lg text-gray-300 hover:bg-gray-700 hover:text-white"
    };

    let icon = match props.icon {
        "ticket" => html! {
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z"/>
            </svg>
        },
        "user" => html! {
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
            </svg>
        },
        "clock" => html! {
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
        },
        "book" => html! {
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
            </svg>
        },
        "key" => html! {
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/>
            </svg>
        },
        "server" => html! {
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"/>
            </svg>
        },
        "microsoft" => html! {
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M11.4 24H0V12.6h11.4V24zM24 24H12.6V12.6H24V24zM11.4 11.4H0V0h11.4v11.4zm12.6 0H12.6V0H24v11.4z"/>
            </svg>
        },
        "cloud" => html! {
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z"/>
            </svg>
        },
        "network" => html! {
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"/>
            </svg>
        },
        _ => html! { <span class="w-5 h-5"></span> },
    };

    html! {
        <Link<Route> to={props.route.clone()} classes={classes}>
            {icon}
            if !props.collapsed {
                <span class="ml-3">{props.label}</span>
            }
        </Link<Route>>
    }
}

#[derive(Properties, PartialEq)]
struct NotificationItemProps {
    title: &'static str,
    message: &'static str,
    time: &'static str,
}

#[function_component(NotificationItem)]
fn notification_item(props: &NotificationItemProps) -> Html {
    html! {
        <div class="px-4 py-3 hover:bg-gray-700 cursor-pointer">
            <p class="text-white text-sm font-medium">{props.title}</p>
            <p class="text-gray-400 text-sm">{props.message}</p>
            <p class="text-gray-500 text-xs mt-1">{props.time}</p>
        </div>
    }
}
