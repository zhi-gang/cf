//! The module to wrap MongoDB features

use mongodb::{bson::Document, Client, Collection};

static mut CLIENT: Option<Client> = None;

pub fn init(client: Client) {
    unsafe { CLIENT = Some(client) };
}

pub fn check_init() -> anyhow::Result<Client> {
    match unsafe { CLIENT } {
        Some(c) => Ok(c),
        None => Err(anyhow::Error::msg("Client is None")),
    }
}

// pub fn use_db(db: &str) -> anyhow::Result<()> {
//     let client = check_init()?;

//     let db: mongodb::Database = client.database(db);
//     let collection: Collection<_> = db.collection("my_collection");
//     Ok(())
// }

pub async fn insert_doc(db: &'_ str, collection: &'_ str, doc: Document) -> anyhow::Result<()> {
    let client = check_init()?;
    let db = client.database(db);
    let c = db.collection(collection);
    c.insert_one(doc, None).await?;
    Ok(())
}

pub async fn delete_doc(db: &'_ str, collection: &'_ str, filter: Document) -> anyhow::Result<()> {
    let client = check_init()?;
    let db = client.database(db);
    let c:Collection<Document> = db.collection(collection);
    c.delete_one(filter, None).await?;
    Ok(())
}
