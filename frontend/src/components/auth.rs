use yew::prelude::*;
use yew_hooks::use_async;
use gloo_net::http::Request;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use resolve_shared::User;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Properties, PartialEq)]
pub struct LoginFormProps {
    pub on_login: Callback<AuthResponse>,
}

#[function_component(LoginForm)]
pub fn login_form(props: &LoginFormProps) -> Html {
    let email = use_state(String::new);
    let password = use_state(String::new);
    let error_message = use_state(|| None::<String>);
    let loading = use_state(|| false);
    
    let on_login = props.on_login.clone();
    let email_clone = email.clone();
    let password_clone = password.clone();
    let error_clone = error_message.clone();
    let loading_clone = loading.clone();
    
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        
        let email = (*email_clone).clone();
        let password = (*password_clone).clone();
        let on_login = on_login.clone();
        let error_message = error_clone.clone();
        let loading = loading_clone.clone();
        
        if email.is_empty() || password.is_empty() {
            error_message.set(Some("Please fill in all fields".to_string()));
            return;
        }
        
        loading.set(true);
        error_message.set(None);
        
        spawn_local(async move {
            let login_request = LoginRequest { email, password };
            
            match Request::post("http://localhost:8080/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .json(&login_request)
                .unwrap()
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<AuthResponse>().await {
                            Ok(auth_response) => {
                                // Store token in local storage
                                let _ = LocalStorage::set("auth_token", &auth_response.token);
                                let _ = LocalStorage::set("user", &auth_response.user);
                                loading.set(false);
                                on_login.emit(auth_response);
                            }
                            Err(e) => {
                                loading.set(false);
                                error_message.set(Some(format!("Failed to parse response: {}", e)));
                            }
                        }
                    } else {
                        loading.set(false);
                        error_message.set(Some("Invalid email or password".to_string()));
                    }
                }
                Err(e) => {
                    loading.set(false);
                    error_message.set(Some(format!("Network error: {}", e)));
                }
            }
        });
    });
    
    let email_oninput = {
        let email = email.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            email.set(input.value());
        })
    };
    
    let password_oninput = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };
    
    html! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                <div>
                    <div class="mx-auto h-12 w-auto">
                        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
                            {"Sign in to Resolve"}
                        </h2>
                        <p class="mt-2 text-center text-sm text-gray-600">
                            {"MSP Management Platform"}
                        </p>
                    </div>
                </div>
                
                <form class="mt-8 space-y-6" {onsubmit}>
                    <div class="rounded-md shadow-sm -space-y-px">
                        <div>
                            <label for="email-address" class="sr-only">{"Email address"}</label>
                            <input
                                id="email-address"
                                name="email"
                                type="email"
                                autocomplete="email"
                                required=true
                                class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                                placeholder="Email address"
                                value={(*email).clone()}
                                oninput={email_oninput}
                            />
                        </div>
                        <div>
                            <label for="password" class="sr-only">{"Password"}</label>
                            <input
                                id="password"
                                name="password"
                                type="password"
                                autocomplete="current-password"
                                required=true
                                class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                                placeholder="Password"
                                value={(*password).clone()}
                                oninput={password_oninput}
                            />
                        </div>
                    </div>
                    
                    if let Some(error) = (*error_message).clone() {
                        <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded relative">
                            {error}
                        </div>
                    }
                    
                    <div>
                        <button
                            type="submit"
                            disabled={*loading}
                            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            if *loading {
                                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                                {"Signing in..."}
                            } else {
                                {"Sign in"}
                            }
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

// Auth context for managing authentication state across the app
#[derive(Clone, Debug, PartialEq)]
pub struct AuthContext {
    pub user: Option<User>,
    pub token: Option<String>,
    pub login: Callback<AuthResponse>,
    pub logout: Callback<()>,
}

impl Default for AuthContext {
    fn default() -> Self {
        Self {
            user: None,
            token: None,
            login: Callback::noop(),
            logout: Callback::noop(),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct AuthProviderProps {
    pub children: Children,
}

#[function_component(AuthProvider)]
pub fn auth_provider(props: &AuthProviderProps) -> Html {
    let auth_state = use_state(|| {
        // Try to restore auth from localStorage
        let token: Option<String> = LocalStorage::get("auth_token").ok();
        let user: Option<User> = LocalStorage::get("user").ok();
        
        (user, token)
    });
    
    let login = {
        let auth_state = auth_state.clone();
        Callback::from(move |auth_response: AuthResponse| {
            auth_state.set((Some(auth_response.user), Some(auth_response.token)));
        })
    };
    
    let logout = {
        let auth_state = auth_state.clone();
        Callback::from(move |_| {
            let _ = LocalStorage::delete("auth_token");
            let _ = LocalStorage::delete("user");
            auth_state.set((None, None));
        })
    };
    
    let context = AuthContext {
        user: auth_state.0.clone(),
        token: auth_state.1.clone(),
        login,
        logout,
    };
    
    html! {
        <ContextProvider<AuthContext> {context}>
            {props.children.clone()}
        </ContextProvider<AuthContext>>
    }
}