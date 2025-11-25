// Time Tracking Page - BMS-style full page time management

use yew::prelude::*;
use gloo_timers::callback::Interval;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct TimeEntry {
    pub id: String,
    pub ticket_id: Option<String>,
    pub ticket_subject: Option<String>,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub user_id: String,
    pub user_name: String,
    pub description: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_minutes: i32,
    pub billable: bool,
    pub billed: bool,
    pub hourly_rate: Option<f64>,
}

#[derive(Clone, PartialEq)]
enum TimerState {
    Idle,
    Running { start_time: f64, accumulated_seconds: i32 },
    Paused { accumulated_seconds: i32 },
}

#[function_component(TimeTrackingPage)]
pub fn time_tracking_page() -> Html {
    let timer_state = use_state(|| TimerState::Idle);
    let current_seconds = use_state(|| 0i32);
    let selected_ticket = use_state(|| None::<String>);
    let selected_client = use_state(|| None::<String>);
    let description = use_state(|| String::new());
    let is_billable = use_state(|| true);
    let time_entries = use_state(|| None::<Vec<TimeEntry>>);
    let show_manual_entry = use_state(|| false);
    let loading = use_state(|| true);

    // Timer interval for real-time display
    {
        let timer_state = timer_state.clone();
        let current_seconds = current_seconds.clone();

        use_effect_with((*timer_state).clone(), move |state| {
            if let TimerState::Running { start_time, accumulated_seconds } = state {
                let start = *start_time;
                let acc = *accumulated_seconds;
                let current_seconds = current_seconds.clone();

                let interval = Interval::new(1000, move || {
                    let now = js_sys::Date::new_0().get_time() / 1000.0;
                    let elapsed = (now - start) as i32 + acc;
                    current_seconds.set(elapsed);
                });

                return move || drop(interval);
            }
            || ()
        });
    }

    // Fetch time entries on mount
    {
        let time_entries = time_entries.clone();
        let loading = loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Mock data
                let mock_entries = vec![
                    TimeEntry {
                        id: "1".to_string(),
                        ticket_id: Some("T-1234".to_string()),
                        ticket_subject: Some("Network connectivity issue".to_string()),
                        client_id: Some("c1".to_string()),
                        client_name: Some("Acme Corp".to_string()),
                        user_id: "u1".to_string(),
                        user_name: "John Doe".to_string(),
                        description: "Diagnosed network latency, replaced faulty cable".to_string(),
                        start_time: "2024-02-20T09:00:00Z".to_string(),
                        end_time: Some("2024-02-20T10:30:00Z".to_string()),
                        duration_minutes: 90,
                        billable: true,
                        billed: false,
                        hourly_rate: Some(125.00),
                    },
                    TimeEntry {
                        id: "2".to_string(),
                        ticket_id: Some("T-1235".to_string()),
                        ticket_subject: Some("Email configuration".to_string()),
                        client_id: Some("c2".to_string()),
                        client_name: Some("TechStart Inc".to_string()),
                        user_id: "u1".to_string(),
                        user_name: "John Doe".to_string(),
                        description: "Set up email accounts for new employees".to_string(),
                        start_time: "2024-02-20T11:00:00Z".to_string(),
                        end_time: Some("2024-02-20T12:15:00Z".to_string()),
                        duration_minutes: 75,
                        billable: true,
                        billed: true,
                        hourly_rate: Some(125.00),
                    },
                    TimeEntry {
                        id: "3".to_string(),
                        ticket_id: None,
                        ticket_subject: None,
                        client_id: None,
                        client_name: None,
                        user_id: "u1".to_string(),
                        user_name: "John Doe".to_string(),
                        description: "Team meeting - weekly standup".to_string(),
                        start_time: "2024-02-20T14:00:00Z".to_string(),
                        end_time: Some("2024-02-20T14:30:00Z".to_string()),
                        duration_minutes: 30,
                        billable: false,
                        billed: false,
                        hourly_rate: None,
                    },
                ];

                time_entries.set(Some(mock_entries));
                loading.set(false);
            });
            || ()
        });
    }

    let start_timer = {
        let timer_state = timer_state.clone();
        let current_seconds = current_seconds.clone();
        Callback::from(move |_| {
            let now = js_sys::Date::new_0().get_time() / 1000.0;
            timer_state.set(TimerState::Running {
                start_time: now,
                accumulated_seconds: 0,
            });
            current_seconds.set(0);
        })
    };

    let pause_timer = {
        let timer_state = timer_state.clone();
        let current_seconds = current_seconds.clone();
        Callback::from(move |_| {
            timer_state.set(TimerState::Paused {
                accumulated_seconds: *current_seconds,
            });
        })
    };

    let resume_timer = {
        let timer_state = timer_state.clone();
        let current_seconds = current_seconds.clone();
        Callback::from(move |_| {
            let now = js_sys::Date::new_0().get_time() / 1000.0;
            if let TimerState::Paused { accumulated_seconds } = *timer_state {
                timer_state.set(TimerState::Running {
                    start_time: now,
                    accumulated_seconds,
                });
            }
        })
    };

    let stop_timer = {
        let timer_state = timer_state.clone();
        let current_seconds = current_seconds.clone();
        Callback::from(move |_| {
            // In real app, save the time entry here
            timer_state.set(TimerState::Idle);
            current_seconds.set(0);
        })
    };

    let adjust_time = |delta: i32| {
        let current_seconds = current_seconds.clone();
        let timer_state = timer_state.clone();
        Callback::from(move |_| {
            let new_seconds = (*current_seconds + delta).max(0);
            current_seconds.set(new_seconds);

            // Also update accumulated if paused
            if let TimerState::Paused { .. } = *timer_state {
                timer_state.set(TimerState::Paused {
                    accumulated_seconds: new_seconds,
                });
            }
        })
    };

    let on_description_input = {
        let description = description.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            description.set(input.value());
        })
    };

    let toggle_billable = {
        let is_billable = is_billable.clone();
        Callback::from(move |_| is_billable.set(!*is_billable))
    };

    // Calculate stats
    let stats = time_entries.as_ref().map(|entries| {
        let total_minutes: i32 = entries.iter().map(|e| e.duration_minutes).sum();
        let billable_minutes: i32 = entries.iter().filter(|e| e.billable).map(|e| e.duration_minutes).sum();
        let unbilled_amount: f64 = entries.iter()
            .filter(|e| e.billable && !e.billed)
            .map(|e| (e.duration_minutes as f64 / 60.0) * e.hourly_rate.unwrap_or(0.0))
            .sum();
        (total_minutes, billable_minutes, unbilled_amount)
    }).unwrap_or((0, 0, 0.0));

    let format_time = |seconds: i32| -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    };

    let format_duration = |minutes: i32| -> String {
        let hours = minutes / 60;
        let mins = minutes % 60;
        if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    };

    let is_running = matches!(*timer_state, TimerState::Running { .. });
    let is_paused = matches!(*timer_state, TimerState::Paused { .. });

    html! {
        <div class="p-6" style="background-color: var(--bg-primary); min-height: 100vh;">
            // Header
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold" style="color: var(--fg-primary);">{"Time Tracking"}</h1>
                    <p class="mt-1" style="color: var(--fg-muted);">{"Track your time, manage projects, and monitor productivity"}</p>
                </div>
                <button
                    onclick={Callback::from(move |_| {})}
                    class="flex items-center space-x-2 px-4 py-2 rounded-lg font-medium"
                    style="background-color: var(--button-secondary-bg); color: var(--fg-secondary);"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    <span>{"Manual Entry"}</span>
                </button>
            </div>

            // Main Timer Widget (BMS-style)
            <div class="rounded-lg p-6 mb-6" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <div class="flex items-center justify-between">
                    // Timer Display
                    <div class="flex items-center space-x-6">
                        // Big Timer Display
                        <div class="text-center">
                            <div
                                class="text-5xl font-mono font-bold tracking-wider"
                                style={if is_running {
                                    "color: var(--color-success-muted);"
                                } else if is_paused {
                                    "color: var(--color-warning);"
                                } else {
                                    "color: var(--fg-primary);"
                                }}
                            >
                                {format_time(*current_seconds)}
                            </div>
                            <div class="text-sm mt-1" style="color: var(--fg-muted);">
                                {if is_running { "Running" } else if is_paused { "Paused" } else { "Ready to start" }}
                            </div>
                        </div>

                        // Time Adjustment Buttons (visible when not idle)
                        if !matches!(*timer_state, TimerState::Idle) {
                            <div class="flex flex-col space-y-1">
                                <button
                                    onclick={adjust_time(300)}
                                    class="px-2 py-1 text-xs rounded"
                                    style="background-color: var(--bg-highlight); color: var(--fg-secondary);"
                                    title="Add 5 minutes"
                                >
                                    {"+5m"}
                                </button>
                                <button
                                    onclick={adjust_time(60)}
                                    class="px-2 py-1 text-xs rounded"
                                    style="background-color: var(--bg-highlight); color: var(--fg-secondary);"
                                    title="Add 1 minute"
                                >
                                    {"+1m"}
                                </button>
                                <button
                                    onclick={adjust_time(-60)}
                                    class="px-2 py-1 text-xs rounded"
                                    style="background-color: var(--bg-highlight); color: var(--fg-secondary);"
                                    title="Subtract 1 minute"
                                >
                                    {"-1m"}
                                </button>
                                <button
                                    onclick={adjust_time(-300)}
                                    class="px-2 py-1 text-xs rounded"
                                    style="background-color: var(--bg-highlight); color: var(--fg-secondary);"
                                    title="Subtract 5 minutes"
                                >
                                    {"-5m"}
                                </button>
                            </div>
                        }
                    </div>

                    // Controls
                    <div class="flex items-center space-x-4">
                        // Description Input
                        <div class="w-64">
                            <input
                                type="text"
                                placeholder="What are you working on?"
                                oninput={on_description_input}
                                value={(*description).clone()}
                                class="w-full px-4 py-2 rounded-lg"
                                style="background-color: var(--bg-input); border: 1px solid var(--border-primary); color: var(--fg-primary);"
                            />
                        </div>

                        // Billable Toggle
                        <button
                            onclick={toggle_billable}
                            class="flex items-center space-x-2 px-3 py-2 rounded-lg"
                            style={if *is_billable {
                                "background-color: var(--color-success); color: var(--bg-primary);"
                            } else {
                                "background-color: var(--bg-highlight); color: var(--fg-muted);"
                            }}
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                            </svg>
                            <span class="text-sm font-medium">{if *is_billable { "Billable" } else { "Non-billable" }}</span>
                        </button>

                        // Timer Control Buttons
                        <div class="flex items-center space-x-2">
                            {match *timer_state {
                                TimerState::Idle => html! {
                                    <button
                                        onclick={start_timer}
                                        class="flex items-center space-x-2 px-6 py-3 rounded-lg font-medium text-lg"
                                        style="background-color: var(--color-success); color: var(--bg-primary);"
                                    >
                                        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                            <path d="M8 5v14l11-7z"/>
                                        </svg>
                                        <span>{"Start"}</span>
                                    </button>
                                },
                                TimerState::Running { .. } => html! {
                                    <>
                                        <button
                                            onclick={pause_timer}
                                            class="flex items-center space-x-2 px-4 py-3 rounded-lg font-medium"
                                            style="background-color: var(--color-warning); color: var(--bg-primary);"
                                        >
                                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                                <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/>
                                            </svg>
                                            <span>{"Pause"}</span>
                                        </button>
                                        <button
                                            onclick={stop_timer.clone()}
                                            class="flex items-center space-x-2 px-4 py-3 rounded-lg font-medium"
                                            style="background-color: var(--color-error); color: white;"
                                        >
                                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                                <rect x="6" y="6" width="12" height="12"/>
                                            </svg>
                                            <span>{"Stop"}</span>
                                        </button>
                                    </>
                                },
                                TimerState::Paused { .. } => html! {
                                    <>
                                        <button
                                            onclick={resume_timer}
                                            class="flex items-center space-x-2 px-4 py-3 rounded-lg font-medium"
                                            style="background-color: var(--color-success); color: var(--bg-primary);"
                                        >
                                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                                <path d="M8 5v14l11-7z"/>
                                            </svg>
                                            <span>{"Resume"}</span>
                                        </button>
                                        <button
                                            onclick={stop_timer}
                                            class="flex items-center space-x-2 px-4 py-3 rounded-lg font-medium"
                                            style="background-color: var(--color-error); color: white;"
                                        >
                                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                                <rect x="6" y="6" width="12" height="12"/>
                                            </svg>
                                            <span>{"Stop"}</span>
                                        </button>
                                    </>
                                },
                            }}
                        </div>
                    </div>
                </div>
            </div>

            // Stats Cards
            <div class="grid grid-cols-4 gap-4 mb-6">
                <div class="rounded-lg p-4" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <div class="text-sm" style="color: var(--fg-muted);">{"Today"}</div>
                    <div class="text-2xl font-bold font-mono" style="color: var(--fg-primary);">
                        {format_duration(stats.0)}
                    </div>
                </div>
                <div class="rounded-lg p-4" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <div class="text-sm" style="color: var(--fg-muted);">{"Billable"}</div>
                    <div class="text-2xl font-bold font-mono" style="color: var(--color-success);">
                        {format_duration(stats.1)}
                    </div>
                </div>
                <div class="rounded-lg p-4" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <div class="text-sm" style="color: var(--fg-muted);">{"Unbilled"}</div>
                    <div class="text-2xl font-bold font-mono" style="color: var(--color-warning);">
                        {format!("${:.0}", stats.2)}
                    </div>
                </div>
                <div class="rounded-lg p-4" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                    <div class="text-sm" style="color: var(--fg-muted);">{"Entries"}</div>
                    <div class="text-2xl font-bold font-mono" style="color: var(--accent-primary);">
                        {time_entries.as_ref().map(|e| e.len()).unwrap_or(0)}
                    </div>
                </div>
            </div>

            // Recent Time Entries
            <div class="rounded-lg overflow-hidden" style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);">
                <div class="p-4 border-b" style="border-color: var(--border-primary);">
                    <h3 class="text-lg font-medium" style="color: var(--fg-primary);">{"Today's Entries"}</h3>
                </div>

                if *loading {
                    <div class="p-8 text-center" style="color: var(--fg-muted);">
                        {"Loading time entries..."}
                    </div>
                } else if let Some(entries) = time_entries.as_ref() {
                    <table class="w-full">
                        <thead>
                            <tr style="background-color: var(--bg-tertiary);">
                                <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Description"}</th>
                                <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Client / Ticket"}</th>
                                <th class="text-left py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Time"}</th>
                                <th class="text-right py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Duration"}</th>
                                <th class="text-center py-3 px-4 text-sm font-medium" style="color: var(--fg-muted);">{"Billable"}</th>
                                <th class="py-3 px-4"></th>
                            </tr>
                        </thead>
                        <tbody>
                            { for entries.iter().map(|entry| {
                                html! {
                                    <tr class="hover:bg-gray-700/30" style="border-bottom: 1px solid var(--border-primary);">
                                        <td class="py-3 px-4">
                                            <div class="font-medium" style="color: var(--fg-primary);">{&entry.description}</div>
                                        </td>
                                        <td class="py-3 px-4">
                                            if let Some(client) = &entry.client_name {
                                                <div class="text-sm" style="color: var(--fg-secondary);">{client}</div>
                                            }
                                            if let Some(ticket) = &entry.ticket_subject {
                                                <div class="text-xs" style="color: var(--fg-muted);">{ticket}</div>
                                            }
                                        </td>
                                        <td class="py-3 px-4 text-sm" style="color: var(--fg-secondary);">
                                            {"09:00 - 10:30"}
                                        </td>
                                        <td class="py-3 px-4 text-right font-mono" style="color: var(--fg-primary);">
                                            {format_duration(entry.duration_minutes)}
                                        </td>
                                        <td class="py-3 px-4 text-center">
                                            if entry.billable {
                                                <span class="px-2 py-1 text-xs rounded" style="background-color: var(--color-success); color: var(--bg-primary);">
                                                    {if entry.billed { "Billed" } else { "Billable" }}
                                                </span>
                                            } else {
                                                <span class="text-xs" style="color: var(--fg-muted);">{"Non-billable"}</span>
                                            }
                                        </td>
                                        <td class="py-3 px-4">
                                            <button class="p-1 rounded hover:bg-gray-700" style="color: var(--fg-muted);">
                                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                                                </svg>
                                            </button>
                                        </td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                } else {
                    <div class="p-8 text-center" style="color: var(--fg-muted);">
                        {"No time entries for today"}
                    </div>
                }
            </div>
        </div>
    }
}
