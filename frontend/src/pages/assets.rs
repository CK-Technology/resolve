// Assets Page - IT Asset Management

use yew::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub status: String,
    pub location: Option<String>,
    pub assigned_to: Option<String>,
    pub purchase_date: Option<String>,
    pub warranty_expiry: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum AssetView {
    List,
    Grid,
}

#[function_component(AssetsPage)]
pub fn assets_page() -> Html {
    let assets = use_state(|| None::<Vec<Asset>>);
    let selected_asset = use_state(|| None::<Asset>);
    let view_mode = use_state(|| AssetView::List);
    let search_query = use_state(|| String::new());
    let filter_type = use_state(|| None::<String>);
    let loading = use_state(|| true);

    // Fetch assets on mount
    {
        let assets = assets.clone();
        let loading = loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Mock data
                let mock_assets = vec![
                    Asset {
                        id: "1".to_string(),
                        name: "Dell OptiPlex 7090".to_string(),
                        asset_type: "Workstation".to_string(),
                        manufacturer: Some("Dell".to_string()),
                        model: Some("OptiPlex 7090".to_string()),
                        serial_number: Some("DELL-7090-001".to_string()),
                        client_id: Some("c1".to_string()),
                        client_name: Some("Acme Corp".to_string()),
                        status: "Active".to_string(),
                        location: Some("Office A - Desk 12".to_string()),
                        assigned_to: Some("John Smith".to_string()),
                        purchase_date: Some("2023-06-15".to_string()),
                        warranty_expiry: Some("2026-06-15".to_string()),
                        notes: Some("Primary workstation for accounting dept".to_string()),
                    },
                    Asset {
                        id: "2".to_string(),
                        name: "HP ProBook 450 G8".to_string(),
                        asset_type: "Laptop".to_string(),
                        manufacturer: Some("HP".to_string()),
                        model: Some("ProBook 450 G8".to_string()),
                        serial_number: Some("HP-450G8-002".to_string()),
                        client_id: Some("c1".to_string()),
                        client_name: Some("Acme Corp".to_string()),
                        status: "Active".to_string(),
                        location: Some("Mobile".to_string()),
                        assigned_to: Some("Jane Doe".to_string()),
                        purchase_date: Some("2023-09-20".to_string()),
                        warranty_expiry: Some("2026-09-20".to_string()),
                        notes: None,
                    },
                    Asset {
                        id: "3".to_string(),
                        name: "Cisco Catalyst 2960".to_string(),
                        asset_type: "Network Switch".to_string(),
                        manufacturer: Some("Cisco".to_string()),
                        model: Some("Catalyst 2960-24".to_string()),
                        serial_number: Some("CISCO-2960-003".to_string()),
                        client_id: Some("c2".to_string()),
                        client_name: Some("TechStart Inc".to_string()),
                        status: "Active".to_string(),
                        location: Some("Server Room - Rack A".to_string()),
                        assigned_to: None,
                        purchase_date: Some("2022-01-10".to_string()),
                        warranty_expiry: Some("2025-01-10".to_string()),
                        notes: Some("Core switch for main office".to_string()),
                    },
                    Asset {
                        id: "4".to_string(),
                        name: "Synology DS920+".to_string(),
                        asset_type: "NAS".to_string(),
                        manufacturer: Some("Synology".to_string()),
                        model: Some("DS920+".to_string()),
                        serial_number: Some("SYN-920-004".to_string()),
                        client_id: Some("c2".to_string()),
                        client_name: Some("TechStart Inc".to_string()),
                        status: "Active".to_string(),
                        location: Some("Server Room - Rack B".to_string()),
                        assigned_to: None,
                        purchase_date: Some("2023-03-05".to_string()),
                        warranty_expiry: Some("2026-03-05".to_string()),
                        notes: Some("File storage and backup".to_string()),
                    },
                    Asset {
                        id: "5".to_string(),
                        name: "Dell PowerEdge R740".to_string(),
                        asset_type: "Server".to_string(),
                        manufacturer: Some("Dell".to_string()),
                        model: Some("PowerEdge R740".to_string()),
                        serial_number: Some("DELL-R740-005".to_string()),
                        client_id: Some("c1".to_string()),
                        client_name: Some("Acme Corp".to_string()),
                        status: "Active".to_string(),
                        location: Some("Server Room".to_string()),
                        assigned_to: None,
                        purchase_date: Some("2022-08-01".to_string()),
                        warranty_expiry: Some("2027-08-01".to_string()),
                        notes: Some("Hyper-V host".to_string()),
                    },
                ];

                assets.set(Some(mock_assets));
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

    let toggle_view = {
        let view_mode = view_mode.clone();
        Callback::from(move |_| {
            view_mode.set(match *view_mode {
                AssetView::List => AssetView::Grid,
                AssetView::Grid => AssetView::List,
            });
        })
    };

    // Filter assets
    let filtered_assets = assets.as_ref().map(|list| {
        let query = search_query.to_lowercase();
        list.iter()
            .filter(|a| {
                let type_match = filter_type.as_ref().map(|t| &a.asset_type == t).unwrap_or(true);
                let search_match = query.is_empty()
                    || a.name.to_lowercase().contains(&query)
                    || a.serial_number.as_ref().map(|s| s.to_lowercase().contains(&query)).unwrap_or(false)
                    || a.client_name.as_ref().map(|c| c.to_lowercase().contains(&query)).unwrap_or(false);
                type_match && search_match
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    let asset_types = vec!["Workstation", "Laptop", "Server", "Network Switch", "NAS", "Firewall", "Printer", "Phone"];

    html! {
        <div class="p-6" style="background-color: var(--bg-primary); min-height: 100vh;">
            // Header
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold" style="color: var(--fg-primary);">{"Assets"}</h1>
                    <p class="mt-1" style="color: var(--fg-muted);">{"Manage IT assets across all clients"}</p>
                </div>
                <button
                    class="flex items-center space-x-2 px-4 py-2 rounded-lg font-medium"
                    style="background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    <span>{"Add Asset"}</span>
                </button>
            </div>

            // Filters and Search Bar
            <div class="flex items-center space-x-4 mb-6">
                // Search
                <div class="flex-1 relative">
                    <svg class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                    </svg>
                    <input
                        type="text"
                        placeholder="Search assets by name, serial, or client..."
                        oninput={on_search}
                        class="w-full pl-10 pr-4 py-2 rounded-lg"
                        style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                    />
                </div>

                // Type Filter
                <select
                    class="px-4 py-2 rounded-lg"
                    style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                >
                    <option value="">{"All Types"}</option>
                    { for asset_types.iter().map(|t| html! { <option value={*t}>{t}</option> })}
                </select>

                // View Toggle
                <div class="flex items-center rounded-lg overflow-hidden" style="border: 1px solid var(--border-primary);">
                    <button
                        onclick={toggle_view.clone()}
                        class="px-3 py-2"
                        style={if *view_mode == AssetView::List { "background-color: var(--bg-highlight); color: var(--fg-primary);" } else { "background-color: var(--bg-input); color: var(--fg-muted);" }}
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16"/>
                        </svg>
                    </button>
                    <button
                        onclick={toggle_view}
                        class="px-3 py-2"
                        style={if *view_mode == AssetView::Grid { "background-color: var(--bg-highlight); color: var(--fg-primary);" } else { "background-color: var(--bg-input); color: var(--fg-muted);" }}
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zM14 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"/>
                        </svg>
                    </button>
                </div>
            </div>

            // Asset Count
            <div class="mb-4 text-sm" style="color: var(--fg-muted);">
                {format!("Showing {} assets", filtered_assets.as_ref().map(|a| a.len()).unwrap_or(0))}
            </div>

            // Asset List/Grid
            if *loading {
                <div class="text-center py-12" style="color: var(--fg-muted);">
                    {"Loading assets..."}
                </div>
            } else if let Some(assets) = &filtered_assets {
                if *view_mode == AssetView::List {
                    <AssetTable assets={assets.clone()} />
                } else {
                    <AssetGrid assets={assets.clone()} />
                }
            } else {
                <div class="text-center py-12" style="color: var(--fg-muted);">
                    {"No assets found"}
                </div>
            }
        </div>
    }
}

// ===== Asset Table Component =====

#[derive(Properties, PartialEq)]
struct AssetTableProps {
    assets: Vec<Asset>,
}

#[function_component(AssetTable)]
fn asset_table(props: &AssetTableProps) -> Html {
    html! {
        <div class="rounded-lg overflow-hidden" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
            <table class="w-full">
                <thead>
                    <tr style="background-color: var(--bg-tertiary);">
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Name"}</th>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Type"}</th>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Client"}</th>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Serial"}</th>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Status"}</th>
                        <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Warranty"}</th>
                        <th class="py-3 px-4"></th>
                    </tr>
                </thead>
                <tbody>
                    { for props.assets.iter().map(|asset| {
                        let warranty_status = asset.warranty_expiry.as_ref().map(|_| {
                            // In a real app, compare with current date
                            ("valid", "var(--color-success)")
                        }).unwrap_or(("unknown", "var(--fg-muted)"));

                        html! {
                            <tr class="hover:bg-gray-700/30" style="border-bottom: 1px solid var(--border-primary);">
                                <td class="py-3 px-4">
                                    <div class="flex items-center space-x-3">
                                        <div class="w-8 h-8 rounded flex items-center justify-center" style="background-color: var(--bg-highlight);">
                                            <AssetIcon asset_type={asset.asset_type.clone()} />
                                        </div>
                                        <div>
                                            <div class="font-medium" style="color: var(--fg-primary);">{&asset.name}</div>
                                            <div class="text-xs" style="color: var(--fg-muted);">
                                                {asset.manufacturer.as_deref().unwrap_or("-")}
                                            </div>
                                        </div>
                                    </div>
                                </td>
                                <td class="py-3 px-4" style="color: var(--fg-secondary);">{&asset.asset_type}</td>
                                <td class="py-3 px-4" style="color: var(--fg-secondary);">
                                    {asset.client_name.as_deref().unwrap_or("-")}
                                </td>
                                <td class="py-3 px-4 font-mono text-sm" style="color: var(--fg-muted);">
                                    {asset.serial_number.as_deref().unwrap_or("-")}
                                </td>
                                <td class="py-3 px-4">
                                    <span
                                        class="px-2 py-1 text-xs rounded"
                                        style="background-color: var(--color-success); color: var(--bg-primary);"
                                    >
                                        {&asset.status}
                                    </span>
                                </td>
                                <td class="py-3 px-4 text-sm" style={format!("color: {}", warranty_status.1)}>
                                    {asset.warranty_expiry.as_deref().unwrap_or("-")}
                                </td>
                                <td class="py-3 px-4">
                                    <button class="p-1 rounded hover:bg-gray-700" style="color: var(--fg-muted);">
                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"/>
                                        </svg>
                                    </button>
                                </td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        </div>
    }
}

// ===== Asset Grid Component =====

#[derive(Properties, PartialEq)]
struct AssetGridProps {
    assets: Vec<Asset>,
}

#[function_component(AssetGrid)]
fn asset_grid(props: &AssetGridProps) -> Html {
    html! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            { for props.assets.iter().map(|asset| {
                html! {
                    <div
                        class="rounded-lg p-4 hover:shadow-lg transition-shadow cursor-pointer"
                        style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
                    >
                        <div class="flex items-start justify-between mb-3">
                            <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background-color: var(--bg-highlight);">
                                <AssetIcon asset_type={asset.asset_type.clone()} />
                            </div>
                            <span
                                class="px-2 py-1 text-xs rounded"
                                style="background-color: var(--color-success); color: var(--bg-primary);"
                            >
                                {&asset.status}
                            </span>
                        </div>

                        <h3 class="font-medium mb-1" style="color: var(--fg-primary);">{&asset.name}</h3>
                        <p class="text-sm mb-2" style="color: var(--fg-muted);">{&asset.asset_type}</p>

                        <div class="space-y-1 text-xs" style="color: var(--fg-dimmed);">
                            <div class="flex justify-between">
                                <span>{"Serial:"}</span>
                                <span class="font-mono">{asset.serial_number.as_deref().unwrap_or("-")}</span>
                            </div>
                            <div class="flex justify-between">
                                <span>{"Client:"}</span>
                                <span>{asset.client_name.as_deref().unwrap_or("-")}</span>
                            </div>
                            if let Some(assigned) = &asset.assigned_to {
                                <div class="flex justify-between">
                                    <span>{"Assigned:"}</span>
                                    <span>{assigned}</span>
                                </div>
                            }
                        </div>
                    </div>
                }
            })}
        </div>
    }
}

// ===== Asset Icon Component =====

#[derive(Properties, PartialEq)]
struct AssetIconProps {
    asset_type: String,
}

#[function_component(AssetIcon)]
fn asset_icon(props: &AssetIconProps) -> Html {
    let icon = match props.asset_type.as_str() {
        "Workstation" | "Laptop" => html! {
            <svg class="w-5 h-5" style="color: var(--accent-primary);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
            </svg>
        },
        "Server" => html! {
            <svg class="w-5 h-5" style="color: var(--color-warning);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"/>
            </svg>
        },
        "Network Switch" | "Firewall" => html! {
            <svg class="w-5 h-5" style="color: var(--color-success-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"/>
            </svg>
        },
        "NAS" => html! {
            <svg class="w-5 h-5" style="color: var(--syntax-keyword);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"/>
            </svg>
        },
        _ => html! {
            <svg class="w-5 h-5" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"/>
            </svg>
        },
    };

    icon
}
