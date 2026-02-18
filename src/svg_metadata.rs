/// SVG-based NFT metadata generation.
/// On-chain URIs are kept minimal to fit Solana's ~1232 byte tx limit.
/// Full SVGs are only used client-side (profile page rendering).

/// Simple base64 encoder (no extra dependency needed).
pub fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

const BG: &str = "rgb(10,10,10)";

/// Generate a full colored stick-figure SVG for a character (client-side only).
/// Used on the profile page to render owned characters — NOT sent on-chain.
pub fn generate_character_svg(index: usize, name: &str) -> String {
    let hue = index * 60;
    let color = format!("hsl({},80%,60%)", hue);
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"120\" height=\"160\" viewBox=\"0 0 120 160\">\
         <rect width=\"120\" height=\"160\" fill=\"{BG}\"/>\
         <circle cx=\"60\" cy=\"35\" r=\"16\" fill=\"none\" stroke=\"{c}\" stroke-width=\"2\"/>\
         <line x1=\"60\" y1=\"51\" x2=\"60\" y2=\"100\" stroke=\"{c}\" stroke-width=\"2\"/>\
         <line x1=\"60\" y1=\"65\" x2=\"35\" y2=\"85\" stroke=\"{c}\" stroke-width=\"2\"/>\
         <line x1=\"60\" y1=\"65\" x2=\"85\" y2=\"85\" stroke=\"{c}\" stroke-width=\"2\"/>\
         <line x1=\"60\" y1=\"100\" x2=\"40\" y2=\"140\" stroke=\"{c}\" stroke-width=\"2\"/>\
         <line x1=\"60\" y1=\"100\" x2=\"80\" y2=\"140\" stroke=\"{c}\" stroke-width=\"2\"/>\
         <text x=\"60\" y=\"155\" text-anchor=\"middle\" fill=\"{c}\" font-family=\"monospace\" font-size=\"11\">{n}</text>\
         </svg>",
        BG = BG,
        c = color,
        n = name,
    )
}

/// Build a tiny on-chain metadata URI for a character NFT.
/// Keeps it small (~120 bytes) to fit within Solana tx limits.
/// The image is a minimal colored-circle SVG with the character's initial.
pub fn build_character_metadata_uri(index: usize, name: &str, _description: &str) -> String {
    let hue = index * 60;
    let initial = name.chars().next().unwrap_or('?');
    // Tiny SVG: colored circle + initial letter (~120 bytes)
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"32\" height=\"32\">\
         <circle cx=\"16\" cy=\"16\" r=\"15\" fill=\"hsl({h},80%,60%)\"/>\
         <text x=\"16\" y=\"22\" text-anchor=\"middle\" fill=\"#fff\" font-size=\"18\">{i}</text>\
         </svg>",
        h = hue,
        i = initial,
    );
    let svg_b64 = base64_encode(svg.as_bytes());
    let img = format!("data:image/svg+xml;base64,{}", svg_b64);

    // Minimal JSON, no extra fields
    let json = format!(
        "{{\"name\":\"{n}\",\"image\":\"{img}\"}}",
        n = name,
        img = img,
    );
    let json_b64 = base64_encode(json.as_bytes());
    format!("data:application/json;base64,{}", json_b64)
}

/// Build a tiny on-chain metadata URI for a collection.
pub fn build_collection_metadata_uri(name: &str) -> String {
    let json = format!(
        "{{\"name\":\"{n}\",\"image\":\"\"}}",
        n = name,
    );
    let json_b64 = base64_encode(json.as_bytes());
    format!("data:application/json;base64,{}", json_b64)
}

/// Build a tiny on-chain metadata URI for a profile picture.
pub fn build_profile_metadata_uri(name: &str) -> String {
    let json = format!(
        "{{\"name\":\"{n}\",\"image\":\"\"}}",
        n = name,
    );
    let json_b64 = base64_encode(json.as_bytes());
    format!("data:application/json;base64,{}", json_b64)
}
