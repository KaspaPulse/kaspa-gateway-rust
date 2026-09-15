const fs = require("node:fs");
const path = require("node:path");
const root = path.resolve(__dirname, "..");
const settings = fs.readFileSync(path.join(root, "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js"), "utf8");
const backend = fs.readFileSync(path.join(root, "apps/kaspa-gateway-desktop/src-tauri/src/settings_commands.rs"), "utf8");
const addressBook = fs.readFileSync(path.join(root, "apps/kaspa-gateway-desktop/src-tauri/src/address_book.rs"), "utf8");
function check(condition, message) { if (!condition) throw new Error(message); }
const placeholder = settings.match(/const placeholderActions = \[[\s\S]*?\];/)?.[0] || "";
for (const id of ["settingsProfileAdd", "settingsProfileRename", "settingsProfileDelete", "settingsResetSelectedEndpoint", "settingsExportAddresses", "settingsImportAddresses"]) {
  check(!placeholder.includes(id), `${id} must not remain placeholder-owned`);
}
for (const command of ["settings_profile_add", "settings_profile_rename", "settings_profile_delete", "settings_profile_select", "settings_reset_selected_endpoint", "address_book_export_json", "address_book_import_json"]) {
  check(settings.includes(command), `frontend missing real command ${command}`);
}
check(settings.includes("dialog.save") && settings.includes("dialog.open"), "native dialog open/save flows must be present");
check(settings.includes("kgwSettingsUpdateEndpointResetAvailability"), "reset availability must be explicit");
for (const fn of ["settings_profile_add", "settings_profile_rename", "settings_profile_delete", "settings_profile_select", "settings_reset_selected_endpoint"]) {
  check(backend.includes(`pub fn ${fn}`), `backend missing ${fn}`);
}
check(addressBook.includes("import will not overwrite it"), "address import must preserve conflicting saved data");
console.log("kgw_settings_workflow_tests: PASS");
