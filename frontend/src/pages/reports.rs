// Reports/Analytics Page

use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum ReportType {
    Overview,
    Utilization,
    Profitability,
    SlaCompliance,
    TicketMetrics,
}

#[function_component(ReportsPage)]
pub fn reports_page() -> Html {
    let active_report = use_state(|| ReportType::Overview);
    let date_range = use_state(|| "last_30_days".to_string());

    let set_report = |report: ReportType| {
        let active_report = active_report.clone();
        Callback::from(move |_| active_report.set(report))
    };

    let report_class = |report: ReportType| -> String {
        let base = "px-4 py-2 text-sm font-medium rounded-lg transition-colors";
        if *active_report == report {
            format!("{} bg-blue-600 text-white", base)
        } else {
            format!("{} text-gray-400 hover:text-white hover:bg-gray-700", base)
        }
    };

    html! {
        <div class="p-6" style="background-color: var(--bg-primary); min-height: 100vh;">
            // Header
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold" style="color: var(--fg-primary);">{"Reports & Analytics"}</h1>
                    <p class="mt-1" style="color: var(--fg-muted);">{"Business intelligence and performance metrics"}</p>
                </div>
                <div class="flex items-center space-x-3">
                    // Date Range Selector
                    <select
                        class="px-4 py-2 rounded-lg text-sm"
                        style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                    >
                        <option value="last_7_days">{"Last 7 Days"}</option>
                        <option value="last_30_days" selected=true>{"Last 30 Days"}</option>
                        <option value="last_90_days">{"Last 90 Days"}</option>
                        <option value="this_year">{"This Year"}</option>
                        <option value="custom">{"Custom Range"}</option>
                    </select>

                    <button
                        class="flex items-center space-x-2 px-4 py-2 rounded-lg font-medium"
                        style="background-color: var(--button-secondary-bg); color: var(--fg-secondary);"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/>
                        </svg>
                        <span>{"Export"}</span>
                    </button>
                </div>
            </div>

            // Report Type Tabs
            <div class="flex items-center space-x-2 mb-6 overflow-x-auto pb-2">
                <button onclick={set_report(ReportType::Overview)} class={report_class(ReportType::Overview)}>
                    {"Executive Overview"}
                </button>
                <button onclick={set_report(ReportType::Utilization)} class={report_class(ReportType::Utilization)}>
                    {"Technician Utilization"}
                </button>
                <button onclick={set_report(ReportType::Profitability)} class={report_class(ReportType::Profitability)}>
                    {"Client Profitability"}
                </button>
                <button onclick={set_report(ReportType::SlaCompliance)} class={report_class(ReportType::SlaCompliance)}>
                    {"SLA Compliance"}
                </button>
                <button onclick={set_report(ReportType::TicketMetrics)} class={report_class(ReportType::TicketMetrics)}>
                    {"Ticket Metrics"}
                </button>
            </div>

            // Report Content
            {match *active_report {
                ReportType::Overview => html! { <OverviewReport /> },
                ReportType::Utilization => html! { <UtilizationReport /> },
                ReportType::Profitability => html! { <ProfitabilityReport /> },
                ReportType::SlaCompliance => html! { <SlaComplianceReport /> },
                ReportType::TicketMetrics => html! { <TicketMetricsReport /> },
            }}
        </div>
    }
}

// ===== Overview Report =====

#[function_component(OverviewReport)]
fn overview_report() -> Html {
    html! {
        <div class="space-y-6">
            // KPI Cards
            <div class="grid grid-cols-4 gap-4">
                <KpiCard
                    title="Total Revenue"
                    value="$48,250"
                    change="+12.5%"
                    positive={true}
                    icon="dollar"
                />
                <KpiCard
                    title="Active Tickets"
                    value="42"
                    change="-8%"
                    positive={true}
                    icon="ticket"
                />
                <KpiCard
                    title="Avg Response Time"
                    value="2.4h"
                    change="-15%"
                    positive={true}
                    icon="clock"
                />
                <KpiCard
                    title="Customer Satisfaction"
                    value="4.8/5"
                    change="+0.2"
                    positive={true}
                    icon="star"
                />
            </div>

            // Charts Row
            <div class="grid grid-cols-2 gap-6">
                // Revenue Chart Placeholder
                <div
                    class="rounded-lg p-6"
                    style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
                >
                    <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Revenue Trend"}</h3>
                    <div class="h-64 flex items-center justify-center" style="color: var(--fg-muted);">
                        <div class="text-center">
                            <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
                            </svg>
                            <p>{"Chart visualization would appear here"}</p>
                        </div>
                    </div>
                </div>

                // Ticket Distribution Placeholder
                <div
                    class="rounded-lg p-6"
                    style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
                >
                    <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Ticket Distribution"}</h3>
                    <div class="h-64 flex items-center justify-center" style="color: var(--fg-muted);">
                        <div class="text-center">
                            <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 3.055A9.001 9.001 0 1020.945 13H11V3.055z"/>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.488 9H15V3.512A9.025 9.025 0 0120.488 9z"/>
                            </svg>
                            <p>{"Pie chart would appear here"}</p>
                        </div>
                    </div>
                </div>
            </div>

            // Top Clients Table
            <div
                class="rounded-lg p-6"
                style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
            >
                <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Top Clients by Revenue"}</h3>
                <table class="w-full">
                    <thead>
                        <tr style="border-bottom: 1px solid var(--border-primary);">
                            <th class="text-left py-2 text-sm font-medium" style="color: var(--fg-muted);">{"Client"}</th>
                            <th class="text-right py-2 text-sm font-medium" style="color: var(--fg-muted);">{"Revenue"}</th>
                            <th class="text-right py-2 text-sm font-medium" style="color: var(--fg-muted);">{"Tickets"}</th>
                            <th class="text-right py-2 text-sm font-medium" style="color: var(--fg-muted);">{"Hours"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr style="border-bottom: 1px solid var(--border-primary);">
                            <td class="py-3" style="color: var(--fg-primary);">{"Acme Corp"}</td>
                            <td class="py-3 text-right font-mono" style="color: var(--color-success);">{"$12,450"}</td>
                            <td class="py-3 text-right" style="color: var(--fg-secondary);">{"45"}</td>
                            <td class="py-3 text-right" style="color: var(--fg-secondary);">{"124h"}</td>
                        </tr>
                        <tr style="border-bottom: 1px solid var(--border-primary);">
                            <td class="py-3" style="color: var(--fg-primary);">{"TechStart Inc"}</td>
                            <td class="py-3 text-right font-mono" style="color: var(--color-success);">{"$9,800"}</td>
                            <td class="py-3 text-right" style="color: var(--fg-secondary);">{"38"}</td>
                            <td class="py-3 text-right" style="color: var(--fg-secondary);">{"98h"}</td>
                        </tr>
                        <tr style="border-bottom: 1px solid var(--border-primary);">
                            <td class="py-3" style="color: var(--fg-primary);">{"Global Solutions"}</td>
                            <td class="py-3 text-right font-mono" style="color: var(--color-success);">{"$8,200"}</td>
                            <td class="py-3 text-right" style="color: var(--fg-secondary);">{"32"}</td>
                            <td class="py-3 text-right" style="color: var(--fg-secondary);">{"82h"}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

// ===== KPI Card Component =====

#[derive(Properties, PartialEq)]
struct KpiCardProps {
    title: &'static str,
    value: &'static str,
    change: &'static str,
    positive: bool,
    icon: &'static str,
}

#[function_component(KpiCard)]
fn kpi_card(props: &KpiCardProps) -> Html {
    let change_color = if props.positive { "var(--color-success)" } else { "var(--color-error)" };

    html! {
        <div
            class="rounded-lg p-4"
            style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
        >
            <div class="flex items-center justify-between mb-2">
                <span class="text-sm" style="color: var(--fg-muted);">{props.title}</span>
                <div class="w-8 h-8 rounded flex items-center justify-center" style="background-color: var(--bg-highlight);">
                    <svg class="w-4 h-4" style="color: var(--accent-primary);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                </div>
            </div>
            <div class="text-2xl font-bold font-mono" style="color: var(--fg-primary);">{props.value}</div>
            <div class="flex items-center space-x-1 mt-1">
                <span class="text-sm" style={format!("color: {}", change_color)}>{props.change}</span>
                <span class="text-xs" style="color: var(--fg-dimmed);">{"vs last period"}</span>
            </div>
        </div>
    }
}

// ===== Utilization Report =====

#[function_component(UtilizationReport)]
fn utilization_report() -> Html {
    let technicians = vec![
        ("John Doe", 85, 120, 102, 18),
        ("Jane Smith", 78, 110, 86, 24),
        ("Bob Wilson", 92, 130, 120, 10),
        ("Alice Brown", 65, 100, 65, 35),
    ];

    html! {
        <div
            class="rounded-lg p-6"
            style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
        >
            <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Technician Utilization"}</h3>
            <table class="w-full">
                <thead>
                    <tr style="border-bottom: 1px solid var(--border-primary);">
                        <th class="text-left py-3 text-sm font-medium" style="color: var(--fg-muted);">{"Technician"}</th>
                        <th class="text-center py-3 text-sm font-medium" style="color: var(--fg-muted);">{"Utilization"}</th>
                        <th class="text-right py-3 text-sm font-medium" style="color: var(--fg-muted);">{"Total Hours"}</th>
                        <th class="text-right py-3 text-sm font-medium" style="color: var(--fg-muted);">{"Billable"}</th>
                        <th class="text-right py-3 text-sm font-medium" style="color: var(--fg-muted);">{"Non-Billable"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for technicians.iter().map(|(name, util, total, billable, non_billable)| {
                        let util_color = if *util >= 80 {
                            "var(--color-success)"
                        } else if *util >= 60 {
                            "var(--color-warning)"
                        } else {
                            "var(--color-error)"
                        };

                        html! {
                            <tr style="border-bottom: 1px solid var(--border-primary);">
                                <td class="py-3" style="color: var(--fg-primary);">{name}</td>
                                <td class="py-3">
                                    <div class="flex items-center justify-center space-x-2">
                                        <div class="w-24 h-2 rounded-full overflow-hidden" style="background-color: var(--bg-highlight);">
                                            <div
                                                class="h-full rounded-full"
                                                style={format!("width: {}%; background-color: {}", util, util_color)}
                                            />
                                        </div>
                                        <span class="text-sm font-mono" style={format!("color: {}", util_color)}>
                                            {format!("{}%", util)}
                                        </span>
                                    </div>
                                </td>
                                <td class="py-3 text-right font-mono" style="color: var(--fg-secondary);">
                                    {format!("{}h", total)}
                                </td>
                                <td class="py-3 text-right font-mono" style="color: var(--color-success);">
                                    {format!("{}h", billable)}
                                </td>
                                <td class="py-3 text-right font-mono" style="color: var(--fg-muted);">
                                    {format!("{}h", non_billable)}
                                </td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        </div>
    }
}

// ===== Profitability Report =====

#[function_component(ProfitabilityReport)]
fn profitability_report() -> Html {
    html! {
        <div
            class="rounded-lg p-6"
            style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
        >
            <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Client Profitability"}</h3>
            <p style="color: var(--fg-muted);">{"Profitability analysis by client showing revenue, costs, and margins."}</p>
            <div class="mt-8 h-64 flex items-center justify-center" style="color: var(--fg-muted);">
                {"Profitability chart and data table would appear here"}
            </div>
        </div>
    }
}

// ===== SLA Compliance Report =====

#[function_component(SlaComplianceReport)]
fn sla_compliance_report() -> Html {
    html! {
        <div
            class="rounded-lg p-6"
            style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
        >
            <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"SLA Compliance"}</h3>
            <p style="color: var(--fg-muted);">{"SLA compliance metrics by priority level and client."}</p>
            <div class="mt-8 h-64 flex items-center justify-center" style="color: var(--fg-muted);">
                {"SLA compliance charts and metrics would appear here"}
            </div>
        </div>
    }
}

// ===== Ticket Metrics Report =====

#[function_component(TicketMetricsReport)]
fn ticket_metrics_report() -> Html {
    html! {
        <div
            class="rounded-lg p-6"
            style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
        >
            <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Ticket Metrics"}</h3>
            <p style="color: var(--fg-muted);">{"Ticket volume, resolution times, and trend analysis."}</p>
            <div class="mt-8 h-64 flex items-center justify-center" style="color: var(--fg-muted);">
                {"Ticket metrics charts would appear here"}
            </div>
        </div>
    }
}
