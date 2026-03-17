use serenity::{
    all::ButtonStyle,
    builder::{CreateActionRow, CreateButton},
};

pub fn create_music_buttons() -> Vec<CreateActionRow> {
    let clear_button = CreateButton::new("clear")
        .label("📋 Clear")
        .style(ButtonStyle::Danger);

    let resume_button = CreateButton::new("resume")
        .label("▶️ Resume")
        .style(ButtonStyle::Success);

    let pause_button = CreateButton::new("pause")
        .label("⏸️ Pause")
        .style(ButtonStyle::Primary);

    let skip_button = CreateButton::new("skip")
        .label("⏭️ Skip")
        .style(ButtonStyle::Primary);

    let loop_button = CreateButton::new("loop")
        .label("🔄 Loop")
        .style(ButtonStyle::Primary);

    let row = CreateActionRow::Buttons(vec![
        clear_button,
        resume_button,
        pause_button,
        skip_button,
        loop_button,
    ]);

    vec![row]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_music_buttons_returns_single_row() {
        let buttons = create_music_buttons();
        assert_eq!(buttons.len(), 1);
    }

    #[test]
    fn test_music_buttons_has_correct_count() {
        let buttons = create_music_buttons();
        if let CreateActionRow::Buttons(ref button_vec) = buttons[0] {
            assert_eq!(button_vec.len(), 5, "Should have 5 buttons: clear, resume, pause, skip, loop");
        } else {
            panic!("Expected CreateActionRow::Buttons variant");
        }
    }
}
