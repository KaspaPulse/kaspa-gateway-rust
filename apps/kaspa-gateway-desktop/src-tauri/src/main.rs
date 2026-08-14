#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if kaspa_gateway_desktop_lib::try_run_kgw_live_smoke_parent_from_args()
        || kaspa_gateway_desktop_lib::try_run_kgw_self_worker_from_args()
    {
        return;
    }

    kaspa_gateway_desktop_lib::run();
}
