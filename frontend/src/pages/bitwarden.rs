use yew::prelude::*;
use yew_hooks::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BitwardenServer {
    pub id: Uuid,
    pub client_id: Uuid,
    pub server_name: String,
    pub server_url: String,
    pub server_type: String, // vaultwarden, bitwarden
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_enabled: bool,
    pub organization_count: i32,
    pub vault_count: i32,
    pub collection_count: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BitwardenOrganization {
    pub id: Uuid,
    pub server_id: Uuid,
    pub organization_id: String,
    pub name: String,
    pub business_name: Option<String>,
    pub billing_email: String,
    pub plan_type: String,
    pub seats: i32,
    pub max_collections: Option<i32>,
    pub max_storage_gb: Option<i32>,
    pub use_policies: bool,
    pub use_sso: bool,
    pub use_directory: bool,
    pub use_events: bool,
    pub use_groups: bool,
    pub collection_count: i32,
    pub member_count: i32,
    pub vault_count: i32,
}

#[function_component(BitwardenPage)]
pub fn bitwarden_page() -> Html {
    let servers = use_state(Vec::<BitwardenServer>::new);
    let selected_server = use_state(|| None::<Uuid>);
    let active_tab = use_state(|| "overview".to_string());
    let show_add_server = use_state(|| false);
    let loading = use_state(|| false);

    // Fetch servers on mount
    {
        let servers = servers.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            let servers = servers.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match Request::get("/api/v1/bitwarden/servers")
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<Vec<BitwardenServer>>().await {
                            servers.set(data);
                        }
                    }
                    _ => {}
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_sync_server = {
        let selected_server = selected_server.clone();
        Callback::from(move |_| {
            if let Some(server_id) = *selected_server {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post(&format!("/api/v1/bitwarden/servers/{}/sync", server_id))
                        .send()
                        .await;
                });
            }
        })
    };

    html! {
        <div class="p-6">
            <div class="mb-6">
                <h1 class="text-3xl font-bold text-gray-900">{"Bitwarden Management"}</h1>
                <p class="text-gray-600 mt-2">{"Manage Bitwarden/Vaultwarden servers and password synchronization"}</p>
            </div>

            // Server selector
            <div class="bg-white rounded-lg shadow p-4 mb-6">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <label class="text-sm font-medium text-gray-700">{"Select Server:"}</label>
                        <select 
                            class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500"
                            onchange={
                                let selected_server = selected_server.clone();
                                Callback::from(move |e: Event| {
                                    let target: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    if let Ok(id) = target.value().parse::<Uuid>() {
                                        selected_server.set(Some(id));
                                    }
                                })
                            }
                        >
                            <option value="">{"-- Select a server --"}</option>
                            {for servers.iter().map(|s| html! {
                                <option value={s.id.to_string()}>{&s.server_name}</option>
                            })}
                        </select>
                    </div>
                    <div class="flex space-x-2">
                        <button 
                            onclick={on_sync_server}
                            disabled={selected_server.is_none()}
                            class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400"
                        >
                            {"🔄 Sync Passwords"}
                        </button>
                        <button 
                            onclick={
                                let show_add_server = show_add_server.clone();
                                Callback::from(move |_| show_add_server.set(true))
                            }
                            class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                        >
                            {"+ Add Server"}
                        </button>
                    </div>
                </div>
            </div>

            {if let Some(server_id) = *selected_server {
                html! {
                    <>
                        // Tab navigation
                        <div class="border-b border-gray-200 mb-6">
                            <nav class="flex space-x-8">
                                {for ["overview", "organizations", "collections", "passwords", "sync"].iter().map(|tab| {
                                    let is_active = *active_tab == *tab;
                                    let active_tab = active_tab.clone();
                                    let tab_str = tab.to_string();
                                    html! {
                                        <button
                                            onclick={Callback::from(move |_| active_tab.set(tab_str.clone()))}
                                            class={format!(
                                                "py-2 px-1 border-b-2 font-medium text-sm {}",
                                                if is_active {
                                                    "border-green-500 text-green-600"
                                                } else {
                                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                                }
                                            )}
                                        >
                                            {tab.to_uppercase()}
                                        </button>
                                    }
                                })}
                            </nav>
                        </div>

                        // Tab content
                        <div class="bg-white rounded-lg shadow">
                            {match active_tab.as_str() {
                                "overview" => html! { <ServerOverview {server_id} /> },
                                "organizations" => html! { <OrganizationsTab {server_id} /> },
                                "collections" => html! { <CollectionsTab {server_id} /> },
                                "passwords" => html! { <PasswordsTab {server_id} /> },
                                "sync" => html! { <SyncTab {server_id} /> },
                                _ => html! { <div>{"Unknown tab"}</div> }
                            }}
                        </div>
                    </>
                }
            } else if !(*loading) && servers.is_empty() {
                html! {
                    <div class="bg-white rounded-lg shadow p-12 text-center">
                        <h3 class="text-lg font-medium text-gray-900 mb-2">{"No Bitwarden Servers Configured"}</h3>
                        <p class="text-gray-600 mb-4">{"Add your first Bitwarden or Vaultwarden server to start password synchronization"}</p>
                        <button 
                            onclick={
                                let show_add_server = show_add_server.clone();
                                Callback::from(move |_| show_add_server.set(true))
                            }
                            class="px-6 py-3 bg-green-600 text-white rounded-md hover:bg-green-700"
                        >
                            {"Add First Server"}
                        </button>
                    </div>
                }
            } else if !(*loading) {
                html! {
                    <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                        <p class="text-yellow-800">{"Please select a server to manage passwords"}</p>
                    </div>
                }
            } else {
                html! {
                    <div class="flex justify-center items-center h-64">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-green-600"></div>
                    </div>
                }
            }}
        </div>
    }
}

// Component implementations for tabs
#[derive(Properties, Clone, PartialEq)]
struct TabProps {
    server_id: Uuid,
}

#[function_component(ServerOverview)]
fn server_overview(props: &TabProps) -> Html {
    let server = use_state(|| None::<BitwardenServer>);
    let organizations = use_state(Vec::<BitwardenOrganization>::new);
    
    {
        let server = server.clone();
        let organizations = organizations.clone();
        let server_id = props.server_id;
        use_effect_with(server_id, move |_| {
            let server = server.clone();
            let organizations = organizations.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // Fetch server details
                if let Ok(resp) = Request::get(&format!("/api/v1/bitwarden/servers/{}", server_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<BitwardenServer>().await {
                            server.set(Some(data));
                        }
                    }
                }

                // Fetch organizations
                if let Ok(resp) = Request::get(&format!("/api/v1/bitwarden/servers/{}/organizations", server_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<Vec<BitwardenOrganization>>().await {
                            organizations.set(data);
                        }
                    }
                }
            });
            || ()
        });
    }

    if let Some(srv) = server.as_ref() {
        html! {
            <div class="p-6">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    <div class="bg-green-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-green-800 mb-2">{"Server Status"}</h3>
                        <div class="text-2xl font-bold text-green-600">{srv.status.to_uppercase()}</div>
                        <p class="text-sm text-green-600 mt-1">{srv.server_type.to_uppercase()}</p>
                    </div>

                    <div class="bg-blue-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-blue-800 mb-2">{"Organizations"}</h3>
                        <div class="text-2xl font-bold text-blue-600">{srv.organization_count}</div>
                        <p class="text-sm text-blue-600 mt-1">{"Total managed"}</p>
                    </div>

                    <div class="bg-purple-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-purple-800 mb-2">{"Collections"}</h3>
                        <div class="text-2xl font-bold text-purple-600">{srv.collection_count}</div>
                        <p class="text-sm text-purple-600 mt-1">{"Password groups"}</p>
                    </div>

                    <div class="bg-orange-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-orange-800 mb-2">{"Vaults"}</h3>
                        <div class="text-2xl font-bold text-orange-600">{srv.vault_count}</div>
                        <p class="text-sm text-orange-600 mt-1">{"Password entries"}</p>
                    </div>
                </div>

                // Organizations Summary
                <div class="mb-8">
                    <h3 class="text-lg font-medium text-gray-900 mb-4">{"Organizations"}</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {for organizations.iter().map(|org| html! {
                            <div class="border rounded-lg p-4 hover:shadow-md transition-shadow">
                                <div class="flex justify-between items-start mb-2">
                                    <h4 class="font-medium text-gray-900">{&org.name}</h4>
                                    <span class="text-xs bg-gray-100 px-2 py-1 rounded">{&org.plan_type}</span>
                                </div>
                                <div class="grid grid-cols-3 gap-2 text-xs text-gray-600 mb-2">
                                    <div class="text-center">
                                        <div class="font-medium text-blue-600">{org.member_count}</div>
                                        <div>{"Members"}</div>
                                    </div>
                                    <div class="text-center">
                                        <div class="font-medium text-purple-600">{org.collection_count}</div>
                                        <div>{"Collections"}</div>
                                    </div>
                                    <div class="text-center">
                                        <div class="font-medium text-orange-600">{org.vault_count}</div>
                                        <div>{"Vaults"}</div>
                                    </div>
                                </div>
                                <div class="flex justify-between items-center text-xs text-gray-500">
                                    <span>{"Seats: "}{org.seats}</span>
                                    <div class="flex space-x-2">
                                        {if org.use_sso {
                                            html! { <span class="bg-green-100 text-green-800 px-1 rounded">{"SSO"}</span> }
                                        } else { html! {} }}
                                        {if org.use_policies {
                                            html! { <span class="bg-blue-100 text-blue-800 px-1 rounded">{"Policies"}</span> }
                                        } else { html! {} }}
                                    </div>
                                </div>
                            </div>
                        })}
                    </div>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="flex justify-center items-center h-64">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-green-600"></div>
            </div>
        }
    }
}

// Placeholder implementations for other tabs
#[function_component(OrganizationsTab)]
fn organizations_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Organization management coming soon..."}</p>
        </div>
    }
}

#[function_component(CollectionsTab)]
fn collections_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Collection management coming soon..."}</p>
        </div>
    }
}

#[function_component(PasswordsTab)]
fn passwords_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Password synchronization coming soon..."}</p>
        </div>
    }
}

#[function_component(SyncTab)]
fn sync_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Sync configuration coming soon..."}</p>
        </div>
    }
}