use wasm_bindgen::prelude::*;

/// Connect to Phantom wallet, returns public key string
pub async fn connect_wallet() -> Result<String, String> {
    let result = js_connect_wallet()
        .await
        .map_err(|e| format!("{:?}", e))?;
    result
        .as_string()
        .ok_or_else(|| "Failed to get public key from wallet".to_string())
}

/// Disconnect from Phantom wallet
pub async fn disconnect_wallet() -> Result<(), String> {
    js_disconnect_wallet()
        .await
        .map_err(|e| format!("{:?}", e))?;
    Ok(())
}

/// Check if Phantom wallet is connected
pub fn is_connected() -> bool {
    js_is_connected()
}

/// Get connected public key
pub fn get_public_key() -> Option<String> {
    let val = js_get_public_key();
    val.as_string()
}

// JS bridge functions
#[wasm_bindgen(inline_js = "
export function js_connect_wallet() {
    return window.solanaBridge.connectWallet();
}

export function js_disconnect_wallet() {
    return window.solanaBridge.disconnectWallet();
}

export function js_is_connected() {
    return window.solanaBridge.isConnected();
}

export function js_get_public_key() {
    return window.solanaBridge.getPublicKey();
}

export function js_build_and_send_tx(instructions_json, signers_json) {
    return window.solanaBridge.buildAndSendTransaction(instructions_json, signers_json);
}
")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn js_connect_wallet() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_disconnect_wallet() -> Result<JsValue, JsValue>;

    fn js_is_connected() -> bool;

    fn js_get_public_key() -> JsValue;

    #[wasm_bindgen(catch)]
    pub async fn js_build_and_send_tx(
        instructions_json: &str,
        signers_json: &str,
    ) -> Result<JsValue, JsValue>;
}
