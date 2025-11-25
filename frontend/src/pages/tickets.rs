use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::{tickets, PaginatedResponse};
use crate::components::layout::Route;

#[function_component(TicketsPage)]
pub fn tickets_page() -> Html {
    let tickets_data = use_state(|| None::<PaginatedResponse<tickets::Ticket>>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let current_page = use_state(|| 1u32);
    let filter_status = use_state(|| None::<String>);
    let filter_priority = use_state(|| None::<String>);
    let show_create_modal = use_state(|| false);

    // Load tickets
    {
        let tickets_data = tickets_data.clone();
        let loading = loading.clone();
        let error = error.clone();
        let page = *current_page;
        let status = (*filter_status).clone();
        let priority = (*filter_priority).clone();

        use_effect_with(
            (page, status.clone(), priority.clone()),
            move |_| {
                loading.set(true);
                spawn_local(async move {
                    match tickets::list(page, 25, status.as_deref(), None).await {
                        Ok(data) => {
                            tickets_data.set(Some(data));
                            loading.set(false);
                        }
                        Err(e) => {
                            error.set(Some(e.message));
                            loading.set(false);
                        }
                    }
                });
                || ()
            },
        );
    }

    let on_status_change = {
        let filter_status = filter_status.clone();
        let current_page = current_page.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = input.value();
            filter_status.set(if value == "all" { None } else { Some(value) });
            current_page.set(1);
        })
    };

    let on_priority_change = {
        let filter_priority = filter_priority.clone();
        let current_page = current_page.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = input.value();
            filter_priority.set(if value == "all" { None } else { Some(value) });
            current_page.set(1);
        })
    };

    let toggle_create_modal = {
        let show_create_modal = show_create_modal.clone();
        Callback::from(move |_| show_create_modal.set(!*show_create_modal))
    };

    html! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white">{"Tickets"}</h1>
                    <p class="text-gray-400">{"Manage support tickets and track SLAs"}</p>
                </div>
                <button
                    onclick={toggle_create_modal}
                    class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg flex items-center space-x-2"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    <span>{"New Ticket"}</span>
                </button>
            </div>

            // Quick Stats
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                <QuickStat label="Open" count={count_by_status(&tickets_data, "open")} color="blue" />
                <QuickStat label="In Progress" count={count_by_status(&tickets_data, "in_progress")} color="yellow" />
                <QuickStat label="Pending" count={count_by_status(&tickets_data, "pending")} color="gray" />
                <QuickStat label="Resolved Today" count={0} color="green" />
            </div>

            // Filters
            <div class="bg-gray-800 rounded-lg border border-gray-700 p-4">
                <div class="flex flex-wrap items-center gap-4">
                    // Status filter
                    <div class="flex items-center space-x-2">
                        <label class="text-gray-400 text-sm">{"Status:"}</label>
                        <select
                            onchange={on_status_change}
                            class="bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 text-sm focus:ring-blue-500 focus:border-blue-500"
                        >
                            <option value="all" selected={filter_status.is_none()}>{"All"}</option>
                            <option value="open">{"Open"}</option>
                            <option value="in_progress">{"In Progress"}</option>
                            <option value="pending">{"Pending"}</option>
                            <option value="resolved">{"Resolved"}</option>
                            <option value="closed">{"Closed"}</option>
                        </select>
                    </div>

                    // Priority filter
                    <div class="flex items-center space-x-2">
                        <label class="text-gray-400 text-sm">{"Priority:"}</label>
                        <select
                            onchange={on_priority_change}
                            class="bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 text-sm focus:ring-blue-500 focus:border-blue-500"
                        >
                            <option value="all" selected={filter_priority.is_none()}>{"All"}</option>
                            <option value="critical">{"Critical"}</option>
                            <option value="high">{"High"}</option>
                            <option value="medium">{"Medium"}</option>
                            <option value="low">{"Low"}</option>
                        </select>
                    </div>

                    // Search
                    <div class="flex-1 min-w-64">
                        <div class="relative">
                            <svg class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                            </svg>
                            <input
                                type="text"
                                placeholder="Search tickets..."
                                class="w-full bg-gray-700 border border-gray-600 text-white rounded-lg pl-10 pr-4 py-2 text-sm focus:ring-blue-500 focus:border-blue-500"
                            />
                        </div>
                    </div>
                </div>
            </div>

            // Tickets Table
            <div class="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
                if *loading {
                    <div class="flex justify-center items-center h-64">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                    </div>
                } else if let Some(ref err) = *error {
                    <div class="p-6 text-red-400">{"Error: "}{err}</div>
                } else if let Some(ref data) = *tickets_data {
                    if data.data.is_empty() {
                        <div class="p-12 text-center">
                            <svg class="mx-auto h-12 w-12 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z"/>
                            </svg>
                            <h3 class="mt-4 text-lg font-medium text-white">{"No tickets found"}</h3>
                            <p class="mt-2 text-gray-400">{"Try adjusting your filters or create a new ticket."}</p>
                        </div>
                    } else {
                        <div class="overflow-x-auto">
                            <table class="min-w-full divide-y divide-gray-700">
                                <thead class="bg-gray-900">
                                    <tr>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">{"#"}</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">{"Subject"}</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">{"Client"}</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">{"Status"}</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">{"Priority"}</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">{"Assigned"}</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">{"SLA"}</th>
                                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">{"Actions"}</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-700">
                                    {for data.data.iter().map(|ticket| {
                                        html! {
                                            <TicketRow ticket={ticket.clone()} />
                                        }
                                    })}
                                </tbody>
                            </table>
                        </div>

                        // Pagination
                        <div class="px-4 py-3 border-t border-gray-700 flex items-center justify-between">
                            <div class="text-sm text-gray-400">
                                {"Showing "}{((data.meta.page - 1) * data.meta.per_page) + 1}{" to "}{std::cmp::min(data.meta.page * data.meta.per_page, data.meta.total as u32)}{" of "}{data.meta.total}{" tickets"}
                            </div>
                            <div class="flex space-x-2">
                                <button
                                    onclick={
                                        let current_page = current_page.clone();
                                        Callback::from(move |_| {
                                            if *current_page > 1 {
                                                current_page.set(*current_page - 1);
                                            }
                                        })
                                    }
                                    disabled={*current_page <= 1}
                                    class="px-3 py-1 bg-gray-700 text-white rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-600"
                                >
                                    {"Previous"}
                                </button>
                                <span class="px-3 py-1 text-gray-400">
                                    {"Page "}{*current_page}{" of "}{data.meta.total_pages}
                                </span>
                                <button
                                    onclick={
                                        let current_page = current_page.clone();
                                        let total_pages = data.meta.total_pages;
                                        Callback::from(move |_| {
                                            if *current_page < total_pages {
                                                current_page.set(*current_page + 1);
                                            }
                                        })
                                    }
                                    disabled={*current_page >= data.meta.total_pages}
                                    class="px-3 py-1 bg-gray-700 text-white rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-600"
                                >
                                    {"Next"}
                                </button>
                            </div>
                        </div>
                    }
                } else {
                    <div class="p-6 text-gray-400">{"No data"}</div>
                }
            </div>

            // Create Ticket Modal
            if *show_create_modal {
                <CreateTicketModal on_close={toggle_create_modal.clone()} />
            }
        </div>
    }
}

fn count_by_status(data: &UseStateHandle<Option<PaginatedResponse<tickets::Ticket>>>, status: &str) -> i64 {
    match &**data {
        Some(d) => d.data.iter().filter(|t| t.status == status).count() as i64,
        None => 0,
    }
}

// Quick Stat Component
#[derive(Properties, PartialEq)]
struct QuickStatProps {
    label: &'static str,
    count: i64,
    color: &'static str,
}

#[function_component(QuickStat)]
fn quick_stat(props: &QuickStatProps) -> Html {
    let (bg, text) = match props.color {
        "blue" => ("bg-blue-600/20", "text-blue-400"),
        "yellow" => ("bg-yellow-600/20", "text-yellow-400"),
        "green" => ("bg-green-600/20", "text-green-400"),
        "red" => ("bg-red-600/20", "text-red-400"),
        _ => ("bg-gray-600/20", "text-gray-400"),
    };

    html! {
        <div class={format!("rounded-lg p-4 {}", bg)}>
            <div class={format!("text-2xl font-bold {}", text)}>{props.count}</div>
            <div class="text-gray-400 text-sm">{props.label}</div>
        </div>
    }
}

// Ticket Row Component
#[derive(Properties, PartialEq)]
struct TicketRowProps {
    ticket: tickets::Ticket,
}

#[function_component(TicketRow)]
fn ticket_row(props: &TicketRowProps) -> Html {
    let ticket = &props.ticket;

    let status_badge = match ticket.status.as_str() {
        "open" => ("bg-blue-600/20 text-blue-400", "Open"),
        "in_progress" => ("bg-yellow-600/20 text-yellow-400", "In Progress"),
        "pending" => ("bg-gray-600/20 text-gray-400", "Pending"),
        "resolved" => ("bg-green-600/20 text-green-400", "Resolved"),
        "closed" => ("bg-gray-700 text-gray-500", "Closed"),
        _ => ("bg-gray-600/20 text-gray-400", &ticket.status),
    };

    let priority_badge = match ticket.priority.as_str() {
        "critical" => ("bg-red-600/20 text-red-400 border border-red-600", "Critical"),
        "high" => ("bg-orange-600/20 text-orange-400", "High"),
        "medium" => ("bg-yellow-600/20 text-yellow-400", "Medium"),
        "low" => ("bg-green-600/20 text-green-400", "Low"),
        _ => ("bg-gray-600/20 text-gray-400", &ticket.priority),
    };

    // Parse SLA due date and check if breached
    let sla_status = if let Some(ref due) = ticket.sla_resolution_due {
        // Simple check - in real app would compare to current time
        ("text-green-400", "On Track")
    } else {
        ("text-gray-500", "-")
    };

    html! {
        <tr class="hover:bg-gray-700/50 transition-colors">
            <td class="px-4 py-3">
                <span class="text-blue-400 font-medium">{"#"}{ticket.number}</span>
            </td>
            <td class="px-4 py-3">
                <div class="text-white font-medium truncate max-w-xs">{&ticket.subject}</div>
                if let Some(ref queue) = ticket.queue_name {
                    <div class="text-gray-500 text-xs">{queue}</div>
                }
            </td>
            <td class="px-4 py-3">
                <span class="text-gray-300">{ticket.client_name.as_deref().unwrap_or("-")}</span>
            </td>
            <td class="px-4 py-3">
                <span class={format!("px-2 py-1 rounded text-xs font-medium {}", status_badge.0)}>
                    {status_badge.1}
                </span>
            </td>
            <td class="px-4 py-3">
                <span class={format!("px-2 py-1 rounded text-xs font-medium {}", priority_badge.0)}>
                    {priority_badge.1}
                </span>
            </td>
            <td class="px-4 py-3">
                <span class="text-gray-400">{ticket.assigned_to_name.as_deref().unwrap_or("Unassigned")}</span>
            </td>
            <td class="px-4 py-3">
                <span class={sla_status.0}>{sla_status.1}</span>
            </td>
            <td class="px-4 py-3 text-right">
                <div class="flex items-center justify-end space-x-2">
                    <Link<Route> to={Route::TicketDetail { id: ticket.id.clone() }}
                        classes="text-blue-400 hover:text-blue-300 text-sm"
                    >
                        {"View"}
                    </Link<Route>>
                    <button class="text-gray-400 hover:text-white">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"/>
                        </svg>
                    </button>
                </div>
            </td>
        </tr>
    }
}

// Create Ticket Modal
#[derive(Properties, PartialEq)]
struct CreateTicketModalProps {
    on_close: Callback<MouseEvent>,
}

#[function_component(CreateTicketModal)]
fn create_ticket_modal(props: &CreateTicketModalProps) -> Html {
    let subject = use_state(String::new);
    let description = use_state(String::new);
    let priority = use_state(|| "medium".to_string());
    let client_id = use_state(String::new);
    let submitting = use_state(|| false);

    let on_submit = {
        let subject = subject.clone();
        let description = description.clone();
        let priority = priority.clone();
        let client_id = client_id.clone();
        let submitting = submitting.clone();
        let on_close = props.on_close.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let subject_val = (*subject).clone();
            let description_val = (*description).clone();
            let priority_val = (*priority).clone();
            let client_id_val = (*client_id).clone();
            let submitting = submitting.clone();
            let on_close = on_close.clone();

            submitting.set(true);

            spawn_local(async move {
                let request = tickets::CreateTicketRequest {
                    subject: subject_val,
                    description: Some(description_val),
                    priority: priority_val,
                    client_id: client_id_val,
                    assigned_to: None,
                    queue_id: None,
                };

                match tickets::create(&request).await {
                    Ok(_) => {
                        // Trigger close and refresh
                        // In a real app, would also refresh the ticket list
                    }
                    Err(_) => {
                        // Handle error
                    }
                }
                submitting.set(false);
            });
        })
    };

    html! {
        <div class="fixed inset-0 z-50 overflow-y-auto">
            <div class="flex min-h-full items-center justify-center p-4">
                // Backdrop
                <div class="fixed inset-0 bg-black/50" onclick={props.on_close.clone()}></div>

                // Modal
                <div class="relative bg-gray-800 rounded-lg shadow-xl border border-gray-700 w-full max-w-lg">
                    <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
                        <h3 class="text-lg font-medium text-white">{"Create New Ticket"}</h3>
                        <button onclick={props.on_close.clone()} class="text-gray-400 hover:text-white">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>

                    <form onsubmit={on_submit}>
                        <div class="p-6 space-y-4">
                            // Subject
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-1">{"Subject"}</label>
                                <input
                                    type="text"
                                    required=true
                                    value={(*subject).clone()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        subject.set(input.value());
                                    })}
                                    class="w-full bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                                    placeholder="Brief description of the issue"
                                />
                            </div>

                            // Client (placeholder - would be a dropdown in real app)
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-1">{"Client"}</label>
                                <select class="w-full bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500">
                                    <option value="">{"Select a client..."}</option>
                                </select>
                            </div>

                            // Priority
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-1">{"Priority"}</label>
                                <select
                                    value={(*priority).clone()}
                                    onchange={Callback::from(move |e: Event| {
                                        let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                        priority.set(input.value());
                                    })}
                                    class="w-full bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                                >
                                    <option value="low">{"Low"}</option>
                                    <option value="medium" selected=true>{"Medium"}</option>
                                    <option value="high">{"High"}</option>
                                    <option value="critical">{"Critical"}</option>
                                </select>
                            </div>

                            // Description
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-1">{"Description"}</label>
                                <textarea
                                    rows="4"
                                    value={(*description).clone()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                        description.set(input.value());
                                    })}
                                    class="w-full bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                                    placeholder="Detailed description of the issue..."
                                ></textarea>
                            </div>
                        </div>

                        <div class="px-6 py-4 border-t border-gray-700 flex justify-end space-x-3">
                            <button
                                type="button"
                                onclick={props.on_close.clone()}
                                class="px-4 py-2 text-gray-300 hover:text-white"
                            >
                                {"Cancel"}
                            </button>
                            <button
                                type="submit"
                                disabled={*submitting}
                                class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50"
                            >
                                if *submitting {
                                    {"Creating..."}
                                } else {
                                    {"Create Ticket"}
                                }
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}
