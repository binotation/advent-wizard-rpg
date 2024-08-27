mod ui;

use advent_wizard_rpg::{Battle, Spell};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::Alignment,
    prelude::{Constraint, Layout, Margin},
    style::{Color, Style, Stylize},
    symbols::scrollbar,
    text::Line,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::{
    io,
    time::{Duration, Instant},
};
use ui::{tui, CenterPosition};

#[derive(Debug, Default)]
struct App<'a> {
    exit: bool,
    game: Battle,
    spell_selected: usize,
    event_window_scroll_state: ScrollbarState,
    event_window_scroll: usize,
    event_window_height: u16,
    event_window_text: Vec<Line<'a>>,
}

impl<'a> App<'a> {
    /// runs the application's main loop until the user quits
    fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            // Poll for remaining time until next tick
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key.code);
                }
            }
            // Update last tick
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            // Quit
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            // Scroll event window
            KeyCode::Char('j') | KeyCode::Down => self.event_window_scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.event_window_scroll_up(),
            // Change spell selection
            KeyCode::Char('w') => self.select_spell_up(),
            KeyCode::Char('a') => self.select_spell_left(),
            KeyCode::Char('s') => self.select_spell_down(),
            KeyCode::Char('d') => self.select_spell_right(),
            // Cast selected spell
            KeyCode::Enter => self.step_game(),
            _ => (),
        }
    }

    fn event_window_scroll_down(&mut self) {
        self.event_window_scroll = self.event_window_scroll.saturating_add(1);
        self.event_window_scroll_state = self
            .event_window_scroll_state
            .position(self.event_window_scroll);
    }

    fn event_window_scroll_up(&mut self) {
        self.event_window_scroll = self.event_window_scroll.saturating_sub(1);
        self.event_window_scroll_state = self
            .event_window_scroll_state
            .position(self.event_window_scroll);
    }

    fn select_spell_up(&mut self) {
        if self.spell_selected != 0 && self.spell_selected != 1 {
            self.spell_selected -= 2;
        }
    }

    fn select_spell_left(&mut self) {
        if self.spell_selected % 2 != 0 {
            self.spell_selected -= 1;
        }
    }

    fn select_spell_down(&mut self) {
        if self.spell_selected != 4 && self.spell_selected != 5 {
            self.spell_selected += 2;
        }
    }

    fn select_spell_right(&mut self) {
        if self.spell_selected % 2 == 0 {
            self.spell_selected += 1;
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Partition UI chunks
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Percentage(70),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ])
        .split(area);

        // Title
        let title = Block::new()
            .title_alignment(Alignment::Center)
            .title("Wizard RPG".bold());
        frame.render_widget(title, chunks[0]);

        // Layout for 3 game screens
        let game_windows = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(chunks[1]);

        // Middle game screen: scrollable text displaying game events
        let event_window = Paragraph::new(self.event_window_text.clone())
            .gray()
            .block(
                Block::bordered()
                    .gray()
                    .title("Events".bold())
                    .title_alignment(Alignment::Center),
            )
            .scroll((self.event_window_scroll as u16, 0));
        // Scrollbar state
        self.event_window_scroll_state = self
            .event_window_scroll_state
            .content_length(self.event_window_text.len());
        self.event_window_height = game_windows[1].height;
        frame.render_widget(event_window, game_windows[1]); // Middle window
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalLeft)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
            game_windows[1].inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.event_window_scroll_state,
        );

        // Left game screen: text displaying Wizard information
        let wizard_info = Paragraph::new(format!(
            "Hitpoints: {}
Mana: {}",
            self.game.get_wizard().get_hitpoints(),
            self.game.get_wizard().get_mana(),
        ))
        .gray()
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .light_blue()
                .title("Wizard".bold().gray())
                .title_alignment(Alignment::Center),
        );
        frame.render_widget(wizard_info, game_windows[0]);

        // Right game screen: text displaying Boss information
        let boss_info = Paragraph::new(format!(
            "Hitpoints: {}
Mana: {}",
            self.game.get_wizard().get_hitpoints(),
            self.game.get_wizard().get_mana(),
        ))
        .gray()
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .light_red()
                .title("Boss".bold().gray())
                .title_alignment(Alignment::Center),
        );
        frame.render_widget(boss_info, game_windows[2]);

        // Spell selection table
        let spell_row1 = Layout::horizontal([Constraint::Percentage(50); 2]).split(chunks[2]);
        let spell_row2 = Layout::horizontal([Constraint::Percentage(50); 2]).split(chunks[3]);

        frame.render_widget(
            Self::create_spell_select_button(Spell::MagicMissile, self.spell_selected == 0),
            spell_row1[0],
        );
        frame.render_widget(
            Self::create_spell_select_button(Spell::Drain, self.spell_selected == 1),
            spell_row1[1],
        );
        frame.render_widget(
            Self::create_spell_select_button(Spell::Poison, self.spell_selected == 2),
            spell_row2[0],
        );
        frame.render_widget(
            Self::create_spell_select_button(Spell::Shield, self.spell_selected == 3),
            spell_row2[1],
        );
        frame.render_widget(
            Self::create_spell_select_button(
                Spell::Recharge,
                self.spell_selected == 4 || self.spell_selected == 5,
            ),
            chunks[4],
        );
    }

    fn output_event(&mut self, line: String) {
        self.event_window_text.push(Line::from(line));
        if self.event_window_text.len() > self.event_window_height as usize {
            self.event_window_scroll_down();
        }
    }

    fn output_win_loss_event(&mut self, won: bool) {
        if won {
            self.output_event("Glory! Your magic has defeated the enemy!".to_string());
        } else {
            self.output_event("Grief... You have fallen to the enemy...".to_string());
        }
    }

    fn step_game(&mut self) {
        let spell_cast = match self.spell_selected {
            0 => Spell::MagicMissile,
            1 => Spell::Drain,
            2 => Spell::Poison,
            3 => Spell::Shield,
            4 | 5 => Spell::Recharge,
            _ => unreachable!(),
        };

        let outcome = self.game.wizard_turn(&spell_cast);
        if let Err(_err) = outcome {
            self.output_event("You cannot cast that spell!".to_string());
            return;
        } else if let Ok(outcome) = outcome {
            if let Some(won) = outcome {
                self.output_win_loss_event(won);
            } else {
                self.output_event(format!("You cast {:#?}", spell_cast));
            }
        }

        let outcome = self.game.boss_turn();
        if let Some(won) = outcome {
            self.output_win_loss_event(won)
        } else {
            self.output_event("The enemy has attacked you for 8 hitpoints".to_string());
        }
    }

    fn create_spell_select_button<'b>(spell: Spell, is_selected: bool) -> CenterPosition<'b> {
        let color = if is_selected {
            Color::Magenta
        } else {
            Color::Gray
        };
        CenterPosition::default()
            .text(format!(
                "{}: {} Mana",
                spell.get_display_name(),
                spell.get_mana()
            ))
            .block(Block::bordered().border_style(Style::default().fg(color)))
    }
}

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);
    tui::restore()?;
    app_result
}
