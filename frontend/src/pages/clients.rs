// Clients Page - CRM-style client management with contacts

use yew::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
    pub industry: Option<String>,
    pub status: ClientStatus,
    pub contract_type: Option<String>,
    pub monthly_revenue: Option<f64>,
    pub notes: Option<String>,
    pub contacts: Vec<Contact>,
    pub created_at: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Contact {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub title: Option<String>,
    pub is_primary: bool,
    pub notes: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum ClientStatus {
    Active,
    Inactive,
    Prospect,
    Former,
}

impl ClientStatus {
    fn as_str(&self) -> &'static str {
        match self {
            ClientStatus::Active => "Active",
            ClientStatus::Inactive => "Inactive",
            ClientStatus::Prospect => "Prospect",
            ClientStatus::Former => "Former",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            ClientStatus::Active => "var(--color-success)",
            ClientStatus::Inactive => "var(--fg-muted)",
            ClientStatus::Prospect => "var(--color-warning)",
            ClientStatus::Former => "var(--fg-dimmed)",
        }
    }
}

#[function_component(ClientsPage)]
pub fn clients_page() -> Html {
    let clients = use_state(|| None::<Vec<Client>>);
    let selected_client = use_state(|| None::<Client>);
    let loading = use_state(|| true);
    let search_query = use_state(|| String::new());
    let filter_status = use_state(|| None::<ClientStatus>);

    // Fetch clients on mount
    {
        let clients = clients.clone();
        let loading = loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Mock data - replace with actual API call
                let mock_clients = vec![
                    Client {
                        id: "1".to_string(),
                        name: "Acme Corporation".to_string(),
                        email: Some("info@acme.com".to_string()),
                        phone: Some("(555) 123-4567".to_string()),
                        website: Some("https://acme.com".to_string()),
                        address: Some("123 Main Street".to_string()),
                        city: Some("New York".to_string()),
                        state: Some("NY".to_string()),
                        zip: Some("10001".to_string()),
                        country: Some("USA".to_string()),
                        industry: Some("Manufacturing".to_string()),
                        status: ClientStatus::Active,
                        contract_type: Some("Managed Services".to_string()),
                        monthly_revenue: Some(4500.00),
                        notes: Some("Long-term client since 2019".to_string()),
                        contacts: vec![
                            Contact {
                                id: "c1".to_string(),
                                first_name: "John".to_string(),
                                last_name: "Smith".to_string(),
                                email: Some("john.smith@acme.com".to_string()),
                                phone: Some("(555) 123-4568".to_string()),
                                mobile: Some("(555) 987-6543".to_string()),
                                title: Some("IT Director".to_string()),
                                is_primary: true,
                                notes: None,
                            },
                            Contact {
                                id: "c2".to_string(),
                                first_name: "Jane".to_string(),
                                last_name: "Doe".to_string(),
                                email: Some("jane.doe@acme.com".to_string()),
                                phone: Some("(555) 123-4569".to_string()),
                                mobile: None,
                                title: Some("Office Manager".to_string()),
                                is_primary: false,
                                notes: Some("Secondary contact for billing".to_string()),
                            },
                        ],
                        created_at: "2019-06-15".to_string(),
                    },
                    Client {
                        id: "2".to_string(),
                        name: "TechStart Inc".to_string(),
                        email: Some("hello@techstart.io".to_string()),
                        phone: Some("(555) 234-5678".to_string()),
                        website: Some("https://techstart.io".to_string()),
                        address: Some("456 Innovation Way".to_string()),
                        city: Some("San Francisco".to_string()),
                        state: Some("CA".to_string()),
                        zip: Some("94105".to_string()),
                        country: Some("USA".to_string()),
                        industry: Some("Technology".to_string()),
                        status: ClientStatus::Active,
                        contract_type: Some("Block Hours".to_string()),
                        monthly_revenue: Some(2800.00),
                        notes: None,
                        contacts: vec![
                            Contact {
                                id: "c3".to_string(),
                                first_name: "Mike".to_string(),
                                last_name: "Johnson".to_string(),
                                email: Some("mike@techstart.io".to_string()),
                                phone: Some("(555) 234-5679".to_string()),
                                mobile: Some("(555) 345-6789".to_string()),
                                title: Some("CEO".to_string()),
                                is_primary: true,
                                notes: None,
                            },
                        ],
                        created_at: "2022-03-20".to_string(),
                    },
                    Client {
                        id: "3".to_string(),
                        name: "Global Solutions LLC".to_string(),
                        email: Some("support@globalsol.com".to_string()),
                        phone: Some("(555) 345-6789".to_string()),
                        website: Some("https://globalsol.com".to_string()),
                        address: Some("789 Corporate Blvd".to_string()),
                        city: Some("Chicago".to_string()),
                        state: Some("IL".to_string()),
                        zip: Some("60601".to_string()),
                        country: Some("USA".to_string()),
                        industry: Some("Consulting".to_string()),
                        status: ClientStatus::Prospect,
                        contract_type: None,
                        monthly_revenue: None,
                        notes: Some("Interested in managed services package".to_string()),
                        contacts: vec![
                            Contact {
                                id: "c4".to_string(),
                                first_name: "Sarah".to_string(),
                                last_name: "Williams".to_string(),
                                email: Some("sarah.williams@globalsol.com".to_string()),
                                phone: Some("(555) 345-6780".to_string()),
                                mobile: None,
                                title: Some("Operations Manager".to_string()),
                                is_primary: true,
                                notes: None,
                            },
                        ],
                        created_at: "2024-01-10".to_string(),
                    },
                ];

                clients.set(Some(mock_clients));
                loading.set(false);
            });
            || ()
        });
    }

    let on_search = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value());
        })
    };

    let on_client_select = {
        let selected_client = selected_client.clone();
        Callback::from(move |client: Client| {
            selected_client.set(Some(client));
        })
    };

    // Filter clients
    let filtered_clients = clients.as_ref().map(|list| {
        let query = search_query.to_lowercase();
        list.iter()
            .filter(|c| {
                let status_match = filter_status.as_ref().map(|s| &c.status == s).unwrap_or(true);
                let search_match = query.is_empty()
                    || c.name.to_lowercase().contains(&query)
                    || c.email.as_ref().map(|e| e.to_lowercase().contains(&query)).unwrap_or(false)
                    || c.city.as_ref().map(|city| city.to_lowercase().contains(&query)).unwrap_or(false);
                status_match && search_match
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    html! {
        <div class="flex h-full" style="background-color: var(--bg-primary);">
            // Left Panel - Client List
            <div class="w-96 flex-shrink-0 border-r flex flex-col" style="border-color: var(--border-primary); background-color: var(--bg-secondary);">
                // Header
                <div class="p-4 border-b" style="border-color: var(--border-primary);">
                    <div class="flex items-center justify-between mb-4">
                        <h1 class="text-xl font-semibold" style="color: var(--fg-primary);">{"Clients"}</h1>
                        <button
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
                            placeholder="Search clients..."
                            oninput={on_search}
                            class="w-full pl-10 pr-4 py-2 rounded-lg text-sm"
                            style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                        />
                    </div>
                </div>

                // Stats Summary
                <div class="grid grid-cols-3 gap-2 p-4 border-b" style="border-color: var(--border-primary);">
                    <div class="text-center">
                        <div class="text-lg font-bold" style="color: var(--color-success);">
                            {filtered_clients.as_ref().map(|c| c.iter().filter(|cl| matches!(cl.status, ClientStatus::Active)).count()).unwrap_or(0)}
                        </div>
                        <div class="text-xs" style="color: var(--fg-muted);">{"Active"}</div>
                    </div>
                    <div class="text-center">
                        <div class="text-lg font-bold" style="color: var(--color-warning);">
                            {filtered_clients.as_ref().map(|c| c.iter().filter(|cl| matches!(cl.status, ClientStatus::Prospect)).count()).unwrap_or(0)}
                        </div>
                        <div class="text-xs" style="color: var(--fg-muted);">{"Prospects"}</div>
                    </div>
                    <div class="text-center">
                        <div class="text-lg font-bold" style="color: var(--accent-primary);">
                            {filtered_clients.as_ref().map(|c| c.len()).unwrap_or(0)}
                        </div>
                        <div class="text-xs" style="color: var(--fg-muted);">{"Total"}</div>
                    </div>
                </div>

                // Client List
                <div class="flex-1 overflow-y-auto">
                    if *loading {
                        <div class="p-4 text-center" style="color: var(--fg-muted);">
                            {"Loading clients..."}
                        </div>
                    } else if let Some(clients) = &filtered_clients {
                        { for clients.iter().map(|client| {
                            let c = client.clone();
                            let on_select = on_client_select.clone();
                            let is_selected = selected_client.as_ref().map(|s| s.id == client.id).unwrap_or(false);

                            html! {
                                <ClientListItem
                                    client={c.clone()}
                                    selected={is_selected}
                                    on_click={Callback::from(move |_| on_select.emit(c.clone()))}
                                />
                            }
                        })}
                    } else {
                        <div class="p-4 text-center" style="color: var(--fg-muted);">
                            {"No clients found"}
                        </div>
                    }
                </div>
            </div>

            // Right Panel - Client Detail
            <div class="flex-1 overflow-y-auto" style="background-color: var(--bg-primary);">
                if let Some(client) = (*selected_client).clone() {
                    <ClientDetail client={client} />
                } else {
                    <div class="h-full flex items-center justify-center">
                        <div class="text-center">
                            <svg class="w-16 h-16 mx-auto mb-4" style="color: var(--fg-dimmed);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>
                            </svg>
                            <p style="color: var(--fg-muted);">{"Select a client to view details"}</p>
                        </div>
                    </div>
                }
            </div>
        </div>
    }
}

// ===== Client List Item Component =====

#[derive(Properties, PartialEq)]
struct ClientListItemProps {
    client: Client,
    selected: bool,
    on_click: Callback<()>,
}

#[function_component(ClientListItem)]
fn client_list_item(props: &ClientListItemProps) -> Html {
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
            class="px-4 py-3 cursor-pointer border-b hover:bg-gray-700/30"
            style={format!("border-color: var(--border-primary); {}", bg_style)}
        >
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 rounded-full flex items-center justify-center" style="background-color: var(--accent-blue-dark);">
                        <span class="text-white font-medium">
                            {props.client.name.chars().next().unwrap_or('?')}
                        </span>
                    </div>
                    <div>
                        <div class="font-medium" style="color: var(--fg-primary);">{&props.client.name}</div>
                        <div class="text-sm" style="color: var(--fg-muted);">
                            {props.client.city.as_deref().unwrap_or("")}
                            {if props.client.city.is_some() && props.client.state.is_some() { ", " } else { "" }}
                            {props.client.state.as_deref().unwrap_or("")}
                        </div>
                    </div>
                </div>
                <span
                    class="px-2 py-0.5 text-xs rounded"
                    style={format!("background-color: {}20; color: {}", props.client.status.color(), props.client.status.color())}
                >
                    {props.client.status.as_str()}
                </span>
            </div>
        </div>
    }
}

// ===== Client Detail Component =====

#[derive(Properties, PartialEq)]
struct ClientDetailProps {
    client: Client,
}

#[function_component(ClientDetail)]
fn client_detail(props: &ClientDetailProps) -> Html {
    html! {
        <div class="p-6 max-w-4xl">
            // Header
            <div class="flex items-center justify-between mb-6">
                <div class="flex items-center space-x-4">
                    <div class="w-16 h-16 rounded-full flex items-center justify-center text-2xl" style="background-color: var(--accent-blue-dark);">
                        <span class="text-white font-bold">
                            {props.client.name.chars().next().unwrap_or('?')}
                        </span>
                    </div>
                    <div>
                        <h2 class="text-2xl font-semibold" style="color: var(--fg-primary);">{&props.client.name}</h2>
                        <div class="flex items-center space-x-3 mt-1">
                            <span
                                class="px-2 py-0.5 text-xs rounded"
                                style={format!("background-color: {}20; color: {}", props.client.status.color(), props.client.status.color())}
                            >
                                {props.client.status.as_str()}
                            </span>
                            if let Some(industry) = &props.client.industry {
                                <span class="text-sm" style="color: var(--fg-muted);">{industry}</span>
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
                </div>
            </div>

            // Quick Actions
            <div class="flex items-center space-x-3 mb-6">
                <button
                    class="flex items-center space-x-2 px-4 py-2 rounded-lg text-sm font-medium"
                    style="background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z"/>
                    </svg>
                    <span>{"New Ticket"}</span>
                </button>
                <button
                    class="flex items-center space-x-2 px-4 py-2 rounded-lg text-sm font-medium"
                    style="background-color: var(--button-secondary-bg); color: var(--fg-secondary);"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                    </svg>
                    <span>{"New Invoice"}</span>
                </button>
                <button
                    class="flex items-center space-x-2 px-4 py-2 rounded-lg text-sm font-medium"
                    style="background-color: var(--button-secondary-bg); color: var(--fg-secondary);"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
                    </svg>
                    <span>{"View KB"}</span>
                </button>
            </div>

            // Info Cards Grid
            <div class="grid grid-cols-2 gap-6 mb-6">
                // Contact Information
                <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Contact Information"}</h3>
                    <div class="space-y-3">
                        if let Some(email) = &props.client.email {
                            <div class="flex items-center space-x-3">
                                <svg class="w-4 h-4" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
                                </svg>
                                <span style="color: var(--fg-secondary);">{email}</span>
                            </div>
                        }
                        if let Some(phone) = &props.client.phone {
                            <div class="flex items-center space-x-3">
                                <svg class="w-4 h-4" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z"/>
                                </svg>
                                <span style="color: var(--fg-secondary);">{phone}</span>
                            </div>
                        }
                        if let Some(website) = &props.client.website {
                            <div class="flex items-center space-x-3">
                                <svg class="w-4 h-4" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"/>
                                </svg>
                                <a href={website.clone()} target="_blank" style="color: var(--accent-primary);">{website}</a>
                            </div>
                        }
                        if props.client.address.is_some() {
                            <div class="flex items-start space-x-3">
                                <svg class="w-4 h-4 mt-0.5" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"/>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"/>
                                </svg>
                                <div style="color: var(--fg-secondary);">
                                    <div>{props.client.address.as_deref().unwrap_or("")}</div>
                                    <div>
                                        {props.client.city.as_deref().unwrap_or("")}
                                        {if props.client.city.is_some() && props.client.state.is_some() { ", " } else { "" }}
                                        {props.client.state.as_deref().unwrap_or("")}
                                        {" "}
                                        {props.client.zip.as_deref().unwrap_or("")}
                                    </div>
                                </div>
                            </div>
                        }
                    </div>
                </div>

                // Contract Information
                <div class="rounded-lg p-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <h3 class="text-lg font-medium mb-4" style="color: var(--fg-primary);">{"Contract Details"}</h3>
                    <div class="space-y-3">
                        <div class="flex justify-between">
                            <span style="color: var(--fg-muted);">{"Contract Type"}</span>
                            <span style="color: var(--fg-secondary);">
                                {props.client.contract_type.as_deref().unwrap_or("Not specified")}
                            </span>
                        </div>
                        <div class="flex justify-between">
                            <span style="color: var(--fg-muted);">{"Monthly Revenue"}</span>
                            <span class="font-mono" style="color: var(--color-success);">
                                {props.client.monthly_revenue.map(|r| format!("${:.2}", r)).unwrap_or("-".to_string())}
                            </span>
                        </div>
                        <div class="flex justify-between">
                            <span style="color: var(--fg-muted);">{"Client Since"}</span>
                            <span style="color: var(--fg-secondary);">
                                {&props.client.created_at}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            // Contacts Section
            <div class="rounded-lg p-6 mb-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="text-lg font-medium" style="color: var(--fg-primary);">{"Contacts"}</h3>
                    <button
                        class="flex items-center space-x-1 px-3 py-1.5 rounded-lg text-sm"
                        style="background-color: var(--button-secondary-bg); color: var(--fg-secondary);"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                        </svg>
                        <span>{"Add Contact"}</span>
                    </button>
                </div>

                <div class="space-y-3">
                    { for props.client.contacts.iter().map(|contact| {
                        html! {
                            <div class="flex items-center justify-between p-4 rounded-lg" style="background-color: var(--bg-tertiary);">
                                <div class="flex items-center space-x-4">
                                    <div class="w-10 h-10 rounded-full flex items-center justify-center" style="background-color: var(--bg-highlight);">
                                        <span style="color: var(--fg-secondary);">
                                            {contact.first_name.chars().next().unwrap_or('?')}
                                            {contact.last_name.chars().next().unwrap_or('?')}
                                        </span>
                                    </div>
                                    <div>
                                        <div class="flex items-center space-x-2">
                                            <span class="font-medium" style="color: var(--fg-primary);">
                                                {format!("{} {}", contact.first_name, contact.last_name)}
                                            </span>
                                            if contact.is_primary {
                                                <span class="px-2 py-0.5 text-xs rounded" style="background-color: var(--accent-primary); color: white;">
                                                    {"Primary"}
                                                </span>
                                            }
                                        </div>
                                        if let Some(title) = &contact.title {
                                            <div class="text-sm" style="color: var(--fg-muted);">{title}</div>
                                        }
                                    </div>
                                </div>
                                <div class="flex items-center space-x-4 text-sm">
                                    if let Some(email) = &contact.email {
                                        <a href={format!("mailto:{}", email)} style="color: var(--accent-primary);">{email}</a>
                                    }
                                    if let Some(phone) = &contact.phone {
                                        <span style="color: var(--fg-secondary);">{phone}</span>
                                    }
                                    <button class="p-1 rounded hover:bg-gray-700" style="color: var(--fg-muted);">
                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"/>
                                        </svg>
                                    </button>
                                </div>
                            </div>
                        }
                    })}
                </div>
            </div>

            // Notes Section
            if let Some(notes) = &props.client.notes {
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
