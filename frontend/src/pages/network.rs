use yew::prelude::*;
use yew_hooks::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkController {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub controller_type: String, // unifi, fortigate, powerdns, cloudflare
    pub host: String,
    pub port: i32,
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_enabled: bool,
    pub device_count: i32,
    pub site_count: i32,
    pub client_count: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiDevice {
    pub id: Uuid,
    pub controller_id: Uuid,
    pub device_id: String,
    pub name: String,
    pub model: String,
    pub device_type: String, // ap, switch, gateway, security_gateway
    pub site_name: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub status: String,
    pub uptime: Option<i64>,
    pub version: String,
    pub clients_connected: i32,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FortigateInterface {
    pub id: Uuid,
    pub controller_id: Uuid,
    pub name: String,
    pub interface_type: String,
    pub ip_address: Option<String>,
    pub status: String,
    pub speed: Option<String>,
    pub duplex: Option<String>,
    pub vdom: String,
    pub rx_packets: Option<i64>,
    pub tx_packets: Option<i64>,
    pub rx_bytes: Option<i64>,
    pub tx_bytes: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: Uuid,
    pub controller_id: Uuid,
    pub zone: String,
    pub name: String,
    pub record_type: String,
    pub content: String,
    pub ttl: i32,
    pub proxied: Option<bool>,
    pub priority: Option<i32>,
    pub status: String,
}

#[function_component(NetworkPage)]
pub fn network_page() -> Html {
    let controllers = use_state(Vec::<NetworkController>::new);
    let selected_controller = use_state(|| None::<Uuid>);
    let active_tab = use_state(|| "overview".to_string());
    let show_add_controller = use_state(|| false);
    let loading = use_state(|| false);

    // Fetch controllers on mount
    {
        let controllers = controllers.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            let controllers = controllers.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match Request::get("/api/v1/network/controllers")
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<Vec<NetworkController>>().await {
                            controllers.set(data);
                        }
                    }
                    _ => {}
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_sync_controller = {
        let selected_controller = selected_controller.clone();
        Callback::from(move |_| {
            if let Some(controller_id) = *selected_controller {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post(&format!("/api/v1/network/controllers/{}/sync", controller_id))
                        .send()
                        .await;
                });
            }
        })
    };

    html! {
        <div class="p-6">
            <div class="mb-6">
                <h1 class="text-3xl font-bold text-gray-900">{"Network Management"}</h1>
                <p class="text-gray-600 mt-2">{"Manage UniFi, FortiGate, DNS controllers and network infrastructure"}</p>
            </div>

            // Controller selector
            <div class="bg-white rounded-lg shadow p-4 mb-6">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <label class="text-sm font-medium text-gray-700">{"Select Controller:"}</label>
                        <select 
                            class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                            onchange={
                                let selected_controller = selected_controller.clone();
                                Callback::from(move |e: Event| {
                                    let target: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    if let Ok(id) = target.value().parse::<Uuid>() {
                                        selected_controller.set(Some(id));
                                    }
                                })
                            }
                        >
                            <option value="">{"-- Select a controller --"}</option>
                            {for controllers.iter().map(|c| html! {
                                <option value={c.id.to_string()}>{format!("{} ({})", c.name, c.controller_type)}</option>
                            })}
                        </select>
                    </div>
                    <div class="flex space-x-2">
                        <button 
                            onclick={on_sync_controller}
                            disabled={selected_controller.is_none()}
                            class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400"
                        >
                            {"🔄 Sync Devices"}
                        </button>
                        <button 
                            onclick={
                                let show_add_controller = show_add_controller.clone();
                                Callback::from(move |_| show_add_controller.set(true))
                            }
                            class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                        >
                            {"+ Add Controller"}
                        </button>
                    </div>
                </div>
            </div>

            {if let Some(controller_id) = *selected_controller {
                let controller_type = controllers.iter()
                    .find(|c| c.id == controller_id)
                    .map(|c| c.controller_type.as_str())
                    .unwrap_or("unknown");

                html! {
                    <>
                        // Tab navigation based on controller type
                        <div class="border-b border-gray-200 mb-6">
                            <nav class="flex space-x-8">
                                {match controller_type {
                                    "unifi" => {
                                        for ["overview", "devices", "clients", "sites", "wireless"].iter().map(|tab| {
                                            let is_active = *active_tab == *tab;
                                            let active_tab = active_tab.clone();
                                            let tab_str = tab.to_string();
                                            html! {
                                                <button
                                                    onclick={Callback::from(move |_| active_tab.set(tab_str.clone()))}
                                                    class={format!(
                                                        "py-2 px-1 border-b-2 font-medium text-sm {}",
                                                        if is_active {
                                                            "border-purple-500 text-purple-600"
                                                        } else {
                                                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                                        }
                                                    )}
                                                >
                                                    {tab.to_uppercase()}
                                                </button>
                                            }
                                        }).collect::<Html>()
                                    },
                                    "fortigate" => {
                                        for ["overview", "interfaces", "policies", "vpn", "monitoring"].iter().map(|tab| {
                                            let is_active = *active_tab == *tab;
                                            let active_tab = active_tab.clone();
                                            let tab_str = tab.to_string();
                                            html! {
                                                <button
                                                    onclick={Callback::from(move |_| active_tab.set(tab_str.clone()))}
                                                    class={format!(
                                                        "py-2 px-1 border-b-2 font-medium text-sm {}",
                                                        if is_active {
                                                            "border-purple-500 text-purple-600"
                                                        } else {
                                                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                                        }
                                                    )}
                                                >
                                                    {tab.to_uppercase()}
                                                </button>
                                            }
                                        }).collect::<Html>()
                                    },
                                    "powerdns" | "cloudflare" => {
                                        for ["overview", "zones", "records", "analytics"].iter().map(|tab| {
                                            let is_active = *active_tab == *tab;
                                            let active_tab = active_tab.clone();
                                            let tab_str = tab.to_string();
                                            html! {
                                                <button
                                                    onclick={Callback::from(move |_| active_tab.set(tab_str.clone()))}
                                                    class={format!(
                                                        "py-2 px-1 border-b-2 font-medium text-sm {}",
                                                        if is_active {
                                                            "border-purple-500 text-purple-600"
                                                        } else {
                                                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                                        }
                                                    )}
                                                >
                                                    {tab.to_uppercase()}
                                                </button>
                                            }
                                        }).collect::<Html>()
                                    },
                                    _ => html! { <div>{"Unknown controller type"}</div> }
                                }}
                            </nav>
                        </div>

                        // Tab content
                        <div class="bg-white rounded-lg shadow">
                            {match (controller_type, active_tab.as_str()) {
                                ("unifi", "overview") => html! { <UnifiOverview {controller_id} /> },
                                ("unifi", "devices") => html! { <UnifiDevices {controller_id} /> },
                                ("fortigate", "overview") => html! { <FortigateOverview {controller_id} /> },
                                ("fortigate", "interfaces") => html! { <FortigateInterfaces {controller_id} /> },
                                ("powerdns" | "cloudflare", "overview") => html! { <DnsOverview {controller_id} /> },
                                ("powerdns" | "cloudflare", "records") => html! { <DnsRecords {controller_id} /> },
                                (_, _) => html! { <div class="p-6"><p class="text-gray-600">{"Feature coming soon..."}</p></div> }
                            }}
                        </div>
                    </>
                }
            } else if !(*loading) && controllers.is_empty() {
                html! {
                    <div class="bg-white rounded-lg shadow p-12 text-center">
                        <h3 class="text-lg font-medium text-gray-900 mb-2">{"No Network Controllers Configured"}</h3>
                        <p class="text-gray-600 mb-4">{"Add your first UniFi, FortiGate, or DNS controller to start monitoring"}</p>
                        <button 
                            onclick={
                                let show_add_controller = show_add_controller.clone();
                                Callback::from(move |_| show_add_controller.set(true))
                            }
                            class="px-6 py-3 bg-purple-600 text-white rounded-md hover:bg-purple-700"
                        >
                            {"Add First Controller"}
                        </button>
                    </div>
                }
            } else if !(*loading) {
                html! {
                    <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                        <p class="text-yellow-800">{"Please select a controller to manage network devices"}</p>
                    </div>
                }
            } else {
                html! {
                    <div class="flex justify-center items-center h-64">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
                    </div>
                }
            }}
        </div>
    }
}

// Component implementations for different controller types
#[derive(Properties, Clone, PartialEq)]
struct TabProps {
    controller_id: Uuid,
}

#[function_component(UnifiOverview)]
fn unifi_overview(props: &TabProps) -> Html {
    let controller = use_state(|| None::<NetworkController>);
    let devices = use_state(Vec::<UnifiDevice>::new);
    
    {
        let controller = controller.clone();
        let devices = devices.clone();
        let controller_id = props.controller_id;
        use_effect_with(controller_id, move |_| {
            let controller = controller.clone();
            let devices = devices.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // Fetch controller details
                if let Ok(resp) = Request::get(&format!("/api/v1/network/controllers/{}", controller_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<NetworkController>().await {
                            controller.set(Some(data));
                        }
                    }
                }

                // Fetch devices
                if let Ok(resp) = Request::get(&format!("/api/v1/network/controllers/{}/devices", controller_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<Vec<UnifiDevice>>().await {
                            devices.set(data);
                        }
                    }
                }
            });
            || ()
        });
    }

    if let Some(ctrl) = controller.as_ref() {
        let online_devices = devices.iter().filter(|d| d.status == "online").count();
        let total_clients = devices.iter().map(|d| d.clients_connected).sum::<i32>();
        
        html! {
            <div class="p-6">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    <div class="bg-green-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-green-800 mb-2">{"Controller Status"}</h3>
                        <div class="text-2xl font-bold text-green-600">{ctrl.status.to_uppercase()}</div>
                        <p class="text-sm text-green-600 mt-1">{&ctrl.host}</p>
                    </div>

                    <div class="bg-blue-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-blue-800 mb-2">{"Devices"}</h3>
                        <div class="text-2xl font-bold text-blue-600">{online_devices}</div>
                        <p class="text-sm text-blue-600 mt-1">{format!("of {} total", devices.len())}</p>
                    </div>

                    <div class="bg-purple-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-purple-800 mb-2">{"Sites"}</h3>
                        <div class="text-2xl font-bold text-purple-600">{ctrl.site_count}</div>
                        <p class="text-sm text-purple-600 mt-1">{"Managed sites"}</p>
                    </div>

                    <div class="bg-orange-50 p-4 rounded-lg">
                        <h3 class="text-sm font-medium text-orange-800 mb-2">{"Connected Clients"}</h3>
                        <div class="text-2xl font-bold text-orange-600">{total_clients}</div>
                        <p class="text-sm text-orange-600 mt-1">{"Active connections"}</p>
                    </div>
                </div>

                // Device types breakdown
                <div class="mb-8">
                    <h3 class="text-lg font-medium text-gray-900 mb-4">{"Device Types"}</h3>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        {
                            let mut device_types = std::collections::HashMap::new();
                            for device in devices.iter() {
                                *device_types.entry(&device.device_type).or_insert(0) += 1;
                            }
                            
                            device_types.into_iter().map(|(device_type, count)| {
                                let (color_class, icon) = match device_type {
                                    "ap" => ("bg-green-100 text-green-800", "📡"),
                                    "switch" => ("bg-blue-100 text-blue-800", "🔌"),
                                    "gateway" | "security_gateway" => ("bg-purple-100 text-purple-800", "🛡️"),
                                    _ => ("bg-gray-100 text-gray-800", "🔧"),
                                };
                                
                                html! {
                                    <div class="text-center p-4 border rounded-lg">
                                        <div class="text-2xl mb-2">{icon}</div>
                                        <div class="text-2xl font-bold text-gray-900">{count}</div>
                                        <div class={format!("text-xs px-2 py-1 rounded-full {}", color_class)}>
                                            {device_type.to_uppercase()}
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="flex justify-center items-center h-64">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
            </div>
        }
    }
}

#[function_component(UnifiDevices)]
fn unifi_devices(props: &TabProps) -> Html {
    let devices = use_state(Vec::<UnifiDevice>::new);
    let loading = use_state(|| true);
    
    {
        let devices = devices.clone();
        let loading = loading.clone();
        let controller_id = props.controller_id;
        use_effect_with(controller_id, move |_| {
            let devices = devices.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                if let Ok(resp) = Request::get(&format!("/api/v1/network/controllers/{}/devices", controller_id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<Vec<UnifiDevice>>().await {
                            devices.set(data);
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
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
                    </div>
                }
            } else if devices.is_empty() {
                html! {
                    <div class="text-center py-12">
                        <p class="text-gray-500">{"No devices found. Try syncing the controller."}</p>
                    </div>
                }
            } else {
                html! {
                    <div class="overflow-x-auto">
                        <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Device"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Type"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"IP Address"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Status"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Clients"}</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Uptime"}</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                {for devices.iter().map(|device| html! {
                                    <tr class="hover:bg-gray-50">
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div>
                                                <div class="text-sm font-medium text-gray-900">{&device.name}</div>
                                                <div class="text-sm text-gray-500">{&device.model}</div>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{&device.device_type}</td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                            {device.ip_address.as_ref().unwrap_or(&"—".to_string())}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            {match device.status.as_str() {
                                                "online" => html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">{"Online"}</span> },
                                                "offline" => html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800">{"Offline"}</span> },
                                                _ => html! { <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-yellow-100 text-yellow-800">{"Unknown"}</span> }
                                            }}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{device.clients_connected}</td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                            {device.uptime.map(|u| format!("{}d", u / 86400)).unwrap_or_else(|| "—".to_string())}
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

// Placeholder implementations for other controller types
#[function_component(FortigateOverview)]
fn fortigate_overview(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"FortiGate overview coming soon..."}</p>
        </div>
    }
}

#[function_component(FortigateInterfaces)]
fn fortigate_interfaces(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"FortiGate interfaces coming soon..."}</p>
        </div>
    }
}

#[function_component(DnsOverview)]
fn dns_overview(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"DNS overview coming soon..."}</p>
        </div>
    }
}

#[function_component(DnsRecords)]
fn dns_records(props: &TabProps) -> Html {
    html! {
        <div class="p-6">
            <p class="text-gray-600">{"DNS records management coming soon..."}</p>
        </div>
    }
}