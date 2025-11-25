use yew::prelude::*;
use yew_hooks::prelude::*;
use web_sys::HtmlInputElement;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AzureSubscription {
    pub id: Uuid,
    pub client_id: Uuid,
    pub subscription_id: String,
    pub subscription_name: String,
    pub tenant_id: String,
    pub state: Option<String>,
    pub spending_limit: Option<String>,
    pub current_spend_usd: Option<f64>,
    pub budget_limit_usd: Option<f64>,
    pub budget_alerts_enabled: bool,
    pub sync_enabled: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_sync_status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AzureResourceGroup {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub resource_group_id: String,
    pub name: String,
    pub location: String,
    pub provisioning_state: Option<String>,
    pub total_resources: i32,
    pub compute_resources: i32,
    pub storage_resources: i32,
    pub network_resources: i32,
    pub database_resources: i32,
    pub monthly_cost_usd: Option<f64>,
    pub daily_cost_usd: Option<f64>,
    pub tags: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AzureResource {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub resource_group_id: Uuid,
    pub resource_id: String,
    pub name: String,
    pub resource_type: String,
    pub kind: Option<String>,
    pub location: String,
    pub provisioning_state: Option<String>,
    pub power_state: Option<String>,
    pub vm_size: Option<String>,
    pub os_type: Option<String>,
    pub daily_cost_usd: Option<f64>,
    pub monthly_cost_usd: Option<f64>,
    pub cpu_utilization_avg: Option<f64>,
    pub memory_utilization_avg: Option<f64>,
    pub backup_enabled: bool,
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_monthly_cost: f64,
    pub total_daily_cost: f64,
    pub budget_utilization: f64,
    pub top_costs: Vec<ResourceCost>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceCost {
    pub name: String,
    pub resource_type: String,
    pub monthly_cost: f64,
}

#[function_component(AzurePage)]
pub fn azure_page() -> Html {
    let subscriptions = use_state(Vec::<AzureSubscription>::new);
    let selected_subscription = use_state(|| None::<Uuid>);
    let active_tab = use_state(|| "overview".to_string());
    let show_add_subscription = use_state(|| false);
    let loading = use_state(|| false);

    // Fetch subscriptions on mount
    {
        let subscriptions = subscriptions.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            let subscriptions = subscriptions.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match Request::get("/api/v1/azure/subscriptions")
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<Vec<AzureSubscription>>().await {
                            subscriptions.set(data);
                        }
                    }
                    _ => {}
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_sync_subscription = {
        let selected_subscription = selected_subscription.clone();
        Callback::from(move |_| {
            if let Some(subscription_id) = *selected_subscription {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post(&format!("/api/v1/azure/subscriptions/{}/sync", subscription_id))
                        .send()
                        .await;
                });
            }
        })
    };

    html! {
        <div class="p-6">
            <div class="mb-6">
                <h1 class="text-3xl font-bold text-gray-900">{"Azure Resource Monitoring"}</h1>
                <p class="text-gray-600 mt-2">{"Monitor Azure subscriptions, resources, and costs"}</p>
            </div>

            {if *show_add_subscription {
                html! { <AddSubscriptionModal 
                    on_close={
                        let show_add_subscription = show_add_subscription.clone();
                        Callback::from(move |_| show_add_subscription.set(false))
                    } 
                    on_save={
                        let subscriptions = subscriptions.clone();
                        let show_add_subscription = show_add_subscription.clone();
                        Callback::from(move |subscription: AzureSubscription| {
                            let mut current = (*subscriptions).clone();
                            current.push(subscription);
                            subscriptions.set(current);
                            show_add_subscription.set(false);
                        })
                    }
                /> }
            } else {
                html! {}
            }}

            // Subscription selector
            <div class="bg-white rounded-lg shadow p-4 mb-6">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <label class="text-sm font-medium text-gray-700">{"Select Subscription:"}</label>
                        <select 
                            class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            onchange={
                                let selected_subscription = selected_subscription.clone();
                                Callback::from(move |e: Event| {
                                    let target: HtmlInputElement = e.target_unchecked_into();
                                    if let Ok(id) = target.value().parse::<Uuid>() {
                                        selected_subscription.set(Some(id));
                                    }
                                })
                            }
                        >
                            <option value="">{"-- Select a subscription --"}</option>
                            {for subscriptions.iter().map(|s| html! {
                                <option value={s.id.to_string()}>{&s.subscription_name}</option>
                            })}
                        </select>
                    </div>
                    <div class="flex space-x-2">
                        <button 
                            onclick={on_sync_subscription}
                            disabled={selected_subscription.is_none()}
                            class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400"
                        >
                            {"🔄 Sync Resources"}
                        </button>
                        <button 
                            onclick={
                                let show_add_subscription = show_add_subscription.clone();
                                Callback::from(move |_| show_add_subscription.set(true))
                            }
                            class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                        >
                            {"+ Add Subscription"}
                        </button>
                    </div>
                </div>
            </div>

            {if let Some(subscription_id) = *selected_subscription {
                html! {
                    <>
                        // Tab navigation
                        <div class="border-b border-gray-200 mb-6">
                            <nav class="flex space-x-8">
                                {for ["overview", "compute", "storage", "networking", "databases", "costs", "security"].iter().map(|tab| {
                                    let is_active = *active_tab == *tab;
                                    let active_tab = active_tab.clone();
                                    let tab_str = tab.to_string();
                                    html! {
                                        <button
                                            onclick={Callback::from(move |_| active_tab.set(tab_str.clone()))}
                                            class={format!(
                                                "py-2 px-1 border-b-2 font-medium text-sm {}",
                                                if is_active {
                                                    "border-blue-500 text-blue-600"
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
                                "overview" => html! { <SubscriptionOverview {subscription_id} /> },
                                "compute" => html! { <ComputeTab {subscription_id} /> },
                                "storage" => html! { <StorageTab {subscription_id} /> },
                                "networking" => html! { <NetworkingTab {subscription_id} /> },
                                "databases" => html! { <DatabasesTab {subscription_id} /> },
                                "costs" => html! { <CostsTab {subscription_id} /> },
                                "security" => html! { <SecurityTab {subscription_id} /> },
                                _ => html! { <div>{"Unknown tab"}</div> }
                            }}
                        </div>
                    </>
                }
            } else if !(*loading) && subscriptions.is_empty() {
                html! {
                    <div class="bg-white rounded-lg shadow p-12 text-center">
                        <h3 class="text-lg font-medium text-gray-900 mb-2">{"No Azure Subscriptions Configured"}</h3>
                        <p class="text-gray-600 mb-4">{"Add your first Azure subscription to start monitoring resources"}</p>
                        <button 
                            onclick={
                                let show_add_subscription = show_add_subscription.clone();
                                Callback::from(move |_| show_add_subscription.set(true))
                            }
                            class="px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                        >
                            {"Add First Subscription"}
                        </button>
                    </div>
                }
            } else if !(*loading) {
                html! {
                    <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                        <p class="text-yellow-800">{"Please select a subscription to view resources"}</p>
                    </div>
                }
            } else {
                html! {
                    <div class="flex justify-center items-center h-64">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                    </div>
                }
            }}
        </div>
    }
}

// Component implementations for tabs
#[derive(Properties, Clone, PartialEq)]
struct TabProps {
    subscription_id: Uuid,
}

#[function_component(SubscriptionOverview)]
fn subscription_overview(props: &TabProps) -> Html {
    let subscription = use_state(|| None::<AzureSubscription>);
    let resource_groups = use_state(Vec::<AzureResourceGroup>::new);
    let cost_summary = use_state(|| None::<CostSummary>);
    
    {
        let subscription = subscription.clone();
        let resource_groups = resource_groups.clone();
        let cost_summary = cost_summary.clone();
        let subscription_id = props.subscription_id;
        use_effect_with(subscription_id, move |_| {
            let subscription = subscription.clone();
            let resource_groups = resource_groups.clone();
            let cost_summary = cost_summary.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // Fetch subscription details
                if let Ok(resp) = Request::get(&format!("/api/v1/azure/subscriptions/{}", subscription_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<AzureSubscription>().await {
                            subscription.set(Some(data));
                        }
                    }
                }

                // Fetch resource groups
                if let Ok(resp) = Request::get(&format!("/api/v1/azure/subscriptions/{}/resource-groups", subscription_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<Vec<AzureResourceGroup>>().await {
                            resource_groups.set(data);
                        }
                    }
                }

                // Fetch cost summary
                if let Ok(resp) = Request::get(&format!("/api/v1/azure/subscriptions/{}/costs/summary", subscription_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<CostSummary>().await {
                            cost_summary.set(Some(data));
                        }
                    }
                }
            });
            || ()
        });
    }

    if let (Some(sub), Some(costs)) = (subscription.as_ref(), cost_summary.as_ref()) {
        html! {
            <div class="p-6">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    <div class="bg-blue-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-blue-800 mb-2">{"Subscription Status"}</h3>
                        <div class="text-2xl font-bold text-blue-600">{sub.state.as_ref().unwrap_or(&"Unknown".to_string()).to_uppercase()}</div>
                        <p class="text-sm text-blue-600 mt-1">{&sub.subscription_name}</p>
                    </div>

                    <div class="bg-green-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-green-800 mb-2">{"Resource Groups"}</h3>
                        <div class="text-2xl font-bold text-green-600">{resource_groups.len()}</div>
                        <p class="text-sm text-green-600 mt-1">{
                            format!("{} total resources", 
                                resource_groups.iter().map(|rg| rg.total_resources).sum::<i32>()
                            )
                        }</p>
                    </div>

                    <div class="bg-yellow-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-yellow-800 mb-2">{"Monthly Cost"}</h3>
                        <div class="text-2xl font-bold text-yellow-600">{format!("${:.2}", costs.total_monthly_cost)}</div>
                        <p class="text-sm text-yellow-600 mt-1">{format!("${:.2}/day avg", costs.total_daily_cost)}</p>
                    </div>

                    <div class="bg-purple-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-purple-800 mb-2">{"Budget Utilization"}</h3>
                        <div class="text-2xl font-bold text-purple-600">{format!("{:.1}%", costs.budget_utilization)}</div>
                        {if let Some(budget) = sub.budget_limit_usd {
                            html! { <p class="text-sm text-purple-600 mt-1">{format!("of ${:.2}", budget)}</p> }
                        } else {
                            html! { <p class="text-sm text-purple-600 mt-1">{"No budget set"}</p> }
                        }}
                    </div>
                </div>

                // Resource Groups Summary
                <div class="mb-8">
                    <h3 class="text-lg font-medium text-gray-900 mb-4">{"Resource Groups"}</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {for resource_groups.iter().map(|rg| html! {
                            <div class="border rounded-lg p-4 hover:shadow-md transition-shadow">
                                <div class="flex justify-between items-start mb-2">
                                    <h4 class="font-medium text-gray-900">{&rg.name}</h4>
                                    <span class="text-xs bg-gray-100 px-2 py-1 rounded">{&rg.location}</span>
                                </div>
                                <div class="grid grid-cols-4 gap-2 text-xs text-gray-600 mb-2">
                                    <div class="text-center">
                                        <div class="font-medium text-blue-600">{rg.compute_resources}</div>
                                        <div>{"Compute"}</div>
                                    </div>
                                    <div class="text-center">
                                        <div class="font-medium text-green-600">{rg.storage_resources}</div>
                                        <div>{"Storage"}</div>
                                    </div>
                                    <div class="text-center">
                                        <div class="font-medium text-purple-600">{rg.network_resources}</div>
                                        <div>{"Network"}</div>
                                    </div>
                                    <div class="text-center">
                                        <div class="font-medium text-orange-600">{rg.database_resources}</div>
                                        <div>{"Database"}</div>
                                    </div>
                                </div>
                                {if let Some(cost) = rg.monthly_cost_usd {
                                    html! { <p class="text-sm text-gray-600">{format!("${:.2}/month", cost)}</p> }
                                } else {
                                    html! { <p class="text-sm text-gray-400">{"Cost data pending"}</p> }
                                }}
                            </div>
                        })}
                    </div>
                </div>

                // Top Costs
                {if !costs.top_costs.is_empty() {
                    html! {
                        <div>
                            <h3 class="text-lg font-medium text-gray-900 mb-4">{"Top Cost Resources"}</h3>
                            <div class="overflow-x-auto">
                                <table class="min-w-full divide-y divide-gray-200">
                                    <thead class="bg-gray-50">
                                        <tr>
                                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Resource"}</th>
                                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Type"}</th>
                                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Monthly Cost"}</th>
                                        </tr>
                                    </thead>
                                    <tbody class="bg-white divide-y divide-gray-200">
                                        {for costs.top_costs.iter().take(10).map(|resource| html! {
                                            <tr>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{&resource.name}</td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{&resource.resource_type}</td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{format!("${:.2}", resource.monthly_cost)}</td>
                                            </tr>
                                        })}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        }
    } else {
        html! {
            <div class="flex justify-center items-center h-64">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
            </div>
        }
    }
}

#[function_component(ComputeTab)]
fn compute_tab(props: &TabProps) -> Html {
    let resources = use_state(Vec::<AzureResource>::new);
    let loading = use_state(|| true);
    
    {
        let resources = resources.clone();
        let loading = loading.clone();
        let subscription_id = props.subscription_id;
        use_effect_with(subscription_id, move |_| {
            let resources = resources.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                if let Ok(resp) = Request::get(&format!("/api/v1/azure/subscriptions/{}/resources?type=compute", subscription_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<Vec<AzureResource>>().await {
                            resources.set(data);
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
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                    </div>
                }
            } else if resources.is_empty() {
                html! {
                    <div class="text-center py-12">
                        <p class="text-gray-500">{"No compute resources found. Try syncing the subscription."}</p>
                    </div>
                }
            } else {
                html! {
                    <div class="overflow-x-auto">
                        <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Name"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Type"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Size/SKU"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Status"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Location"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Monthly Cost"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"CPU %"}</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                {for resources.iter().map(|resource| html! {
                                    <tr class="hover:bg-gray-50">
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div>
                                                <div class="text-sm font-medium text-gray-900">{&resource.name}</div>
                                                {if let Some(os) = &resource.os_type {
                                                    html! { <div class="text-xs text-gray-500">{format!("{} OS", os.to_uppercase())}</div> }
                                                } else {
                                                    html! {}
                                                }}
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{&resource.resource_type}</td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                            {resource.vm_size.as_ref().unwrap_or(&"—".to_string())}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            {match resource.power_state.as_ref().map(|s| s.as_str()) {
                                                Some("running") => html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">{"Running"}</span> },
                                                Some("stopped") => html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800">{"Stopped"}</span> },
                                                Some("deallocated") => html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800">{"Deallocated"}</span> },
                                                _ => html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-yellow-100 text-yellow-800">{"Unknown"}</span> }
                                            }}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{&resource.location}</td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                            {resource.monthly_cost_usd.map(|c| format!("${:.2}", c)).unwrap_or_else(|| "—".to_string())}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                            {resource.cpu_utilization_avg.map(|c| format!("{:.1}%", c)).unwrap_or_else(|| "—".to_string())}
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

// Placeholder implementations for other tabs
#[function_component(StorageTab)]
fn storage_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Storage resources coming soon..."}</p>
        </div>
    }
}

#[function_component(NetworkingTab)]
fn networking_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Networking resources coming soon..."}</p>
        </div>
    }
}

#[function_component(DatabasesTab)]
fn databases_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Database resources coming soon..."}</p>
        </div>
    }
}

#[function_component(CostsTab)]
fn costs_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Detailed cost analytics coming soon..."}</p>
        </div>
    }
}

#[function_component(SecurityTab)]
fn security_tab(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"Security recommendations coming soon..."}</p>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
struct ModalProps {
    on_close: Callback<()>,
    on_save: Callback<AzureSubscription>,
}

#[function_component(AddSubscriptionModal)]
fn add_subscription_modal(props: &ModalProps) -> Html {
    html! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white rounded-lg p-6 w-full max-w-md">
                <h2 class="text-xl font-bold mb-4">{"Add Azure Subscription"}</h2>
                <p class="text-gray-600 mb-4">{"Configure Azure subscription monitoring"}</p>
                <div class="flex justify-end space-x-2">
                    <button 
                        onclick={let on_close = props.on_close.clone(); Callback::from(move |_| on_close.emit(()))}
                        class="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50"
                    >
                        {"Cancel"}
                    </button>
                    <button class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
                        {"Save"}
                    </button>
                </div>
            </div>
        </div>
    }
}