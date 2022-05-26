// Set the `DB_REMEMBER_REDIS` environmental variable if you want to use Redis.
// Otherwise, the default is Sqlite.
use dotenv::dotenv;
use teloxide::{
    dispatching2::dialogue::{
        serializer::{Bincode, Json},
        ErasedStorage, RedisStorage, SqliteStorage, Storage,
    },
    macros::DialogueState,
    prelude2::*,
    types::Me,
    utils::command::BotCommand,
    types::{
        InlineKeyboardButton,
        InlineKeyboardMarkup,
    }
};

type MyDialogue = Dialogue<State, ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(DialogueState, Clone, serde::Serialize, serde::Deserialize)]
#[handler_out(HandlerResult)]
pub enum State {
    #[handler(handle_start)]
    Start,

    #[handler(handle_got_number)]
    GotNumber(i32),

    #[handler(handle_got_number)]
    AddNumber(i32),

    #[handler(handle_got_number)]
    SubNumber(i32),

    #[handler(handle_got_number)]
    BattlePlayer,
}

impl Default for State {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "get your number.")]
    Get,
    #[command(description = "reset your number.")]
    Reset,
    #[command(description = "add your number.")]
    Add(String),
    #[command(description = "sub your number.")]
    Sub(String),
    #[command(description = "sub your number.")]
    Battle,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting db_remember_bot...");

    let bot = Bot::from_env().auto_send();

    let storage: MyStorage = if std::env::var("DB_REMEMBER_REDIS").is_ok() {
        RedisStorage::open("redis://127.0.0.1:6379", Bincode).await.unwrap().erase()
    } else {
        SqliteStorage::open("db.sqlite", Json).await.unwrap().erase()
    };

    let handler = Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<State>, State>()
        .dispatch_by::<State>();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}

async fn handle_start(bot: AutoSend<Bot>, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    match msg.text().unwrap().parse() {
        Ok(number) => {
            dialogue.update(State::GotNumber(number)).await?;
            bot.send_message(
                msg.chat.id,
                format!("Remembered number {}. Now use /get or /reset", number),
            )
            .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please, send me a number").await?;
        }
    }

    Ok(())
}

async fn handle_got_number(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: MyDialogue,
    num: i32,
    me: Me,
) -> HandlerResult {
    let ans = msg.text().unwrap();
    let bot_name = me.user.username.unwrap();

    match Command::parse(ans, bot_name) {
        Ok(cmd) => match cmd {
            Command::Get => {
                bot.send_message(msg.chat.id, format!("Here is your number: {}", num)).await?;
            }
            Command::Reset => {
                dialogue.reset().await?;
                bot.send_message(msg.chat.id, "Number resetted").await?;
            }
            Command::Add(number_str) => {
                let number: i32 = number_str.parse()?;
                dialogue.update(State::AddNumber(num+number)).await?;
                bot.send_message(msg.chat.id, format!("Number added, now {}", num+number)).await?;
            }
            Command::Sub(number_str) => {
                let number: i32 = number_str.parse()?;
                dialogue.update(State::SubNumber(num-number)).await?;
                bot.send_message(msg.chat.id, format!("Number subed, now {}", num-number)).await?;
            }
            Command::Battle => {
                let keyboard: Vec<Vec<InlineKeyboardButton>> = vec![vec![
                InlineKeyboardButton::callback(".".to_owned(), "0".to_owned()),
                InlineKeyboardButton::callback(".".to_owned(), "1".to_owned()),
                InlineKeyboardButton::callback(".".to_owned(), "2".to_owned())],
                vec![
                InlineKeyboardButton::callback(".".to_owned(), "3".to_owned()),
                InlineKeyboardButton::callback(".".to_owned(), "4".to_owned()),
                InlineKeyboardButton::callback(".".to_owned(), "5".to_owned())],
                vec![
                InlineKeyboardButton::callback(".".to_owned(), "6".to_owned()),
                InlineKeyboardButton::callback(".".to_owned(), "7".to_owned()),
                InlineKeyboardButton::callback(".".to_owned(), "8".to_owned())]];
                bot.send_message(msg.chat.id, "Please, send /get or /reset")
                .reply_markup(InlineKeyboardMarkup::new(keyboard))
                .await?;
            }
        },
        Err(_) => {
            bot.send_message(msg.chat.id, "Please, send /get or /reset").await?;
        }
    }

    Ok(())
}

