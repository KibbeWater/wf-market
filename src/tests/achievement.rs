use crate::client::Client;

#[tokio::test]
async fn all_achievement() {
    let client = Client::new();
    let achievements = client.achievement().get_achievements().await.unwrap();
    for achievement in &achievements {
        println!(
            "Achievement: {:?}",
            achievement.i18n.get("en").unwrap().name
        );
    }
}

#[tokio::test]
async fn locations() {
    let client = Client::new();
    let achievements = client.achievement().get_achievements().await.unwrap();
    for achievement in &achievements {
        println!(
            "Achievement: {:?}",
            achievement.i18n.get("en").unwrap().name
        );
    }
}

#[tokio::test]
async fn get_achievements_for_user_by_slug() {
    let slug = "example_slug"; // Replace with a valid slug
    let client = Client::new();
    let achievements = client
        .achievement()
        .get_achievements_for_user_by_slug(slug)
        .await
        .unwrap();
    for achievement in &achievements {
        println!(
            "User Slug Achievement: {:?}",
            achievement.i18n.get("en").unwrap().name
        );
    }
}

#[tokio::test]
async fn get_achievements_for_user_by_id() {
    let user_id = "example_user_id"; // Replace with a valid user ID
    let client = Client::new();
    let achievements = client
        .achievement()
        .get_achievements_for_user_by_id(user_id)
        .await
        .unwrap();
    for achievement in &achievements {
        println!(
            "User Id Achievement: {:?}",
            achievement.i18n.get("en").unwrap().name
        );
    }
}
