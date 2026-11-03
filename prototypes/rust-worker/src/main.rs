use redb::{Database, Durability, ReadableDatabase, TableDefinition};
use std::io::{self, BufRead, Write};
const T: TableDefinition<&str, &str> = TableDefinition::new("kv");
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let db = Database::create(&a[2])?;
    if a[1] == "read" {
        let tx = db.begin_read()?;
        let t = tx.open_table(T)?;
        println!(
            "{{\"a\":\"{}\",\"b\":\"{}\"}}",
            t.get("a")?
                .map(|x| x.value().to_string())
                .unwrap_or_default(),
            t.get("b")?
                .map(|x| x.value().to_string())
                .unwrap_or_default()
        );
        return Ok(());
    }
    let mut tx = db.begin_write()?;
    if a.get(3).map(String::as_str) == Some("unsafe") {
        tx.set_durability(Durability::None)?;
    }
    {
        let mut t = tx.open_table(T)?;
        let v = if a[1] == "init" { "0" } else { "1" };
        t.insert("a", v)?;
        t.insert("b", v)?;
    }
    tx.commit()?;
    println!("{{\"status\":\"ok\",\"op\":\"put_pair\"}}");
    io::stdout().flush()?;
    if a[1] == "write" {
        let _ = io::stdin().lock().lines().next();
    }
    Ok(())
}
