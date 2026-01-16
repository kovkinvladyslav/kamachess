use kamachess::db;
use kamachess::models::User;
use sqlx::any::AnyPoolOptions;

async fn setup_test_db() -> sqlx::Pool<sqlx::Any> {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::run_migrations(&pool, "sqlite::memory:").await.unwrap();
    pool
}

fn test_user(id: i64, username: Option<&str>) -> User {
    User {
        id,
        is_bot: false,
        username: username.map(String::from),
        first_name: Some(format!("User{}", id)),
        last_name: None,
    }
}

#[tokio::test]
async fn test_upsert_user_creates_new_user() {
    let pool = setup_test_db().await;
    let user = test_user(12345, Some("testuser"));

    let db_user = db::upsert_user(&pool, &user).await.unwrap();

    assert_eq!(db_user.telegram_id, Some(12345));
    assert_eq!(db_user.username, Some("testuser".to_string()));
    assert_eq!(db_user.first_name, Some("User12345".to_string()));
    assert_eq!(db_user.wins, 0);
    assert_eq!(db_user.losses, 0);
    assert_eq!(db_user.draws, 0);
}

#[tokio::test]
async fn test_upsert_user_updates_existing() {
    let pool = setup_test_db().await;
    let user1 = test_user(12345, Some("oldname"));
    db::upsert_user(&pool, &user1).await.unwrap();

    let user2 = User {
        id: 12345,
        is_bot: false,
        username: Some("newname".to_string()),
        first_name: Some("NewFirst".to_string()),
        last_name: None,
    };
    let db_user = db::upsert_user(&pool, &user2).await.unwrap();

    assert_eq!(db_user.username, Some("newname".to_string()));
    assert_eq!(db_user.first_name, Some("NewFirst".to_string()));
}

#[tokio::test]
async fn test_upsert_user_by_username() {
    let pool = setup_test_db().await;

    let db_user = db::upsert_user_by_username(&pool, "newplayer").await.unwrap();

    assert_eq!(db_user.username, Some("newplayer".to_string()));
    assert_eq!(db_user.telegram_id, None);
}

#[tokio::test]
async fn test_upsert_user_merges_placeholder() {
    let pool = setup_test_db().await;
    
    // Create a placeholder user (like when starting a game with /start @username)
    let placeholder = db::upsert_user_by_username(&pool, "emovadilda").await.unwrap();
    let placeholder_id = placeholder.id;
    assert_eq!(placeholder.telegram_id, None);
    assert_eq!(placeholder.username, Some("emovadilda".to_string()));
    
    // Now the real user tries to make a move (they have telegram_id)
    let real_user = User {
        id: 12345,
        is_bot: false,
        username: Some("emovadilda".to_string()),
        first_name: Some("Real".to_string()),
        last_name: None,
    };
    let merged_user = db::upsert_user(&pool, &real_user).await.unwrap();
    
    // Should be the same user ID (merged, not a new user)
    assert_eq!(merged_user.id, placeholder_id);
    assert_eq!(merged_user.telegram_id, Some(12345));
    assert_eq!(merged_user.username, Some("emovadilda".to_string()));
    assert_eq!(merged_user.first_name, Some("Real".to_string()));
}

#[tokio::test]
async fn test_get_user_by_telegram_id() {
    let pool = setup_test_db().await;
    let user = test_user(99999, Some("finder"));
    db::upsert_user(&pool, &user).await.unwrap();

    let found = db::get_user_by_telegram_id(&pool, 99999).await.unwrap();

    assert_eq!(found.username, Some("finder".to_string()));
}

#[tokio::test]
async fn test_get_user_by_username() {
    let pool = setup_test_db().await;
    let user = test_user(11111, Some("byname"));
    db::upsert_user(&pool, &user).await.unwrap();

    let found = db::get_user_by_username(&pool, "byname").await.unwrap();

    assert_eq!(found.telegram_id, Some(11111));
}

#[tokio::test]
async fn test_create_game() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, Some("white"))).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, Some("black"))).await.unwrap();

    let game_id = db::create_game(
        &pool,
        -100,
        white.id,
        black.id,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "white",
    )
    .await
    .unwrap();

    assert!(game_id > 0);
}

#[tokio::test]
async fn test_find_ongoing_game() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, Some("w"))).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, Some("b"))).await.unwrap();
    let chat_id = -200;

    db::create_game(
        &pool,
        chat_id,
        white.id,
        black.id,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "white",
    )
    .await
    .unwrap();

    let found = db::find_ongoing_game(&pool, chat_id, white.id, black.id)
        .await
        .unwrap();
    assert!(found.is_some());

    let found_reverse = db::find_ongoing_game(&pool, chat_id, black.id, white.id)
        .await
        .unwrap();
    assert!(found_reverse.is_some());

    let not_found = db::find_ongoing_game(&pool, chat_id, white.id, white.id)
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_find_game_by_message() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();
    let chat_id = -300;
    let message_id = 42;

    let game_id = db::create_game(
        &pool,
        chat_id,
        white.id,
        black.id,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "white",
    )
    .await
    .unwrap();
    db::update_game_message(&pool, game_id, message_id).await.unwrap();

    let found = db::find_game_by_message(&pool, chat_id, message_id)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, game_id);
}

#[tokio::test]
async fn test_find_game_by_message_via_game_messages_table() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();
    let chat_id = -400;
    let old_message_id = 100;
    let new_message_id = 101;

    let game_id = db::create_game(
        &pool,
        chat_id,
        white.id,
        black.id,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "white",
    )
    .await
    .unwrap();
    
    // Insert an old message into game_messages table
    db::insert_game_message(&pool, game_id, old_message_id).await.unwrap();
    
    // Update last_message_id to a newer message
    db::update_game_message(&pool, game_id, new_message_id).await.unwrap();

    // Should find game by old message ID via game_messages table
    let found = db::find_game_by_message(&pool, chat_id, old_message_id)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, game_id);
    
    // Should also find game by new message ID via last_message_id
    let found = db::find_game_by_message(&pool, chat_id, new_message_id)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, game_id);
}

#[tokio::test]
async fn test_update_game_fen() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();

    let game_id = db::create_game(
        &pool,
        -400,
        white.id,
        black.id,
        "start_fen",
        "white",
    )
    .await
    .unwrap();

    db::update_game_fen(&pool, game_id, "new_fen", "black").await.unwrap();
    db::update_game_message(&pool, game_id, 1).await.unwrap();

    let game = db::find_game_by_message(&pool, -400, 1).await.unwrap().unwrap();
    assert_eq!(game.current_fen, "new_fen");
    assert_eq!(game.turn, "black");
}

#[tokio::test]
async fn test_insert_and_get_next_move_number() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();

    let game_id = db::create_game(&pool, -500, white.id, black.id, "fen", "white")
        .await
        .unwrap();

    let next = db::next_move_number(&pool, game_id).await.unwrap();
    assert_eq!(next, 1);

    db::insert_move(&pool, game_id, white.id, 1, "e2e4", Some("e4"))
        .await
        .unwrap();

    let next = db::next_move_number(&pool, game_id).await.unwrap();
    assert_eq!(next, 2);

    db::insert_move(&pool, game_id, black.id, 2, "e7e5", Some("e5"))
        .await
        .unwrap();

    let next = db::next_move_number(&pool, game_id).await.unwrap();
    assert_eq!(next, 3);
}

#[tokio::test]
async fn test_update_game_result() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();

    let game_id = db::create_game(&pool, -600, white.id, black.id, "fen", "white")
        .await
        .unwrap();
    db::update_game_message(&pool, game_id, 1).await.unwrap();

    db::update_game_result(&pool, game_id, &Some("1-0".to_string()), "finished")
        .await
        .unwrap();

    let game = db::find_game_by_message(&pool, -600, 1).await.unwrap().unwrap();
    assert_eq!(game.status, "finished");
    assert_eq!(game.result, Some("1-0".to_string()));
}

#[tokio::test]
async fn test_update_player_stats_white_wins() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();

    db::update_player_stats(&pool, white.id, black.id, "1-0").await.unwrap();

    let white_updated = db::get_user_by_id(&pool, white.id).await.unwrap();
    let black_updated = db::get_user_by_id(&pool, black.id).await.unwrap();

    assert_eq!(white_updated.wins, 1);
    assert_eq!(white_updated.losses, 0);
    assert_eq!(black_updated.wins, 0);
    assert_eq!(black_updated.losses, 1);
}

#[tokio::test]
async fn test_update_player_stats_black_wins() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();

    db::update_player_stats(&pool, white.id, black.id, "0-1").await.unwrap();

    let white_updated = db::get_user_by_id(&pool, white.id).await.unwrap();
    let black_updated = db::get_user_by_id(&pool, black.id).await.unwrap();

    assert_eq!(white_updated.losses, 1);
    assert_eq!(black_updated.wins, 1);
}

#[tokio::test]
async fn test_update_player_stats_draw() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();

    db::update_player_stats(&pool, white.id, black.id, "1/2-1/2").await.unwrap();

    let white_updated = db::get_user_by_id(&pool, white.id).await.unwrap();
    let black_updated = db::get_user_by_id(&pool, black.id).await.unwrap();

    assert_eq!(white_updated.draws, 1);
    assert_eq!(black_updated.draws, 1);
}

#[tokio::test]
async fn test_propose_and_clear_draw() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, None)).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();

    let game_id = db::create_game(&pool, -700, white.id, black.id, "fen", "white")
        .await
        .unwrap();
    db::update_game_message(&pool, game_id, 1).await.unwrap();

    db::propose_draw(&pool, game_id, white.id).await.unwrap();
    let game = db::find_game_by_message(&pool, -700, 1).await.unwrap().unwrap();
    assert_eq!(game.draw_proposed_by, Some(white.id));

    db::clear_draw_proposal(&pool, game_id).await.unwrap();
    let game = db::find_game_by_message(&pool, -700, 1).await.unwrap().unwrap();
    assert_eq!(game.draw_proposed_by, None);
}

#[tokio::test]
async fn test_format_user_history_empty() {
    let pool = setup_test_db().await;
    let user = db::upsert_user(&pool, &test_user(1, Some("histuser"))).await.unwrap();

    let history = db::format_user_history(&pool, &user, -800, 1).await.unwrap();

    assert!(history.contains("History for"));
    assert!(history.contains("No games yet."));
    assert!(history.contains("Wins: 0"));
}

#[tokio::test]
async fn test_format_user_history_with_games() {
    let pool = setup_test_db().await;
    let white = db::upsert_user(&pool, &test_user(1, Some("player1"))).await.unwrap();
    let black = db::upsert_user(&pool, &test_user(2, Some("player2"))).await.unwrap();
    let chat_id = -900;

    let game_id = db::create_game(
        &pool,
        chat_id,
        white.id,
        black.id,
        "fen",
        "white",
    )
    .await
    .unwrap();
    db::insert_move(&pool, game_id, white.id, 1, "e2e4", Some("e4")).await.unwrap();
    db::update_game_result(&pool, game_id, &Some("1-0".to_string()), "finished")
        .await
        .unwrap();
    db::update_player_stats(&pool, white.id, black.id, "1-0").await.unwrap();

    let history = db::format_user_history(&pool, &white, chat_id, 1).await.unwrap();

    assert!(history.contains("@player1"));
    assert!(history.contains("@player2"));
    assert!(history.contains("1-0"));
    assert!(history.contains("lichess.org"));
}

#[tokio::test]
async fn test_format_head_to_head() {
    let pool = setup_test_db().await;
    let user_a = db::upsert_user(&pool, &test_user(1, Some("alice"))).await.unwrap();
    let user_b = db::upsert_user(&pool, &test_user(2, Some("bob"))).await.unwrap();
    let chat_id = -1000;

    db::create_game(&pool, chat_id, user_a.id, user_b.id, "fen", "white")
        .await
        .unwrap();

    let h2h = db::format_head_to_head(&pool, &user_a, &user_b, chat_id, 1)
        .await
        .unwrap();

    assert!(h2h.contains("Head-to-head"));
    assert!(h2h.contains("@alice"));
    assert!(h2h.contains("@bob"));
    assert!(h2h.contains("Total games: 1"));
}

#[tokio::test]
async fn test_db_user_display_name() {
    let pool = setup_test_db().await;

    let user_with_username = db::upsert_user(&pool, &test_user(1, Some("testname"))).await.unwrap();
    assert_eq!(user_with_username.display_name(), "@testname");

    let user_without_username = db::upsert_user(&pool, &test_user(2, None)).await.unwrap();
    assert_eq!(user_without_username.display_name(), "User2");
}

#[tokio::test]
async fn test_db_user_mention_html() {
    let pool = setup_test_db().await;

    let user = db::upsert_user(&pool, &test_user(12345, Some("htmltest"))).await.unwrap();
    let mention = user.mention_html();
    assert!(mention.contains("tg://user?id=12345"));
    assert!(mention.contains("User12345"));
}
