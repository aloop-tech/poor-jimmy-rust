pub mod clear;
pub mod damnit_jimmy;
pub mod help;
pub mod join;
pub mod leave;
pub mod list;
pub mod r#loop;
pub mod move_song;
pub mod now_playing;
pub mod pause;
pub mod ping;
pub mod play;
pub mod remove;
pub mod resume;
pub mod search;
pub mod shuffle;
pub mod skip;

use crate::data::{Data, Error};

/// Every slash command the bot registers
pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        clear::clear(),
        damnit_jimmy::damnit_jimmy(),
        help::help(),
        join::join(),
        leave::leave(),
        list::list(),
        r#loop::loop_song(),
        move_song::move_song(),
        now_playing::now_playing(),
        pause::pause(),
        ping::ping(),
        play::play(),
        remove::remove(),
        resume::resume(),
        search::search(),
        shuffle::shuffle(),
        skip::skip(),
    ]
}
