// Knowledge Base Page

use yew::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct KbArticle {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub folder_id: Option<String>,
    pub folder_name: Option<String>,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub is_global: bool,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    pub views: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct KbFolder {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub article_count: u32,
}

#[function_component(KnowledgeBasePage)]
pub fn knowledge_base_page() -> Html {
    let articles = use_state(|| None::<Vec<KbArticle>>);
    let folders = use_state(|| None::<Vec<KbFolder>>);
    let selected_folder = use_state(|| None::<String>);
    let search_query = use_state(|| String::new());
    let loading = use_state(|| true);
    let show_global = use_state(|| true);

    // Fetch data on mount
    {
        let articles = articles.clone();
        let folders = folders.clone();
        let loading = loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let mock_folders = vec![
                    KbFolder { id: "1".to_string(), name: "Getting Started".to_string(), icon: Some("rocket".to_string()), article_count: 5 },
                    KbFolder { id: "2".to_string(), name: "Network Documentation".to_string(), icon: Some("network".to_string()), article_count: 12 },
                    KbFolder { id: "3".to_string(), name: "Server Setup".to_string(), icon: Some("server".to_string()), article_count: 8 },
                    KbFolder { id: "4".to_string(), name: "Security Policies".to_string(), icon: Some("shield".to_string()), article_count: 6 },
                    KbFolder { id: "5".to_string(), name: "Troubleshooting".to_string(), icon: Some("wrench".to_string()), article_count: 15 },
                ];

                let mock_articles = vec![
                    KbArticle {
                        id: "1".to_string(),
                        title: "VPN Setup Guide".to_string(),
                        slug: "vpn-setup-guide".to_string(),
                        content: "# VPN Setup Guide\n\nThis guide covers...".to_string(),
                        folder_id: Some("2".to_string()),
                        folder_name: Some("Network Documentation".to_string()),
                        client_id: None,
                        client_name: None,
                        is_global: true,
                        author: "John Doe".to_string(),
                        created_at: "2024-01-15".to_string(),
                        updated_at: "2024-02-10".to_string(),
                        views: 245,
                    },
                    KbArticle {
                        id: "2".to_string(),
                        title: "Office 365 Migration Checklist".to_string(),
                        slug: "o365-migration-checklist".to_string(),
                        content: "# Office 365 Migration\n\n## Pre-migration tasks...".to_string(),
                        folder_id: Some("1".to_string()),
                        folder_name: Some("Getting Started".to_string()),
                        client_id: None,
                        client_name: None,
                        is_global: true,
                        author: "Jane Smith".to_string(),
                        created_at: "2024-01-20".to_string(),
                        updated_at: "2024-01-20".to_string(),
                        views: 189,
                    },
                    KbArticle {
                        id: "3".to_string(),
                        title: "Windows Server Backup Procedures".to_string(),
                        slug: "windows-server-backup".to_string(),
                        content: "# Backup Procedures\n\n...".to_string(),
                        folder_id: Some("3".to_string()),
                        folder_name: Some("Server Setup".to_string()),
                        client_id: None,
                        client_name: None,
                        is_global: true,
                        author: "Bob Wilson".to_string(),
                        created_at: "2024-02-01".to_string(),
                        updated_at: "2024-02-15".to_string(),
                        views: 156,
                    },
                    KbArticle {
                        id: "4".to_string(),
                        title: "Acme Corp - Network Diagram".to_string(),
                        slug: "acme-network-diagram".to_string(),
                        content: "# Acme Corp Network\n\n...".to_string(),
                        folder_id: Some("2".to_string()),
                        folder_name: Some("Network Documentation".to_string()),
                        client_id: Some("c1".to_string()),
                        client_name: Some("Acme Corp".to_string()),
                        is_global: false,
                        author: "John Doe".to_string(),
                        created_at: "2024-01-10".to_string(),
                        updated_at: "2024-02-20".to_string(),
                        views: 42,
                    },
                ];

                folders.set(Some(mock_folders));
                articles.set(Some(mock_articles));
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

    let select_folder = |folder_id: Option<String>| {
        let selected_folder = selected_folder.clone();
        Callback::from(move |_| {
            selected_folder.set(folder_id.clone());
        })
    };

    // Filter articles
    let filtered_articles = articles.as_ref().map(|list| {
        let query = search_query.to_lowercase();
        list.iter()
            .filter(|a| {
                let folder_match = selected_folder.as_ref().map(|f| a.folder_id.as_ref() == Some(f)).unwrap_or(true);
                let global_match = *show_global || !a.is_global;
                let search_match = query.is_empty() || a.title.to_lowercase().contains(&query);
                folder_match && global_match && search_match
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    html! {
        <div class="flex h-full" style="background-color: var(--bg-primary);">
            // Left Sidebar - Folders
            <div class="w-64 flex-shrink-0 border-r flex flex-col" style="border-color: var(--border-primary); background-color: var(--bg-secondary);">
                <div class="p-4 border-b" style="border-color: var(--border-primary);">
                    <h2 class="font-semibold" style="color: var(--fg-primary);">{"Knowledge Base"}</h2>
                </div>

                <div class="p-4">
                    <button
                        class="w-full flex items-center space-x-2 px-3 py-2 rounded-lg font-medium text-sm"
                        style="background-color: var(--button-primary-bg); color: var(--button-primary-text);"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                        </svg>
                        <span>{"New Article"}</span>
                    </button>
                </div>

                <nav class="flex-1 overflow-y-auto p-2">
                    // All Articles
                    <button
                        onclick={select_folder(None)}
                        class="w-full flex items-center justify-between px-3 py-2 rounded-lg mb-1"
                        style={if selected_folder.is_none() { "background-color: var(--bg-highlight); color: var(--fg-primary);" } else { "color: var(--fg-secondary);" }}
                    >
                        <div class="flex items-center space-x-2">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/>
                            </svg>
                            <span>{"All Articles"}</span>
                        </div>
                        <span class="text-xs px-2 py-0.5 rounded" style="background-color: var(--bg-highlight); color: var(--fg-muted);">
                            {articles.as_ref().map(|a| a.len()).unwrap_or(0)}
                        </span>
                    </button>

                    // Folders
                    if let Some(folders) = folders.as_ref() {
                        { for folders.iter().map(|folder| {
                            let folder_id = folder.id.clone();
                            let is_selected = selected_folder.as_ref() == Some(&folder.id);

                            html! {
                                <button
                                    onclick={select_folder(Some(folder_id))}
                                    class="w-full flex items-center justify-between px-3 py-2 rounded-lg mb-1"
                                    style={if is_selected { "background-color: var(--bg-highlight); color: var(--fg-primary);" } else { "color: var(--fg-secondary);" }}
                                >
                                    <div class="flex items-center space-x-2">
                                        <svg class="w-4 h-4" style="color: var(--accent-primary);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
                                        </svg>
                                        <span class="text-sm">{&folder.name}</span>
                                    </div>
                                    <span class="text-xs px-2 py-0.5 rounded" style="background-color: var(--bg-highlight); color: var(--fg-muted);">
                                        {folder.article_count}
                                    </span>
                                </button>
                            }
                        })}
                    }
                </nav>
            </div>

            // Main Content
            <div class="flex-1 overflow-y-auto">
                <div class="p-6">
                    // Search Bar
                    <div class="flex items-center space-x-4 mb-6">
                        <div class="flex-1 relative">
                            <svg class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4" style="color: var(--fg-muted);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                            </svg>
                            <input
                                type="text"
                                placeholder="Search articles..."
                                oninput={on_search}
                                class="w-full pl-10 pr-4 py-2 rounded-lg"
                                style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                            />
                        </div>
                        <label class="flex items-center space-x-2 cursor-pointer">
                            <input
                                type="checkbox"
                                checked={*show_global}
                                onchange={Callback::from(move |_| {})}
                                class="rounded"
                            />
                            <span class="text-sm" style="color: var(--fg-secondary);">{"Show global articles"}</span>
                        </label>
                    </div>

                    // Articles Grid
                    if *loading {
                        <div class="text-center py-12" style="color: var(--fg-muted);">
                            {"Loading articles..."}
                        </div>
                    } else if let Some(articles) = &filtered_articles {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            { for articles.iter().map(|article| {
                                html! {
                                    <ArticleCard article={article.clone()} />
                                }
                            })}
                        </div>
                    } else {
                        <div class="text-center py-12" style="color: var(--fg-muted);">
                            {"No articles found"}
                        </div>
                    }
                </div>
            </div>
        </div>
    }
}

// ===== Article Card Component =====

#[derive(Properties, PartialEq)]
struct ArticleCardProps {
    article: KbArticle,
}

#[function_component(ArticleCard)]
fn article_card(props: &ArticleCardProps) -> Html {
    html! {
        <div
            class="rounded-lg p-4 cursor-pointer hover:shadow-lg transition-all"
            style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
        >
            <div class="flex items-start justify-between mb-3">
                <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background-color: var(--bg-highlight);">
                    <svg class="w-5 h-5" style="color: var(--accent-primary);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                    </svg>
                </div>
                if props.article.is_global {
                    <span class="px-2 py-0.5 text-xs rounded" style="background-color: var(--accent-blue-dark); color: white;">
                        {"Global"}
                    </span>
                } else if let Some(client) = &props.article.client_name {
                    <span class="px-2 py-0.5 text-xs rounded" style="background-color: var(--color-success); color: var(--bg-primary);">
                        {client}
                    </span>
                }
            </div>

            <h3 class="font-medium mb-2 line-clamp-2" style="color: var(--fg-primary);">
                {&props.article.title}
            </h3>

            if let Some(folder) = &props.article.folder_name {
                <div class="flex items-center space-x-1 text-xs mb-3" style="color: var(--fg-muted);">
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
                    </svg>
                    <span>{folder}</span>
                </div>
            }

            <div class="flex items-center justify-between text-xs" style="color: var(--fg-dimmed);">
                <span>{format!("Updated {}", &props.article.updated_at)}</span>
                <div class="flex items-center space-x-1">
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
                    </svg>
                    <span>{props.article.views}</span>
                </div>
            </div>
        </div>
    }
}
