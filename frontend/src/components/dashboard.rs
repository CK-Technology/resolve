use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::dashboard::{self, DashboardStats};

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    let stats = use_state(|| None::<DashboardStats>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Fetch dashboard stats on component mount
    {
        let stats = stats.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match dashboard::get_stats().await {
                    Ok(data) => {
                        stats.set(Some(data));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e.message));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    if *loading {
        return html! {
            <div class="flex justify-center items-center h-64">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
            </div>
        };
    }

    if let Some(error_msg) = (*error).clone() {
        return html! {
            <div class="p-6">
                <div class="bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 rounded-lg">
                    <div class="flex items-center">
                        <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
                        </svg>
                        {"Error loading dashboard: "}{error_msg}
                    </div>
                </div>
            </div>
        };
    }

    let stats_data = match (*stats).clone() {
        Some(data) => data,
        None => return html! { <div class="p-6 text-gray-400">{"No data available"}</div> },
    };

    html! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white">{"Dashboard"}</h1>
                    <p class="text-gray-400">{"Overview of your MSP operations"}</p>
                </div>
                <button class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg flex items-center space-x-2">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                    </svg>
                    <span>{"Refresh"}</span>
                </button>
            </div>

            // Key metrics cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                <MetricCard
                    title="Active Clients"
                    value={stats_data.overview.total_clients.to_string()}
                    icon="users"
                    color="blue"
                    trend={None}
                />
                <MetricCard
                    title="Open Tickets"
                    value={stats_data.overview.active_tickets.to_string()}
                    icon="ticket"
                    color="yellow"
                    trend={None}
                />
                <MetricCard
                    title="Monthly Revenue"
                    value={format!("${:.0}", stats_data.overview.monthly_revenue)}
                    icon="dollar"
                    color="green"
                    trend={Some((12.5, true))}
                />
                <MetricCard
                    title="Unbilled Time"
                    value={format!("${:.0}", stats_data.overview.unbilled_time)}
                    icon="clock"
                    color="orange"
                    trend={None}
                />
                <MetricCard
                    title="Overdue Invoices"
                    value={stats_data.overview.overdue_invoices.to_string()}
                    icon="alert"
                    color="red"
                    trend={None}
                />
            </div>

            // Main content grid
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Ticket Status Panel
                <div class="bg-gray-800 rounded-lg border border-gray-700">
                    <div class="px-4 py-3 border-b border-gray-700 flex items-center justify-between">
                        <h3 class="text-lg font-medium text-white">{"Ticket Status"}</h3>
                        <a href="/tickets" class="text-sm text-blue-400 hover:text-blue-300">{"View all"}</a>
                    </div>
                    <div class="p-4">
                        <div class="space-y-4">
                            <TicketStatusRow label="Open" count={stats_data.tickets.open} color="blue" />
                            <TicketStatusRow label="In Progress" count={stats_data.tickets.in_progress} color="yellow" />
                            <TicketStatusRow label="Pending" count={stats_data.tickets.pending} color="gray" />
                            <TicketStatusRow label="Resolved Today" count={stats_data.tickets.resolved_today} color="green" />
                            <TicketStatusRow label="SLA Breached" count={stats_data.tickets.sla_breached} color="red" />
                        </div>
                    </div>
                </div>

                // Time Tracking Panel
                <div class="bg-gray-800 rounded-lg border border-gray-700">
                    <div class="px-4 py-3 border-b border-gray-700 flex items-center justify-between">
                        <h3 class="text-lg font-medium text-white">{"Time Tracking"}</h3>
                        <a href="/time" class="text-sm text-blue-400 hover:text-blue-300">{"View all"}</a>
                    </div>
                    <div class="p-4 space-y-4">
                        <div class="flex justify-between items-center">
                            <span class="text-gray-400">{"Hours Today"}</span>
                            <span class="text-xl font-semibold text-white">{format!("{:.1}h", stats_data.time.hours_today)}</span>
                        </div>
                        <div class="flex justify-between items-center">
                            <span class="text-gray-400">{"Billable Today"}</span>
                            <span class="text-xl font-semibold text-green-400">{format!("{:.1}h", stats_data.time.billable_hours_today)}</span>
                        </div>
                        <div class="flex justify-between items-center">
                            <span class="text-gray-400">{"This Week"}</span>
                            <span class="text-xl font-semibold text-white">{format!("{:.1}h", stats_data.time.hours_this_week)}</span>
                        </div>
                        <div class="flex justify-between items-center">
                            <span class="text-gray-400">{"Active Timers"}</span>
                            <span class="text-xl font-semibold text-blue-400">{stats_data.time.active_timers}</span>
                        </div>
                        if let Some(util) = stats_data.time.team_utilization {
                            <div class="pt-2 border-t border-gray-700">
                                <div class="flex justify-between items-center mb-1">
                                    <span class="text-gray-400 text-sm">{"Team Utilization"}</span>
                                    <span class="text-sm text-white">{format!("{:.0}%", util * 100.0)}</span>
                                </div>
                                <div class="w-full bg-gray-700 rounded-full h-2">
                                    <div
                                        class="bg-blue-500 h-2 rounded-full"
                                        style={format!("width: {}%", util * 100.0)}
                                    ></div>
                                </div>
                            </div>
                        }
                    </div>
                </div>

                // Invoicing Panel
                <div class="bg-gray-800 rounded-lg border border-gray-700">
                    <div class="px-4 py-3 border-b border-gray-700 flex items-center justify-between">
                        <h3 class="text-lg font-medium text-white">{"Invoicing"}</h3>
                        <a href="/invoices" class="text-sm text-blue-400 hover:text-blue-300">{"View all"}</a>
                    </div>
                    <div class="p-4 space-y-4">
                        <div class="flex justify-between items-center">
                            <span class="text-gray-400">{"Outstanding"}</span>
                            <span class="text-xl font-semibold text-white">{format!("${:.0}", stats_data.invoices.outstanding_amount)}</span>
                        </div>
                        <div class="flex justify-between items-center">
                            <span class="text-gray-400">{"Overdue"}</span>
                            <span class="text-xl font-semibold text-red-400">{format!("${:.0}", stats_data.invoices.overdue_amount)}</span>
                        </div>
                        <div class="flex justify-between items-center">
                            <span class="text-gray-400">{"Draft Invoices"}</span>
                            <span class="text-xl font-semibold text-gray-300">{stats_data.invoices.draft_count}</span>
                        </div>
                        <div class="flex justify-between items-center">
                            <span class="text-gray-400">{"Paid This Month"}</span>
                            <span class="text-xl font-semibold text-green-400">{format!("${:.0}", stats_data.invoices.paid_this_month)}</span>
                        </div>
                        if let Some(ratio) = stats_data.invoices.collection_ratio {
                            <div class="pt-2 border-t border-gray-700">
                                <div class="flex justify-between items-center mb-1">
                                    <span class="text-gray-400 text-sm">{"Collection Rate"}</span>
                                    <span class="text-sm text-white">{format!("{:.0}%", ratio * 100.0)}</span>
                                </div>
                                <div class="w-full bg-gray-700 rounded-full h-2">
                                    <div
                                        class={format!("h-2 rounded-full {}", if ratio >= 0.9 { "bg-green-500" } else if ratio >= 0.7 { "bg-yellow-500" } else { "bg-red-500" })}
                                        style={format!("width: {}%", ratio * 100.0)}
                                    ></div>
                                </div>
                            </div>
                        }
                    </div>
                </div>
            </div>

            // Bottom row - Assets and Top Clients
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Assets Overview
                <div class="bg-gray-800 rounded-lg border border-gray-700">
                    <div class="px-4 py-3 border-b border-gray-700 flex items-center justify-between">
                        <h3 class="text-lg font-medium text-white">{"Assets Overview"}</h3>
                        <a href="/assets" class="text-sm text-blue-400 hover:text-blue-300">{"View all"}</a>
                    </div>
                    <div class="p-4">
                        <div class="grid grid-cols-2 gap-4">
                            <div class="bg-gray-900 rounded-lg p-4 text-center">
                                <div class="text-3xl font-bold text-white">{stats_data.assets.total_assets}</div>
                                <div class="text-sm text-gray-400">{"Total Assets"}</div>
                            </div>
                            <div class="bg-gray-900 rounded-lg p-4 text-center">
                                <div class="text-3xl font-bold text-red-400">{stats_data.assets.critical_alerts}</div>
                                <div class="text-sm text-gray-400">{"Critical Alerts"}</div>
                            </div>
                            <div class="bg-gray-900 rounded-lg p-4 text-center">
                                <div class="text-3xl font-bold text-yellow-400">{stats_data.assets.warranty_expiring}</div>
                                <div class="text-sm text-gray-400">{"Warranty Expiring"}</div>
                            </div>
                            <div class="bg-gray-900 rounded-lg p-4 text-center">
                                <div class="text-3xl font-bold text-green-400">
                                    {stats_data.assets.online_percentage.map(|p| format!("{:.0}%", p * 100.0)).unwrap_or_else(|| "N/A".to_string())}
                                </div>
                                <div class="text-sm text-gray-400">{"Online"}</div>
                            </div>
                        </div>
                    </div>
                </div>

                // Top Clients
                <div class="bg-gray-800 rounded-lg border border-gray-700">
                    <div class="px-4 py-3 border-b border-gray-700 flex items-center justify-between">
                        <h3 class="text-lg font-medium text-white">{"Top Clients by Revenue"}</h3>
                        <a href="/clients" class="text-sm text-blue-400 hover:text-blue-300">{"View all"}</a>
                    </div>
                    <div class="p-4">
                        if stats_data.clients.top_clients_by_revenue.is_empty() {
                            <div class="text-gray-400 text-center py-4">{"No client data available"}</div>
                        } else {
                            <div class="space-y-3">
                                {for stats_data.clients.top_clients_by_revenue.iter().enumerate().map(|(i, client)| {
                                    html! {
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center space-x-3">
                                                <span class="text-gray-500 text-sm w-4">{i + 1}{"."}</span>
                                                <span class="text-white">{&client.name}</span>
                                            </div>
                                            <span class="text-green-400 font-medium">{format!("${:.0}", client.revenue)}</span>
                                        </div>
                                    }
                                })}
                            </div>
                        }
                        <div class="mt-4 pt-4 border-t border-gray-700 flex justify-between">
                            <span class="text-gray-400">{"New This Month"}</span>
                            <span class="text-green-400 font-medium">{format!("+{}", stats_data.clients.new_this_month)}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ===== HELPER COMPONENTS =====

#[derive(Properties, PartialEq)]
pub struct MetricCardProps {
    pub title: String,
    pub value: String,
    pub icon: String,
    pub color: String,
    #[prop_or_default]
    pub trend: Option<(f64, bool)>, // (percentage, is_positive)
}

#[function_component(MetricCard)]
pub fn metric_card(props: &MetricCardProps) -> Html {
    let (bg_color, text_color) = match props.color.as_str() {
        "blue" => ("bg-blue-600", "text-blue-400"),
        "green" => ("bg-green-600", "text-green-400"),
        "yellow" => ("bg-yellow-600", "text-yellow-400"),
        "orange" => ("bg-orange-600", "text-orange-400"),
        "red" => ("bg-red-600", "text-red-400"),
        _ => ("bg-gray-600", "text-gray-400"),
    };

    let icon = match props.icon.as_str() {
        "users" => html! {
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"/>
            </svg>
        },
        "ticket" => html! {
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z"/>
            </svg>
        },
        "dollar" => html! {
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
        },
        "clock" => html! {
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
        },
        "alert" => html! {
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
            </svg>
        },
        _ => html! { <span class="w-6 h-6"></span> },
    };

    html! {
        <div class="bg-gray-800 rounded-lg border border-gray-700 p-4">
            <div class="flex items-center justify-between">
                <div class={format!("p-2 rounded-lg {}", bg_color)}>
                    <div class="text-white">
                        {icon}
                    </div>
                </div>
                if let Some((pct, positive)) = props.trend {
                    <div class={format!("flex items-center text-sm {}", if positive { "text-green-400" } else { "text-red-400" })}>
                        if positive {
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"/>
                            </svg>
                        } else {
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"/>
                            </svg>
                        }
                        <span class="ml-1">{format!("{:.1}%", pct)}</span>
                    </div>
                }
            </div>
            <div class="mt-3">
                <p class="text-2xl font-bold text-white">{&props.value}</p>
                <p class={format!("text-sm {}", text_color)}>{&props.title}</p>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TicketStatusRowProps {
    pub label: &'static str,
    pub count: i64,
    pub color: &'static str,
}

#[function_component(TicketStatusRow)]
pub fn ticket_status_row(props: &TicketStatusRowProps) -> Html {
    let dot_color = match props.color {
        "blue" => "bg-blue-500",
        "yellow" => "bg-yellow-500",
        "green" => "bg-green-500",
        "red" => "bg-red-500",
        "gray" => "bg-gray-500",
        _ => "bg-gray-500",
    };

    let text_color = match props.color {
        "red" => "text-red-400",
        _ => "text-white",
    };

    html! {
        <div class="flex items-center justify-between">
            <div class="flex items-center space-x-2">
                <span class={format!("w-2 h-2 rounded-full {}", dot_color)}></span>
                <span class="text-gray-300">{props.label}</span>
            </div>
            <span class={format!("font-semibold {}", text_color)}>{props.count}</span>
        </div>
    }
}
