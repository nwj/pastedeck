use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::paste_helper::TestPaste;
use crate::prelude::*;
use reqwest::StatusCode;

#[tokio::test]
async fn create_persists_when_valid_form_data() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = app.seed_random_user().await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let paste = TestPaste::builder().random()?.build();

    let response = client.pastes().post(&paste).await?;

    assert_eq!(response.status(), StatusCode::OK);
    let persisted = app
        .db
        .conn
        .call(|conn| {
            Ok(conn
                .prepare("SELECT filename, description, body FROM pastes")?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<std::result::Result<Vec<(String, String, String)>, _>>())
        })
        .await??;
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].0, paste.filename);
    assert_eq!(persisted[0].1, paste.description);
    assert_eq!(persisted[0].2, paste.body);
    Ok(())
}

#[tokio::test]
async fn create_responds_with_400_when_data_missing() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = app.seed_random_user().await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let bad_pastes = vec![
        TestPaste::builder().filename("").build(),
        TestPaste::builder().body("").build(),
    ];

    for bad_paste in bad_pastes {
        let response = client.pastes().post(&bad_paste).await?;
        assert_eq!(response.status(), 400);
    }
    Ok(())
}

#[tokio::test]
async fn index_lists_all_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste1 = TestPaste::builder()
        .random()?
        .build()
        .persist(&client)
        .await?;
    let paste2 = TestPaste::builder()
        .random()?
        .build()
        .persist(&client)
        .await?;

    let response = client.pastes().get().await?;
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();

    assert!(body.contains(&paste1.filename));
    assert!(body.contains(&paste2.filename));
    Ok(())
}
