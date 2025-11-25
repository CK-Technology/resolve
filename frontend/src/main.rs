use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
mod services;
mod theme;
mod utils;

use components::{layout::Layout, auth::{AuthProvider, LoginForm, AuthContext}};
use pages::{
    clients::ClientsPage,
    dashboard::DashboardPage,
    tickets::TicketsPage,
    time_tracking::TimeTrackingPage,
    passwords::PasswordsPage,
    assets::AssetsPage,
    invoices::InvoicesPage,
    knowledge_base::KnowledgeBasePage,
    reports::ReportsPage,
    admin::AdminPage,
    m365::M365Page,
    azure::AzurePage,
    bitwarden::BitwardenPage,
    network::NetworkPage,
};
use theme::ThemeProvider;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Dashboard,
    #[at("/login")]
    Login,
    #[at("/clients")]
    Clients,
    #[at("/tickets")]
    Tickets,
    #[at("/time")]
    TimeTracking,
    #[at("/passwords")]
    Passwords,
    #[at("/assets")]
    Assets,
    #[at("/invoices")]
    Invoices,
    #[at("/kb")]
    KnowledgeBase,
    #[at("/reports")]
    Reports,
    #[at("/admin")]
    Admin,
    #[at("/projects")]
    Projects,
    #[at("/m365")]
    M365,
    #[at("/azure")]
    Azure,
    #[at("/bitwarden")]
    Bitwarden,
    #[at("/network")]
    Network,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Dashboard => html! { <DashboardPage /> },
        Route::Login => html! { <LoginPage /> },
        Route::Clients => html! { <ClientsPage /> },
        Route::Tickets => html! { <TicketsPage /> },
        Route::TimeTracking => html! { <TimeTrackingPage /> },
        Route::Passwords => html! { <PasswordsPage /> },
        Route::Assets => html! { <AssetsPage /> },
        Route::Invoices => html! { <InvoicesPage /> },
        Route::KnowledgeBase => html! { <KnowledgeBasePage /> },
        Route::Reports => html! { <ReportsPage /> },
        Route::Admin => html! { <AdminPage /> },
        Route::Projects => html! { <div class="p-6" style="background-color: var(--bg-primary); min-height: 100vh;"><h1 class="text-2xl font-bold" style="color: var(--fg-primary);">{"Projects"}</h1><p style="color: var(--fg-muted);">{"Project management coming soon..."}</p></div> },
        Route::M365 => html! { <M365Page /> },
        Route::Azure => html! { <AzurePage /> },
        Route::Bitwarden => html! { <BitwardenPage /> },
        Route::Network => html! { <NetworkPage /> },
        Route::NotFound => html! {
            <div class="min-h-screen flex items-center justify-center" style="background-color: var(--bg-primary);">
                <div class="text-center">
                    <h1 class="text-6xl font-bold" style="color: var(--fg-primary);">{"404"}</h1>
                    <p class="text-xl mt-4" style="color: var(--fg-muted);">{"Page Not Found"}</p>
                </div>
            </div>
        },
    }
}

#[function_component(LoginPage)]
fn login_page() -> Html {
    let navigator = use_navigator().unwrap();
    
    let on_login = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Dashboard);
        })
    };
    
    html! {
        <LoginForm {on_login} />
    }
}

#[function_component(AppRouter)]
fn app_router() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthContext not found");
    
    // If not authenticated, show login page
    if auth_ctx.user.is_none() {
        return html! {
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        };
    }
    
    // If authenticated, show main app with layout
    html! {
        <BrowserRouter>
            <Layout>
                <Switch<Route> render={switch} />
            </Layout>
        </BrowserRouter>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <ThemeProvider>
            <AuthProvider>
                <AppRouter />
            </AuthProvider>
        </ThemeProvider>
    }
}

fn main() {
    let document = web_sys::window().unwrap().document().unwrap();
    let head = document.head().unwrap();

    // Load Tailwind CSS
    let tailwind = document.create_element("link").unwrap();
    tailwind.set_attribute("href", "https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css").unwrap();
    tailwind.set_attribute("rel", "stylesheet").unwrap();
    head.append_child(&tailwind).unwrap();

    // Load Google Fonts (Fira Code + JetBrains Mono)
    let fonts = document.create_element("link").unwrap();
    fonts.set_attribute("href", "https://fonts.googleapis.com/css2?family=Fira+Code:wght@300;400;500;600;700&family=JetBrains+Mono:wght@300;400;500;600;700&family=Inter:wght@300;400;500;600;700&display=swap").unwrap();
    fonts.set_attribute("rel", "stylesheet").unwrap();
    head.append_child(&fonts).unwrap();

    // Load our Tokyo Night theme CSS
    let theme_css = document.create_element("link").unwrap();
    theme_css.set_attribute("href", "/static/themes.css").unwrap();
    theme_css.set_attribute("rel", "stylesheet").unwrap();
    head.append_child(&theme_css).unwrap();

    // Apply initial theme
    theme::apply_theme(theme::load_theme());

    yew::Renderer::<App>::new().render();
}