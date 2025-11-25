// Invoices & Billing Page

use yew::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Invoice {
    pub id: String,
    pub invoice_number: String,
    pub client_id: String,
    pub client_name: String,
    pub status: InvoiceStatus,
    pub issue_date: String,
    pub due_date: String,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub paid_amount: f64,
    pub balance_due: f64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum InvoiceStatus {
    Draft,
    Sent,
    Viewed,
    Paid,
    Overdue,
    Cancelled,
}

impl InvoiceStatus {
    fn as_str(&self) -> &'static str {
        match self {
            InvoiceStatus::Draft => "Draft",
            InvoiceStatus::Sent => "Sent",
            InvoiceStatus::Viewed => "Viewed",
            InvoiceStatus::Paid => "Paid",
            InvoiceStatus::Overdue => "Overdue",
            InvoiceStatus::Cancelled => "Cancelled",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            InvoiceStatus::Draft => "var(--fg-muted)",
            InvoiceStatus::Sent => "var(--accent-primary)",
            InvoiceStatus::Viewed => "var(--color-warning)",
            InvoiceStatus::Paid => "var(--color-success)",
            InvoiceStatus::Overdue => "var(--color-error)",
            InvoiceStatus::Cancelled => "var(--fg-dimmed)",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum InvoiceTab {
    All,
    Draft,
    Outstanding,
    Paid,
}

#[function_component(InvoicesPage)]
pub fn invoices_page() -> Html {
    let invoices = use_state(|| None::<Vec<Invoice>>);
    let active_tab = use_state(|| InvoiceTab::All);
    let loading = use_state(|| true);
    let search_query = use_state(|| String::new());

    // Fetch invoices on mount
    {
        let invoices = invoices.clone();
        let loading = loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let mock_invoices = vec![
                    Invoice {
                        id: "1".to_string(),
                        invoice_number: "INV-2024-001".to_string(),
                        client_id: "c1".to_string(),
                        client_name: "Acme Corp".to_string(),
                        status: InvoiceStatus::Paid,
                        issue_date: "2024-01-15".to_string(),
                        due_date: "2024-02-15".to_string(),
                        subtotal: 2500.00,
                        tax: 200.00,
                        total: 2700.00,
                        paid_amount: 2700.00,
                        balance_due: 0.00,
                    },
                    Invoice {
                        id: "2".to_string(),
                        invoice_number: "INV-2024-002".to_string(),
                        client_id: "c2".to_string(),
                        client_name: "TechStart Inc".to_string(),
                        status: InvoiceStatus::Sent,
                        issue_date: "2024-02-01".to_string(),
                        due_date: "2024-03-01".to_string(),
                        subtotal: 1800.00,
                        tax: 144.00,
                        total: 1944.00,
                        paid_amount: 0.00,
                        balance_due: 1944.00,
                    },
                    Invoice {
                        id: "3".to_string(),
                        invoice_number: "INV-2024-003".to_string(),
                        client_id: "c1".to_string(),
                        client_name: "Acme Corp".to_string(),
                        status: InvoiceStatus::Overdue,
                        issue_date: "2024-01-01".to_string(),
                        due_date: "2024-01-31".to_string(),
                        subtotal: 3200.00,
                        tax: 256.00,
                        total: 3456.00,
                        paid_amount: 1000.00,
                        balance_due: 2456.00,
                    },
                    Invoice {
                        id: "4".to_string(),
                        invoice_number: "INV-2024-004".to_string(),
                        client_id: "c3".to_string(),
                        client_name: "Global Solutions".to_string(),
                        status: InvoiceStatus::Draft,
                        issue_date: "2024-02-20".to_string(),
                        due_date: "2024-03-20".to_string(),
                        subtotal: 4500.00,
                        tax: 360.00,
                        total: 4860.00,
                        paid_amount: 0.00,
                        balance_due: 4860.00,
                    },
                ];

                invoices.set(Some(mock_invoices));
                loading.set(false);
            });
            || ()
        });
    }

    // Calculate summary stats
    let stats = invoices.as_ref().map(|list| {
        let total_outstanding = list.iter()
            .filter(|i| matches!(i.status, InvoiceStatus::Sent | InvoiceStatus::Viewed | InvoiceStatus::Overdue))
            .map(|i| i.balance_due)
            .sum::<f64>();

        let total_overdue = list.iter()
            .filter(|i| matches!(i.status, InvoiceStatus::Overdue))
            .map(|i| i.balance_due)
            .sum::<f64>();

        let total_paid = list.iter()
            .filter(|i| matches!(i.status, InvoiceStatus::Paid))
            .map(|i| i.total)
            .sum::<f64>();

        let draft_count = list.iter()
            .filter(|i| matches!(i.status, InvoiceStatus::Draft))
            .count();

        (total_outstanding, total_overdue, total_paid, draft_count)
    }).unwrap_or((0.0, 0.0, 0.0, 0));

    let set_tab = |tab: InvoiceTab| {
        let active_tab = active_tab.clone();
        Callback::from(move |_| active_tab.set(tab))
    };

    let tab_class = |tab: InvoiceTab| -> String {
        let base = "px-4 py-2 text-sm font-medium rounded-lg transition-colors";
        if *active_tab == tab {
            format!("{} bg-blue-600 text-white", base)
        } else {
            format!("{} text-gray-400 hover:text-white hover:bg-gray-700", base)
        }
    };

    // Filter invoices by tab
    let filtered_invoices = invoices.as_ref().map(|list| {
        list.iter()
            .filter(|i| match *active_tab {
                InvoiceTab::All => true,
                InvoiceTab::Draft => matches!(i.status, InvoiceStatus::Draft),
                InvoiceTab::Outstanding => matches!(i.status, InvoiceStatus::Sent | InvoiceStatus::Viewed | InvoiceStatus::Overdue),
                InvoiceTab::Paid => matches!(i.status, InvoiceStatus::Paid),
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    html! {
        <div class="p-6" style="background-color: var(--bg-primary); min-height: 100vh;">
            // Header
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold" style="color: var(--fg-primary);">{"Invoices"}</h1>
                    <p class="mt-1" style="color: var(--fg-muted);">{"Manage billing and payments"}</p>
                </div>
                <div class="flex items-center space-x-3">
                    <button
                        class="flex items-center space-x-2 px-4 py-2 rounded-lg font-medium"
                        style="background-color: var(--button-secondary-bg); color: var(--fg-secondary);"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/>
                        </svg>
                        <span>{"Export"}</span>
                    </button>
                    <button
                        class="flex items-center space-x-2 px-4 py-2 rounded-lg font-medium"
                        style="background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                        </svg>
                        <span>{"New Invoice"}</span>
                    </button>
                </div>
            </div>

            // Summary Cards
            <div class="grid grid-cols-4 gap-4 mb-6">
                <SummaryCard
                    title="Outstanding"
                    value={format!("${:.2}", stats.0)}
                    subtitle="Awaiting payment"
                    color="var(--accent-primary)"
                />
                <SummaryCard
                    title="Overdue"
                    value={format!("${:.2}", stats.1)}
                    subtitle="Past due date"
                    color="var(--color-error)"
                />
                <SummaryCard
                    title="Paid (YTD)"
                    value={format!("${:.2}", stats.2)}
                    subtitle="This year"
                    color="var(--color-success)"
                />
                <SummaryCard
                    title="Drafts"
                    value={stats.3.to_string()}
                    subtitle="Not sent"
                    color="var(--fg-muted)"
                />
            </div>

            // Tabs
            <div class="flex items-center space-x-2 mb-6">
                <button onclick={set_tab(InvoiceTab::All)} class={tab_class(InvoiceTab::All)}>{"All"}</button>
                <button onclick={set_tab(InvoiceTab::Draft)} class={tab_class(InvoiceTab::Draft)}>{"Drafts"}</button>
                <button onclick={set_tab(InvoiceTab::Outstanding)} class={tab_class(InvoiceTab::Outstanding)}>{"Outstanding"}</button>
                <button onclick={set_tab(InvoiceTab::Paid)} class={tab_class(InvoiceTab::Paid)}>{"Paid"}</button>
            </div>

            // Invoice Table
            if *loading {
                <div class="text-center py-12" style="color: var(--fg-muted);">
                    {"Loading invoices..."}
                </div>
            } else if let Some(invoices) = &filtered_invoices {
                <div class="rounded-lg overflow-hidden" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <table class="w-full">
                        <thead>
                            <tr style="background-color: var(--bg-tertiary);">
                                <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Invoice #"}</th>
                                <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Client"}</th>
                                <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Status"}</th>
                                <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Issue Date"}</th>
                                <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Due Date"}</th>
                                <th class="text-right py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Total"}</th>
                                <th class="text-right py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Balance"}</th>
                                <th class="py-3 px-4"></th>
                            </tr>
                        </thead>
                        <tbody>
                            { for invoices.iter().map(|invoice| {
                                html! {
                                    <tr class="hover:bg-gray-700/30" style="border-bottom: 1px solid var(--border-primary);">
                                        <td class="py-3 px-4">
                                            <span class="font-mono font-medium" style="color: var(--accent-primary);">
                                                {&invoice.invoice_number}
                                            </span>
                                        </td>
                                        <td class="py-3 px-4" style="color: var(--fg-primary);">
                                            {&invoice.client_name}
                                        </td>
                                        <td class="py-3 px-4">
                                            <span
                                                class="px-2 py-1 text-xs rounded font-medium"
                                                style={format!("background-color: {}20; color: {}", invoice.status.color(), invoice.status.color())}
                                            >
                                                {invoice.status.as_str()}
                                            </span>
                                        </td>
                                        <td class="py-3 px-4 text-sm" style="color: var(--fg-secondary);">
                                            {&invoice.issue_date}
                                        </td>
                                        <td class="py-3 px-4 text-sm" style="color: var(--fg-secondary);">
                                            {&invoice.due_date}
                                        </td>
                                        <td class="py-3 px-4 text-right font-mono" style="color: var(--fg-primary);">
                                            {format!("${:.2}", invoice.total)}
                                        </td>
                                        <td class="py-3 px-4 text-right font-mono" style={if invoice.balance_due > 0.0 { "color: var(--color-warning);" } else { "color: var(--color-success);" }}>
                                            {format!("${:.2}", invoice.balance_due)}
                                        </td>
                                        <td class="py-3 px-4">
                                            <div class="flex items-center space-x-2">
                                                <button class="p-1 rounded hover:bg-gray-700" style="color: var(--fg-muted);" title="View">
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
                                                    </svg>
                                                </button>
                                                <button class="p-1 rounded hover:bg-gray-700" style="color: var(--fg-muted);" title="Download">
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
                                                    </svg>
                                                </button>
                                                <button class="p-1 rounded hover:bg-gray-700" style="color: var(--fg-muted);" title="More">
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"/>
                                                    </svg>
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                </div>
            } else {
                <div class="text-center py-12" style="color: var(--fg-muted);">
                    {"No invoices found"}
                </div>
            }
        </div>
    }
}

// ===== Summary Card Component =====

#[derive(Properties, PartialEq)]
struct SummaryCardProps {
    title: &'static str,
    value: String,
    subtitle: &'static str,
    color: &'static str,
}

#[function_component(SummaryCard)]
fn summary_card(props: &SummaryCardProps) -> Html {
    html! {
        <div
            class="rounded-lg p-4"
            style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
        >
            <div class="text-sm mb-1" style="color: var(--fg-muted);">{props.title}</div>
            <div class="text-2xl font-bold font-mono" style={format!("color: {}", props.color)}>
                {&props.value}
            </div>
            <div class="text-xs mt-1" style="color: var(--fg-dimmed);">{props.subtitle}</div>
        </div>
    }
}
