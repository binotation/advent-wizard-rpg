mod ui;

use advent_wizard_rpg::{Battle, Spell};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Rect},
    prelude::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style, Stylize},
    symbols::{border, scrollbar},
    text::{Line, Masked, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Widget, Wrap,
    },
    Frame,
};
use std::{
    io,
    time::{Duration, Instant},
};
use ui::{tui, CenterPosition};

impl<'a> App<'a> {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => self.exit = true,
                        KeyCode::Char('j') | KeyCode::Down => {
                            self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                            self.vertical_scroll_state =
                                self.vertical_scroll_state.position(self.vertical_scroll);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                            self.vertical_scroll_state =
                                self.vertical_scroll_state.position(self.vertical_scroll);
                        }
                        KeyCode::Char('a') => self.select_left(),
                        KeyCode::Char('d') => self.select_right(),
                        KeyCode::Char('w') => self.select_up(),
                        KeyCode::Char('s') => self.select_down(),
                        KeyCode::Enter => self.step_game(),
                        _ => (),
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Percentage(70),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ])
        .split(area);

    let row1 = Layout::horizontal([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)]).split(chunks[1]);

        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(self.game_output.len());


        let create_block = |title: &'static str| Block::bordered().gray().title(title.bold());

        let title = Block::new()
            .title_alignment(Alignment::Center)
            .title("Wizard RPG from Advent of Code 2015".bold());
        frame.render_widget(title, chunks[0]);

        let paragraph = Paragraph::new(self.game_output.clone())
            .gray()
            .block(create_block("Game Screen"))
            .scroll((self.vertical_scroll as u16, 0));
        frame.render_widget(paragraph, row1[1]);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalLeft)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
            row1[1].inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.vertical_scroll_state,
        );

        let wizard_info = Paragraph::new(format!("Hitpoints: {}
Mana: {}", self.game.get_wizard().hitpoints, self.game.get_wizard().mana))
        .gray()
        .block(create_block("Game Screen"));
    frame.render_widget(wizard_info, row1[0]);

        let row2 = Layout::horizontal([Constraint::Percentage(50); 2]).split(chunks[2]);
        let row3 = Layout::horizontal([Constraint::Percentage(50); 2]).split(chunks[3]);
        if self.spell_selected == 0 {}
        frame.render_widget(
            CenterPosition::default()
                .text("Magic Missile")
                .block(title_block(self.spell_selected == 0)),
            row2[0],
        );
        frame.render_widget(
            CenterPosition::default()
                .text("Drain")
                .block(title_block(self.spell_selected == 1)),
            row2[1],
        );
        frame.render_widget(
            CenterPosition::default()
                .text("Poison")
                .block(title_block(self.spell_selected == 2)),
            row3[0],
        );
        frame.render_widget(
            CenterPosition::default()
                .text("Shield")
                .block(title_block(self.spell_selected == 3)),
            row3[1],
        );
        frame.render_widget(
            CenterPosition::default()
                .text("Recharge")
                .block(title_block(
                    self.spell_selected == 4 || self.spell_selected == 5,
                )),
            chunks[4],
        );
    }

    fn select_left(&mut self) {
        if self.spell_selected % 2 != 0 {
            self.spell_selected -= 1;
        }
    }

    fn select_right(&mut self) {
        if self.spell_selected % 2 == 0 {
            self.spell_selected += 1;
        }
    }

    fn select_up(&mut self) {
        if self.spell_selected != 0 && self.spell_selected != 1 {
            self.spell_selected -= 2;
        }
    }

    fn select_down(&mut self) {
        if self.spell_selected != 4 && self.spell_selected != 5 {
            self.spell_selected += 2;
        }
    }

    fn output_screen(&mut self, line: String) {
        self.game_output.push(Line::from(line));
        if self.game_output.len() > 6 {
            self.vertical_scroll += 1;
        }
    }

    fn output_win_loss(&mut self, won: bool) {
        if won {
            self.output_screen("Glory! Your magic has defeated the enemy!".to_string());
        } else {
            self.output_screen("Grief... You have fallen to the enemy...".to_string());
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
        if let Err(err) = outcome {
            self.output_screen("You cannot cast that spell!".to_string());
            return;
        } else if let Ok(outcome) = outcome {
            if let Some(won) = outcome {
                self.output_win_loss(won);
            } else {
                self.output_screen(format!("You cast {:#?}", spell_cast));
            }
        }

        let outcome = self.game.boss_turn();
        if let Some(won) = outcome {
            self.output_win_loss(won)
        } else {
            self.output_screen("The enemy has attacked you for 8 hitpoints".to_string());
        }
    }
}

fn title_block(selected: bool) -> Block<'static> {
    let color = if selected {
        Color::Magenta
    } else {
        Color::Gray
    };
    Block::bordered().fg(color)
}

#[derive(Debug, Default)]
pub struct App<'a> {
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    exit: bool,
    spell_selected: usize,
    game: Battle,
    game_output: Vec<Line<'a>>,
}

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);
    tui::restore()?;
    app_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }
}
