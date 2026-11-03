use adversary_sdk::serve;
use redb::{Database, Durability, ReadableDatabase, TableDefinition};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf};
const TABLE: TableDefinition<&str, &str> = TableDefinition::new("kv");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = PathBuf::from(std::env::var("ADVERSARY_DATA_DIR")?);
    std::fs::create_dir_all(&data)?;
    let db = Database::create(data.join("redb.db"))?;
    let mode = std::env::var("ADVERSARY_EXAMPLE_MODE").unwrap_or_else(|_| "atomic".into());
    serve(|request, ctx| {
        match request.method.as_str() {
            "initialize" | "apply" => {
                let is_init = request.method == "initialize";
                let values: BTreeMap<String, String> = serde_json::from_value(
                    request.params[if is_init { "state" } else { "batch" }].clone(),
                )
                .map_err(|e| e.to_string())?;
                if !is_init && mode == "split" {
                    // Deliberate application-level bug: a promised atomic batch
                    // is implemented as two independently durable transactions.
                    for (i, (key, value)) in values.iter().enumerate() {
                        let tx = db.begin_write().map_err(|e| e.to_string())?;
                        {
                            let mut t = tx.open_table(TABLE).map_err(|e| e.to_string())?;
                            t.insert(key.as_str(), value.as_str())
                                .map_err(|e| e.to_string())?;
                        }
                        tx.commit().map_err(|e| e.to_string())?;
                        if i == 0 {
                            ctx.checkpoint("after-first-write")?;
                        }
                    }
                } else {
                    let mut tx = db.begin_write().map_err(|e| e.to_string())?;
                    if !is_init && mode == "relaxed" {
                        tx.set_durability(Durability::None)
                            .map_err(|e| e.to_string())?;
                    }
                    {
                        let mut table = tx.open_table(TABLE).map_err(|e| e.to_string())?;
                        for (i, (key, value)) in values.iter().enumerate() {
                            table
                                .insert(key.as_str(), value.as_str())
                                .map_err(|e| e.to_string())?;
                            if !is_init && i == 0 {
                                ctx.checkpoint("after-first-write")?;
                            }
                        }
                    }
                    tx.commit().map_err(|e| e.to_string())?;
                }
                Ok(json!({"accepted":true}))
            }
            "observe" => {
                let tx = db.begin_read().map_err(|e| e.to_string())?;
                let table = tx.open_table(TABLE).map_err(|e| e.to_string())?;
                let keys: Vec<String> = serde_json::from_value(request.params["keys"].clone())
                    .map_err(|e| e.to_string())?;
                let mut state = BTreeMap::new();
                for key in keys {
                    let value = table
                        .get(key.as_str())
                        .map_err(|e| e.to_string())?
                        .map(|x| x.value().to_string());
                    state.insert(key, value);
                }
                serde_json::to_value(state).map_err(|e| e.to_string())
            }
            _ => Err("unknown method".into()),
        }
    })
    .map_err(std::io::Error::other)?;
    Ok(())
}
