use tui::widgets::{Block, Borders, List, Text, Paragraph, ListState};
use tui::widgets::canvas::{Canvas, Points};
use tui::layout::{Layout, Constraint, Alignment, Direction};
use tui::style::{Style, Color};
use tui::{Frame, backend};
use crossterm::event::KeyCode;

use crate::computer::{Computer, SCR_ADDRESS, KBD_ADDRESS};
use crate::assembler::to_asm;
use crate::utils::get_bit;

#[derive(Eq, PartialEq)]
enum InputMode {
    Normal,
    Editing,
    Keyboard
}

pub struct App {
    filename: String,
    computer: Computer,
    rom_cursor: ListState,
    ram_cursor: ListState,
    input: String,
    input_mode: InputMode,
    pub cursor_pos: Option<(u16, u16)>,
    is_full_screen: bool
}

impl App {
    pub fn new(filename: String, program: Vec<i16>) -> App {
        let mut rom_cursor = ListState::default();
        rom_cursor.select(Some(0));

        let mut ram_cursor = ListState::default();
        ram_cursor.select(Some(0));

        let mut computer = Computer::new();
        for (i, instr) in program.iter().enumerate() {
            computer.rom[i] = Some(*instr);
        }

        App {
            filename,
            computer,
            rom_cursor,
            ram_cursor,
            input: String::new(),
            input_mode: InputMode::Normal,
            cursor_pos: None,
            is_full_screen: false
        }
    }

    pub fn clear_input_event(&mut self) {
        match self.input_mode {
            InputMode::Keyboard => self.computer.memory[KBD_ADDRESS] = 0,
            _ => {}
        }
    }

    pub fn handle_input_event(&mut self, event: KeyCode) -> bool {
        match self.input_mode {
            InputMode::Editing => match event {
                KeyCode::Char(c @ '0'..='9') | KeyCode::Char(c @ '-') => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    let input: String = self.input.drain(..).collect();
                    let cursor = self.ram_cursor.selected().unwrap_or(0);
                    self.computer.memory[cursor] = input.parse().unwrap();
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Esc => {
                    self.input.drain(..);
                    self.input_mode = InputMode::Normal
                }
                _ => {}
            }
            InputMode::Normal => match event {
                KeyCode::Char('n') => {
                    self.computer.step();
                    self.rom_cursor.select(Some(self.computer.pc as usize));
                }
                KeyCode::Char('f') => {
                    self.is_full_screen = !self.is_full_screen;
                }
                KeyCode::Char('j') => {
                    if let Some(i) = self.ram_cursor.selected() {
                        self.ram_cursor.select(Some(i + 1));
                    }
                }
                KeyCode::Char('k') => {
                    if let Some(i) = self.ram_cursor.selected() {
                        if i != 0 {
                            self.ram_cursor.select(Some(i-1));
                        }
                    }
                }
                KeyCode::Char('r') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('b') => {
                    self.input_mode = InputMode::Keyboard;
                }
                KeyCode::Char('q') => {
                    return true;
                }
                _ => {}
            }
            InputMode::Keyboard => match event {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    self.computer.memory[KBD_ADDRESS] = c as i16;
                }
                _ => {}
            }
        }
        false
    }

    pub fn draw<B: backend::Backend>(&mut self, f: &mut Frame<B>) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(4), Constraint::Length(1)])
            .split(f.size());

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(60)])
            .split(rows[0]);

        let column1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(4), Constraint::Length(3)])
            .split(columns[0]);
        
        let column2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(4), Constraint::Length(3), Constraint::Length(3)])
            .split(columns[1]);
        
        let column3 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(columns[2]);

        let text = self.computer.rom.iter().enumerate()
            .map(|(i, v)| {
                let asm = match v {
                    Some(v) => to_asm(*v),
                    None => "".to_owned()
                };
                Text::raw(format!("{:5}| {}", i, asm))
            });
        let rom_block = List::new(text)
            .block(Block::default().title("[ROM]").borders(Borders::ALL))
            .highlight_symbol(">")
            .highlight_style(Style::default().fg(Color::Yellow));

        let text = self.computer.memory.iter().enumerate()
            .map(|(i, v)| Text::raw(format!("{:5}| {}", i, v)));
        let ram_block = List::new(text)
            .block(Block::default().title("[RAM]").borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow));

        let text = [Text::raw(self.computer.d_register.to_string())];
        let d_register_block = Paragraph::new(text.iter())
            .block(Block::default().title("[D Register]").borders(Borders::ALL))
            .alignment(Alignment::Center);

        let text = [Text::raw(self.computer.a_register.to_string())];
        let a_register_block = Paragraph::new(text.iter())
            .block(Block::default().title("[A Register]").borders(Borders::ALL))
            .alignment(Alignment::Center);

        let text = [Text::raw(self.computer.pc.to_string())];
        let pc_block = Paragraph::new(text.iter())
            .block(Block::default().title("[PC]").borders(Borders::ALL))
            .alignment(Alignment::Center);

        let dots = Points {coords: &self.get_screen_dots(), color: Color::White};
        let screen_block = Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("[Screen]"))
            .paint(|ctx| {
                ctx.draw(&dots);
            })
            .x_bounds([0.0, 512.0])
            .y_bounds([0.0, 256.0]);

        let (text, style, cursor_pos) = match self.input_mode {
            InputMode::Editing => {
                let prompt = format!(
                    " Enter the new value at memory address ({}): {}",
                    self.ram_cursor.selected().unwrap_or(0),
                    self.input
                );
                let cursor_pos = Some((prompt.len() as u16, rows[1].y));
                let text = [Text::raw(prompt)];
                let style = Style::default().bg(Color::Yellow).fg(Color::Black);
                (text, style, cursor_pos)
            }
            InputMode::Normal => {
                let text = [Text::raw(format!(" {}", self.filename))];
                let style = Style::default().bg(Color::White).fg(Color::Black);
                let cursor_pos = None;
                (text, style, cursor_pos)
            }
            InputMode::Keyboard => {
                let prompt = format!(" [Keyboard mode] {}", self.computer.memory[KBD_ADDRESS]);
                let text = [Text::raw(prompt)];
                let style = Style::default().bg(Color::Yellow).fg(Color::Black);
                let cursor_pos = None;
                (text, style, cursor_pos)
            }
        };
        let command_input = Paragraph::new(text.iter()).style(style);
        self.cursor_pos = cursor_pos;

        if self.is_full_screen {
            f.render_widget(screen_block, rows[0]);
        } else {
            f.render_stateful_widget(rom_block, column1[0], &mut self.rom_cursor);
            f.render_widget(pc_block, column1[1]);
            f.render_stateful_widget(ram_block, column2[0], &mut self.ram_cursor);
            f.render_widget(d_register_block, column2[1]);
            f.render_widget(a_register_block, column2[2]);
            f.render_widget(screen_block, column3[0]);
            f.render_widget(command_input, rows[1]);
        }
    }

    fn get_screen_dots(&self) -> Vec<(f64, f64)> {
        let mut coords = vec![];
        let mut n = 0;
        for word in self.computer.memory[SCR_ADDRESS..KBD_ADDRESS].iter() {
            if *word != 0 {
                let x = (n % 512) as f64;
                let y = (256 - 1 - (n / 512)) as f64;
                for i in 0..16 {
                    if get_bit(*word, i) {
                        coords.push((x + (i as f64), y));
                    }
                }
            }
            n += 16;
        }
        coords
    }
}
