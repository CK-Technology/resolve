use yew::prelude::*;
use yew_hooks::prelude::*;
use web_sys::HtmlInputElement;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct M365Tenant {
    pub id: Uuid,
    pub client_id: Uuid,
    pub tenant_id: String,
    pub tenant_name: String,
    pub domain_name: String,
    pub display_name: Option<String>,
    pub default_domain: Option<String>,
    pub tenant_type: String,
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_enabled: bool,
    pub total_licenses: i32,
    pub assigned_licenses: i32,
    pub available_licenses: i32,
    pub security_defaults_enabled: Option<bool>,
    pub mfa_required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct M365User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: String,
    pub user_principal_name: String,
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub mail: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub account_enabled: bool,
    pub last_sign_in: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
    pub assigned_licenses: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct M365Group {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub group_id: String,
    pub display_name: String,
    pub mail: Option<String>,
    pub description: Option<String>,
    pub group_type: Option<String>,
    pub security_enabled: bool,
    pub mail_enabled: bool,
    pub is_teams_enabled: bool,
    pub member_count: i32,
    pub owner_count: i32,
    pub visibility: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct M365License {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub sku_id: String,
    pub sku_part_number: String,
    pub product_name: String,
    pub total_units: i32,
    pub consumed_units: i32,
    pub enabled_units: i32,
    pub cost_per_license: Option<f64>,
    pub renewal_date: Option<chrono::NaiveDate>,
}

#[function_component(M365Page)]
pub fn m365_page() -> Html {
    let tenants = use_state(Vec::<M365Tenant>::new);
    let selected_tenant = use_state(|| None::<Uuid>);
    let active_tab = use_state(|| "overview".to_string());
    let show_add_tenant = use_state(|| false);
    let loading = use_state(|| false);

    // Fetch tenants on mount
    {
        let tenants = tenants.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            let tenants = tenants.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match Request::get("/api/v1/m365/tenants")
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<Vec<M365Tenant>>().await {
                            tenants.set(data);
                        }
                    }
                    _ => {}
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_sync_tenant = {
        let selected_tenant = selected_tenant.clone();
        Callback::from(move |_| {
            if let Some(tenant_id) = *selected_tenant {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post(&format!("/api/v1/m365/tenants/{}/sync", tenant_id))
                        .send()
                        .await;
                });
            }
        })
    };

    html! {
        <div class="p-6">
            <div class="mb-6">
                <h1 class="text-3xl font-bold text-gray-900">{"Microsoft 365 Management"}</h1>
                <p class="text-gray-600 mt-2">{"Manage M365 tenants, users, licenses, and services"}</p>
            </div>

            {if *show_add_tenant {
                html! { <AddTenantModal 
                    on_close={
                        let show_add_tenant = show_add_tenant.clone();
                        Callback::from(move |_| show_add_tenant.set(false))
                    } 
                    on_save={
                        let tenants = tenants.clone();
                        let show_add_tenant = show_add_tenant.clone();
                        Callback::from(move |tenant: M365Tenant| {
                            let mut current = (*tenants).clone();
                            current.push(tenant);
                            tenants.set(current);
                            show_add_tenant.set(false);
                        })
                    }
                /> }
            } else {
                html! {}
            }}

            // Tenant selector
            <div class="bg-white rounded-lg shadow p-4 mb-6">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <label class="text-sm font-medium text-gray-700">{"Select Tenant:"}</label>
                        <select 
                            class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            onchange={
                                let selected_tenant = selected_tenant.clone();
                                Callback::from(move |e: Event| {
                                    let target: HtmlInputElement = e.target_unchecked_into();
                                    if let Ok(id) = target.value().parse::<Uuid>() {
                                        selected_tenant.set(Some(id));
                                    }
                                })
                            }
                        >
                            <option value="">{"-- Select a tenant --"}</option>
                            {for tenants.iter().map(|t| html! {
                                <option value={t.id.to_string()}>{&t.tenant_name}</option>
                            })}
                        </select>
                    </div>
                    <div class="flex space-x-2">
                        <button 
                            onclick={on_sync_tenant}
                            disabled={selected_tenant.is_none()}
                            class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400"
                        >
                            {"🔄 Sync Now"}
                        </button>
                        <button 
                            onclick={
                                let show_add_tenant = show_add_tenant.clone();
                                Callback::from(move |_| show_add_tenant.set(true))
                            }
                            class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                        >
                            {"+ Add Tenant"}
                        </button>
                    </div>
                </div>
            </div>

            {if let Some(tenant_id) = *selected_tenant {
                html! {
                    <>
                        // Tab navigation
                        <div class="border-b border-gray-200 mb-6">
                            <nav class="flex space-x-8">
                                {for ["overview", "users", "groups", "licenses", "services", "security"].iter().map(|tab| {
                                    let is_active = *active_tab == *tab;
                                    let active_tab = active_tab.clone();
                                    let tab_str = tab.to_string();
                                    html! {
                                        <button
                                            onclick={Callback::from(move |_| active_tab.set(tab_str.clone()))}
                                            class={format!(
                                                "py-2 px-1 border-b-2 font-medium text-sm {}",
                                                if is_active {
                                                    "border-indigo-500 text-indigo-600"
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
                                "overview" => html! { <TenantOverview {tenant_id} /> },
                                "users" => html! { <UsersTab {tenant_id} /> },
                                "groups" => html! { <GroupsTab {tenant_id} /> },
                                "licenses" => html! { <LicensesTab {tenant_id} /> },
                                "services" => html! { <ServicesTab {tenant_id} /> },
                                "security" => html! { <SecurityTab {tenant_id} /> },
                                _ => html! { <div>{"Unknown tab"}</div> }
                            }}
                        </div>
                    </>
                }
            } else if !(*loading) && tenants.is_empty() {
                html! {
                    <div class="bg-white rounded-lg shadow p-12 text-center">
                        <h3 class="text-lg font-medium text-gray-900 mb-2">{"No M365 Tenants Configured"}</h3>
                        <p class="text-gray-600 mb-4">{"Add your first Microsoft 365 tenant to get started"}</p>
                        <button 
                            onclick={
                                let show_add_tenant = show_add_tenant.clone();
                                Callback::from(move |_| show_add_tenant.set(true))
                            }
                            class="px-6 py-3 bg-indigo-600 text-white rounded-md hover:bg-indigo-700"
                        >
                            {"Add First Tenant"}
                        </button>
                    </div>
                }
            } else if !(*loading) {
                html! {
                    <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                        <p class="text-yellow-800">{"Please select a tenant to view details"}</p>
                    </div>
                }
            } else {
                html! {
                    <div class="flex justify-center items-center h-64">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                    </div>
                }
            }}
        </div>
    }
}

// Component implementations for tabs
#[derive(Properties, Clone, PartialEq)]
struct TabProps {
    tenant_id: Uuid,
}

#[function_component(TenantOverview)]
fn tenant_overview(props: &TabProps) -> Html {
    let tenant = use_state(|| None::<M365Tenant>);
    
    {
        let tenant = tenant.clone();
        let tenant_id = props.tenant_id;
        use_effect_with(tenant_id, move |_| {
            let tenant = tenant.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get(&format!("/api/v1/m365/tenants/{}", tenant_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<M365Tenant>().await {
                            tenant.set(Some(data));
                        }
                    }
                }
            });
            || ()
        });
    }

    if let Some(tenant) = &*tenant {
        html! {
            <div class="p-6">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    <div class="bg-gray-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-gray-500 mb-2">{"Tenant Information"}</h3>
                        <dl class="space-y-2">
                            <div>
                                <dt class="text-xs text-gray-500">{"Name"}</dt>
                                <dd class="text-sm font-medium">{&tenant.tenant_name}</dd>
                            </div>
                            <div>
                                <dt class="text-xs text-gray-500">{"Domain"}</dt>
                                <dd class="text-sm font-medium">{&tenant.domain_name}</dd>
                            </div>
                            <div>
                                <dt class="text-xs text-gray-500">{"Type"}</dt>
                                <dd class="text-sm font-medium capitalize">{&tenant.tenant_type}</dd>
                            </div>
                        </dl>
                    </div>

                    <div class="bg-gray-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-gray-500 mb-2">{"License Summary"}</h3>
                        <dl class="space-y-2">
                            <div>
                                <dt class="text-xs text-gray-500">{"Total Licenses"}</dt>
                                <dd class="text-2xl font-bold text-indigo-600">{tenant.total_licenses}</dd>
                            </div>
                            <div class="flex space-x-4">
                                <div>
                                    <dt class="text-xs text-gray-500">{"Assigned"}</dt>
                                    <dd class="text-sm font-medium">{tenant.assigned_licenses}</dd>
                                </div>
                                <div>
                                    <dt class="text-xs text-gray-500">{"Available"}</dt>
                                    <dd class="text-sm font-medium text-green-600">{tenant.available_licenses}</dd>
                                </div>
                            </div>
                        </dl>
                    </div>

                    <div class="bg-gray-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-gray-500 mb-2">{"Security Status"}</h3>
                        <dl class="space-y-2">
                            <div class="flex items-center justify-between">
                                <dt class="text-xs text-gray-500">{"MFA Required"}</dt>
                                <dd>{if tenant.mfa_required {
                                    html! { <span class="text-green-600">{"✓ Enabled"}</span> }
                                } else {
                                    html! { <span class="text-red-600">{"✗ Disabled"}</span> }
                                }}</dd>
                            </div>
                            <div class="flex items-center justify-between">
                                <dt class="text-xs text-gray-500">{"Security Defaults"}</dt>
                                <dd>{if tenant.security_defaults_enabled.unwrap_or(false) {
                                    html! { <span class="text-green-600">{"✓ Enabled"}</span> }
                                } else {
                                    html! { <span class="text-yellow-600">{"⚠ Check Settings"}</span> }
                                }}</dd>
                            </div>
                        </dl>
                    </div>
                </div>

                <div class="mt-6 bg-gray-50 p-4 rounded-lg">
                    <h3 class="text-sm font-medium text-gray-500 mb-2">{"Sync Information"}</h3>
                    <dl class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div>
                            <dt class="text-xs text-gray-500">{"Status"}</dt>
                            <dd class="text-sm font-medium">{&tenant.status}</dd>
                        </div>
                        <div>
                            <dt class="text-xs text-gray-500">{"Sync Enabled"}</dt>
                            <dd class="text-sm font-medium">{if tenant.sync_enabled { "Yes" } else { "No" }}</dd>
                        </div>
                        <div>
                            <dt class="text-xs text-gray-500">{"Last Sync"}</dt>
                            <dd class="text-sm font-medium">{
                                tenant.last_sync.map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                    .unwrap_or_else(|| "Never".to_string())
                            }</dd>
                        </div>
                        <div>
                            <dt class="text-xs text-gray-500">{"Tenant ID"}</dt>
                            <dd class="text-sm font-mono text-xs">{&tenant.tenant_id}</dd>
                        </div>
                    </dl>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="flex justify-center items-center h-64">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
            </div>
        }
    }
}

#[function_component(UsersTab)]
fn users_tab(props: &TabProps) -> Html {
    let users = use_state(Vec::<M365User>::new);
    let loading = use_state(|| true);
    
    {
        let users = users.clone();
        let loading = loading.clone();
        let tenant_id = props.tenant_id;
        use_effect_with(tenant_id, move |_| {
            let users = users.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                if let Ok(resp) = Request::get(&format!("/api/v1/m365/tenants/{}/users", tenant_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<Vec<M365User>>().await {
                            users.set(data);
                        }
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    html! {
        <div class="p-6">
            {if *loading {
                html! {
                    <div class="flex justify-center items-center h-64">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                    </div>
                }
            } else if users.is_empty() {
                html! {
                    <div class="text-center py-12">
                        <p class="text-gray-500">{"No users found. Try syncing the tenant."}</p>
                    </div>
                }
            } else {
                html! {
                    <div class="overflow-x-auto">
                        <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"User"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Email"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Department"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Status"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"MFA"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Last Sign In"}</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                {for users.iter().map(|user| html! {
                                    <tr class="hover:bg-gray-50">
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div>
                                                <div class="text-sm font-medium text-gray-900">
                                                    {user.display_name.as_ref().unwrap_or(&user.user_principal_name)}
                                                </div>
                                                <div class="text-sm text-gray-500">{&user.user_principal_name}</div>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                            {user.mail.as_ref().unwrap_or(&"—".to_string())}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                            {user.department.as_ref().unwrap_or(&"—".to_string())}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            {if user.account_enabled {
                                                html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">{"Active"}</span> }
                                            } else {
                                                html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800">{"Disabled"}</span> }
                                            }}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            {if user.mfa_enabled {
                                                html! { <span class="text-green-600">{"✓"}</span> }
                                            } else {
                                                html! { <span class="text-red-600">{"✗"}</span> }
                                            }}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                            {user.last_sign_in.map(|dt| dt.format("%Y-%m-%d").to_string())
                                                .unwrap_or_else(|| "Never".to_string())}
                                        </td>
                                    </tr>
                                })}
                            </tbody>
                        </table>
                    </div>
                }
            }}
        </div>
    }
}

#[function_component(GroupsTab)]
fn groups_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Groups management coming soon..."}</p>
        </div>
    }
}

#[function_component(LicensesTab)]
fn licenses_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"License management coming soon..."}</p>
        </div>
    }
}

#[function_component(ServicesTab)]
fn services_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Services monitoring coming soon..."}</p>
        </div>
    }
}

#[function_component(SecurityTab)]
fn security_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Security settings coming soon..."}</p>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
struct ModalProps {
    on_close: Callback<()>,
    on_save: Callback<M365Tenant>,
}

#[function_component(AddTenantModal)]
fn add_tenant_modal(props: &ModalProps) -> Html {
    html! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white rounded-lg p-6 w-full max-w-md">
                <h2 class="text-xl font-bold mb-4">{"Add M365 Tenant"}</h2>
                <p class="text-gray-600 mb-4">{"Configure Microsoft 365 tenant connection"}</p>
                <div class="flex justify-end space-x-2">
                    <button 
                        onclick={let on_close = props.on_close.clone(); Callback::from(move |_| on_close.emit(()))}
                        class="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50"
                    >
                        {"Cancel"}
                    </button>
                    <button class="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700">
                        {"Save"}
                    </button>
                </div>
            </div>
        </div>
    }
}