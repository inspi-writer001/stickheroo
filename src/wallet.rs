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

/// Save a value to localStorage
pub fn save_to_storage(key: &str, value: &str) {
    js_save_to_storage(key, value);
}

/// Load a value from localStorage (returns empty string if not found)
pub fn load_from_storage(key: &str) -> Option<String> {
    let val = js_load_from_storage(key);
    let s = val.as_string();
    match s {
        Some(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

/// Upload arbitrary text to Arweave via Irys devnet.
/// Used for collection metadata (no image needed).
/// Throws on failure — caller decides on fallback.
pub async fn upload_to_irys(content: &str, content_type: &str) -> Result<String, String> {
    js_upload_to_irys(content, content_type)
        .await
        .map_err(|e| format!("Irys upload error: {:?}", e))?
        .as_string()
        .ok_or_else(|| "No URL returned from Irys upload".to_string())
}

/// Upload binary data to Arweave via Irys devnet.
pub async fn upload_bytes_to_irys(data: &[u8], content_type: &str) -> Result<String, String> {
    let js_data = js_sys::Uint8Array::from(data);
    js_upload_bytes_to_irys(&js_data.into(), content_type)
        .await
        .map_err(|e| format!("Irys upload error: {:?}", e))?
        .as_string()
        .ok_or_else(|| "No URL returned from Irys upload".to_string())
}

/// Upload character metadata to Arweave (two-step: PNG image then JSON).
/// Generates a PNG of the stick figure on an HTML canvas, uploads it to get
/// a real https://gateway.irys.xyz/<id> image URL, builds the metadata JSON
/// referencing that URL, uploads the JSON, and returns the metadata URL.
/// char_index drives the character colour (hue = index * 60 degrees).
pub async fn upload_character_metadata(
    char_index: usize,
    name: &str,
    description: &str,
    hp: u32,
    atk: u32,
    def: u32,
) -> Result<String, String> {
    let hue = (char_index * 60) as u32;
    js_upload_character_metadata(hue, name, description, hp, atk, def)
        .await
        .map_err(|e| format!("Upload error: {:?}", e))?
        .as_string()
        .ok_or_else(|| "No URL returned from character upload".to_string())
}

// JS bridge functions
#[wasm_bindgen(inline_js = "
// ANS-104 Irys devnet upload - browser-native Web Crypto + fetch

async function sha384(data) {
    return new Uint8Array(await window.crypto.subtle.digest('SHA-384', data));
}

function stringToBytes(s) {
    return new TextEncoder().encode(s);
}

function concat(...arrays) {
    const total = arrays.reduce((s, a) => s + a.length, 0);
    const out = new Uint8Array(total);
    let off = 0;
    for (const a of arrays) { out.set(a, off); off += a.length; }
    return out;
}

// Avro zigzag varint encoding
function avroInt(n) {
    const bytes = [];
    let v = n * 2; // zigzag encode for non-negative
    do { bytes.push((v & 0x7f) | (v > 0x7f ? 0x80 : 0)); v >>>= 7; } while (v > 0);
    return new Uint8Array(bytes);
}

function avroString(s) {
    const b = new TextEncoder().encode(s);
    return concat(avroInt(b.length), b);
}

// Serialize tags using Avro array format
function serializeTags(tags) {
    if (!tags || !tags.length) return new Uint8Array(0);
    const parts = [];
    parts.push(avroInt(tags.length));
    for (const t of tags) {
        parts.push(avroString(t.name), avroString(t.value));
    }
    parts.push(new Uint8Array([0]));
    return concat(...parts);
}

// Deep hash per ANS-104 - uses SHA-384
async function deepHash(data) {
    if (Array.isArray(data)) {
        const tag = concat(stringToBytes('list'), stringToBytes(data.length.toString()));
        let h = await sha384(tag);
        for (const item of data) {
            h = await sha384(concat(h, await deepHash(item)));
        }
        return h;
    }
    const bytes = data;
    const tag = concat(stringToBytes('blob'), stringToBytes(bytes.length.toString()));
    return await sha384(concat(await sha384(tag), await sha384(bytes)));
}

// 8-byte little-endian
function longTo8Bytes(n) {
    const a = new Uint8Array(8);
    for (let i = 0; i < 8; i++) { a[i] = n & 0xff; n = Math.floor(n / 256); }
    return a;
}

// 2-byte little-endian
function shortTo2Bytes(n) {
    return new Uint8Array([n & 0xff, (n >> 8) & 0xff]);
}

async function irysUpload(data, tags) {
    const enc = new TextEncoder();
    const dataBytes = typeof data === 'string' ? enc.encode(data) : data;
    const tagsBytes = serializeTags(tags);

    const keypair = solanaWeb3.Keypair.generate();
    const owner = keypair.publicKey.toBytes();
    const sigType = 2; // Solana
    const sigTypeBytes = shortTo2Bytes(sigType);

    // Deep hash for signature - per arbundles getSignatureData
    const sigData = await deepHash([
        stringToBytes('dataitem'),
        stringToBytes('1'),
        stringToBytes(sigType.toString()),
        owner,
        new Uint8Array(0), // rawTarget - empty
        new Uint8Array(0), // rawAnchor - empty
        tagsBytes,         // rawTags - Avro serialized
        dataBytes,
    ]);

    // Ed25519 signing
    const seed = keypair.secretKey.slice(0, 32);
    const pkcs8 = concat(
        new Uint8Array([0x30,0x2e,0x02,0x01,0x00,0x30,0x05,0x06,0x03,0x2b,0x65,0x70,0x04,0x22,0x04,0x20]),
        seed
    );
    const cryptoKey = await window.crypto.subtle.importKey('pkcs8', pkcs8, { name: 'Ed25519' }, false, ['sign']);
    const signature = new Uint8Array(await window.crypto.subtle.sign('Ed25519', cryptoKey, sigData));

    // Build binary DataItem
    const item = concat(
        sigTypeBytes,      // signature type (2 bytes)
        signature,         // 64 bytes
        owner,             // 32 bytes
        new Uint8Array([0]), // target presence byte (0 = not present)
        new Uint8Array([0]), // anchor presence byte (0 = not present)
        longTo8Bytes(tags ? tags.length : 0),
        longTo8Bytes(tagsBytes.length),
        tagsBytes,
        dataBytes
    );

    const resp = await fetch('https://devnet.irys.xyz/tx/solana', {
        method: 'POST',
        headers: { 'Content-Type': 'application/octet-stream' },
        body: item,
    });
    if (!resp.ok) throw new Error('Irys upload ' + resp.status + ': ' + (await resp.text()));
    return 'https://gateway.irys.xyz/' + (await resp.json()).id;
}

export async function js_upload_to_irys(content, content_type) {
    return irysUpload(content, [{ name: 'Content-Type', value: content_type }]);
}

export async function js_upload_bytes_to_irys(data, content_type) {
    return irysUpload(new Uint8Array(data), [{ name: 'Content-Type', value: content_type }]);
}

// Two-step upload: PNG image first, then metadata JSON referencing the image URL.
export async function js_upload_character_metadata(hue, name, description, hp, atk, def) {
    // --- Step 1: draw the character as a PNG on an HTML canvas ---
    const canvas = document.createElement('canvas');
    canvas.width = 200;
    canvas.height = 280;
    const ctx = canvas.getContext('2d');

    ctx.fillStyle = '#0a0a0a';
    ctx.fillRect(0, 0, 200, 280);

    const color = 'hsl(' + hue + ', 80%, 60%)';
    ctx.shadowColor = color;
    ctx.shadowBlur = 10;
    ctx.strokeStyle = color;
    ctx.lineWidth = 4;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';

    ctx.beginPath(); ctx.arc(100, 55, 25, 0, Math.PI * 2); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(100, 80); ctx.lineTo(100, 155); ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(100, 100); ctx.lineTo(55, 130);
    ctx.moveTo(100, 100); ctx.lineTo(145, 130);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(100, 155); ctx.lineTo(65, 215);
    ctx.moveTo(100, 155); ctx.lineTo(135, 215);
    ctx.stroke();

    ctx.shadowBlur = 6; ctx.fillStyle = color;
    ctx.font = 'bold 16px monospace'; ctx.textAlign = 'center';
    ctx.fillText(name.toUpperCase(), 100, 245);
    ctx.shadowBlur = 0; ctx.fillStyle = '#888888';
    ctx.font = '11px monospace';
    ctx.fillText('HP:' + hp + '  ATK:' + atk + '  DEF:' + def, 100, 265);

    const imgBytes = await new Promise((resolve, reject) => {
        canvas.toBlob(blob => {
            if (!blob) { reject(new Error('canvas.toBlob failed')); return; }
            blob.arrayBuffer().then(buf => resolve(new Uint8Array(buf))).catch(reject);
        }, 'image/png');
    });

    // --- Step 2: upload PNG, get image URL ---
    const imageUrl = await irysUpload(imgBytes, [{ name: 'Content-Type', value: 'image/png' }]);

    // --- Step 3: build metadata JSON and upload ---
    const metadata = JSON.stringify({
        name: name,
        description: description,
        image: imageUrl,
        attributes: [
            { trait_type: 'HP', value: hp },
            { trait_type: 'ATK', value: atk },
            { trait_type: 'DEF', value: def }
        ]
    });
    return irysUpload(metadata, [{ name: 'Content-Type', value: 'application/json' }]);
}

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

export function js_save_to_storage(key, value) {
    try { localStorage.setItem(key, value); } catch(e) {}
}

export function js_load_from_storage(key) {
    try { return localStorage.getItem(key) || ''; } catch(e) { return ''; }
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

    fn js_save_to_storage(key: &str, value: &str);

    fn js_load_from_storage(key: &str) -> JsValue;

    #[wasm_bindgen(catch)]
    async fn js_upload_to_irys(content: &str, content_type: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_upload_bytes_to_irys(data: &JsValue, content_type: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_upload_character_metadata(
        hue: u32,
        name: &str,
        description: &str,
        hp: u32,
        atk: u32,
        def: u32,
    ) -> Result<JsValue, JsValue>;
}
