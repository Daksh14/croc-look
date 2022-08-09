use std::time::Instant;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

type Components<'a> = (Paragraph<'a>, Paragraph<'a>, Paragraph<'a>);

// Handle TUI components
pub struct CrocTui {
    layout: Layout,
    pub code_block: String,
    pub expanding: String,
    pub scroll: Scroll,
}

// Handle scrolling
pub struct Scroll {
    vs: usize,
    vs_state: u16,
    hs: usize,
    hs_state: u16,
}

impl CrocTui {
    pub fn new(code_block: String, expanding: String) -> Self {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            );

        let mut lines = 0;
        let mut line_array = Vec::new();

        code_block.lines().for_each(|e| {
            line_array.push(e.len() - 1);
            lines += 1;
        });

        let scroll = Scroll::new(lines, line_array.iter().max().copied().unwrap_or(0));

        Self {
            layout,
            code_block,
            expanding,
            scroll,
        }
    }

    pub fn code_block(&mut self, code_block: String) -> &mut Self {
        self.code_block = code_block;
        self
    }

    pub fn components(&self, now: Instant) -> (Paragraph, Paragraph, Paragraph) {
        let paragraph = Paragraph::new(&*self.expanding)
            .block(Block::default().title("Expanding").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let code_block = Paragraph::new(&*self.code_block)
            .block(Block::default().title("Segment").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .scroll(self.scroll.offset());

        let info = Paragraph::new(format!(
            "Took: {}ms, q for quit, r for reload, arrow keys for scrolling",
            now.elapsed().as_millis(),
        ))
        .block(Block::default().title("Info").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: true });

        (paragraph, code_block, info)
    }

    pub fn scroll_code_block_and_render<B: Backend>(&self, f: &mut Frame<B>, now: Instant) {
        let components = self.components(now);
        let scrolled = components.1.clone().scroll(self.scroll.offset());

        self.render(f, (components.0, scrolled, components.2))
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, components: Components) {
        let chunks = self.layout.split(f.size());

        let (paragraph, code_block, info) = components;

        f.render_widget(paragraph, chunks[0]);
        f.render_widget(code_block, chunks[1]);
        f.render_widget(info, chunks[2]);
    }
}

impl Scroll {
    pub const fn new(vs: usize, hs: usize) -> Self {
        Self {
            vs,
            vs_state: 0,
            hs,
            hs_state: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.vs > self.vs_state.into() {
            self.vs_state += 1
        }
    }

    pub fn scroll_right(&mut self, screen_width: u16) {
        let screen_width = usize::from(screen_width);
        if screen_width < self.hs {
            // + 10 is for some extra room for readability
            let amount_you_can_scroll = self.hs - screen_width + 10;
            if usize::from(self.hs_state) <= amount_you_can_scroll {
                self.hs_state += 1
            }
        }
    }

    pub fn scroll_down(&mut self) {
        self.vs_state = self.vs_state.saturating_sub(1);
    }

    pub fn scroll_left(&mut self) {
        self.hs_state = self.hs_state.saturating_sub(1);
    }

    pub const fn offset(&self) -> (u16, u16) {
        (self.vs_state, self.hs_state)
    }
}
