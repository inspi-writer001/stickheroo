use mojo_rust_sdk::transaction::TransactionBundle;
use serde::Serialize;

use crate::wallet::js_build_and_send_tx;

#[derive(Serialize)]
struct SerializedAccount {
    pubkey: Vec<u8>,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct SerializedInstruction {
    program_id: Vec<u8>,
    accounts: Vec<SerializedAccount>,
    data: Vec<u8>,
}

/// Send a TransactionBundle via the JS bridge (Phantom wallet signing)
pub async fn send_transaction_bundle(bundle: TransactionBundle) -> Result<String, String> {
    // Serialize instructions to JSON
    let serialized_ixs: Vec<SerializedInstruction> = bundle
        .instructions
        .iter()
        .map(|ix| SerializedInstruction {
            program_id: ix.program_id.to_bytes().to_vec(),
            accounts: ix
                .accounts
                .iter()
                .map(|acc| SerializedAccount {
                    pubkey: acc.pubkey.to_bytes().to_vec(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect(),
            data: ix.data.clone(),
        })
        .collect();

    let ixs_json =
        serde_json::to_string(&serialized_ixs).map_err(|e| format!("Serialize ixs: {}", e))?;

    // Serialize ephemeral signer secret keys
    let signer_bytes: Vec<Vec<u8>> = bundle
        .signers
        .iter()
        .map(|kp| kp.to_bytes().to_vec())
        .collect();

    let signers_json =
        serde_json::to_string(&signer_bytes).map_err(|e| format!("Serialize signers: {}", e))?;

    let result = js_build_and_send_tx(&ixs_json, &signers_json)
        .await
        .map_err(|e| format!("{:?}", e))?;

    result
        .as_string()
        .ok_or_else(|| "No signature returned".to_string())
}
