use yew::prelude::*;
use crate::components::Dashboard;

#[function_component(DashboardPage)]
pub fn dashboard_page() -> Html {
    html! {
        <Dashboard />
    }
}