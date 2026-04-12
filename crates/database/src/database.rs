use brain_common::config;
use tokio::sync::OnceCell;

static DB: OnceCell<toasty::Db> = OnceCell::const_new();

pub async fn get() -> toasty::Db {
    let datasource = config::get().datasource();
    let conn_url = format!(
        "postgresql://{}:{}@{}:{}/{}?schema={}",
        datasource.user(),
        datasource.password(),
        datasource.host(),
        datasource.port(),
        datasource.database(),
        datasource.schema()
    );

    DB.get_or_init(|| async {
        toasty::Db::builder()
            .models(toasty::models!(crate::*))
            .connect(&conn_url)
            .await
            .expect("Failed to connect to database")
    })
    .await
    .clone()
}

pub async fn init_table() {
    get().await.push_schema().await.expect("Init talbe failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_table() {
        init_table().await;
    }
}
