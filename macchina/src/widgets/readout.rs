use crate::data::{Readout, ReadoutKey};
use crate::theme::Theme;
use std::collections::HashMap;
use tui::buffer::Buffer;
use tui::layout::{Margin, Rect};
use tui::style::{Color, Style};
use tui::text::{Text};
use tui::widgets::{Block, Paragraph, Widget};

pub struct ReadoutList<'a> {
    block: Option<Block<'a>>,
    style: Style,
    items: Vec<Readout<'a>>,
    theme: Box<dyn Theme>,
    block_inner_margin: Margin,
}

impl<'a, 'b> ReadoutList<'a> {
    pub fn new<T>(items: T, theme: Box<dyn Theme>) -> ReadoutList<'a>
    where
        T: Into<Vec<Readout<'a>>>,
    {
        ReadoutList {
            block: None,
            style: Style::default(),
            items: items.into(),
            theme,
            block_inner_margin: Margin {
                horizontal: 0,
                vertical: 0,
            },
        }
    }

    pub fn block(mut self, block: Block<'a>) -> ReadoutList<'a> {
        self.block = Some(block);
        self
    }

    pub fn add_item(mut self, item: Readout<'a>) -> ReadoutList<'a> {
        self.items.push(item);
        self
    }

    pub fn theme(mut self, theme: Box<dyn Theme>) -> ReadoutList<'a> {
        self.theme = theme;
        self
    }

    pub fn style(mut self, style: Style) -> ReadoutList<'a> {
        self.style = style;
        self
    }

    pub fn block_inner_margin(mut self, margin: Margin) -> ReadoutList<'a> {
        self.block_inner_margin = margin;
        self
    }
}

impl<'a> Widget for ReadoutList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let list_area = match &self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                inner_area.inner(&self.block_inner_margin)
            }
            None => area,
        };

        if list_area.width < 1 || list_area.height < 1 {
            return;
        }

        if self.items.is_empty() {
            return;
        }

        let mut height = 0;
        let keys = self.keys_to_text(&self.theme.get_color());
        let max_key_width = Self::get_max_key_width(&keys);
        let themed_separator = Self::get_themed_separator(
            &self.theme.get_separator(),
            &self.theme.get_separator_color(),
        );

        let mut max_line_width: u16 = 0;

        for item in self.items.iter() {
            let readout_data = &item.1;
            let readout_key = keys.get(&item.0).unwrap();

            let list_item_area = Rect {
                x: list_area.x,
                y: list_area.y + height,
                width: list_area.width,
                height: readout_data.height() as u16,
            };

            let constraints =
                self.create_item_constraints(max_key_width, &themed_separator, readout_data);
            let layout = Self::create_layout(&list_item_area, &constraints);

            let total_line_width = constraints.iter().sum::<u16>() + constraints.len() as u16 - 1;
            if total_line_width > max_line_width {
                max_line_width = total_line_width;
            }

            Paragraph::new(readout_key.clone()).render(layout[0], buf);
            Paragraph::new(themed_separator.clone()).render(layout[1], buf);
            Paragraph::new(readout_data.to_owned()).render(layout[3], buf);
            height += readout_data.height() as u16;
        }

        Self::render_block(
            self.block,
            buf,
            area.x,
            area.y,
            height,
            max_line_width,
            &self.block_inner_margin,
        );
    }
}

impl<'a> ReadoutList<'a> {
    fn keys_to_text(&self, key_color: &Color) -> HashMap<ReadoutKey, Text> {
        let color_style = Style::default().fg(key_color.clone());

        self.items
            .iter()
            .map(|i| {
                (
                    i.0,
                    Text::styled(
                        self.theme.key(&i.0, self.theme.default_abbreviation()),
                        color_style,
                    ),
                )
            })
            .collect()
    }

    fn get_max_key_width(keys: &HashMap<ReadoutKey, Text>) -> usize {
        keys.iter().map(|i| i.1.width()).max().unwrap()
    }

    fn render_block(
        block: Option<Block<'a>>,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        content_height: u16,
        content_width: u16,
        margin: &Margin,
    ) {
        if let Some(block) = block {
            block.render(
                Rect {
                    x,
                    y,
                    width: content_width + 2 + margin.horizontal * 2,
                    height: content_height + 2 + margin.vertical * 2,
                },
                buf,
            );
        }
    }

    fn get_themed_separator(separator: &'a str, sep_color: &Color) -> Text<'a> {
        Text::styled(separator, Style::default().fg(sep_color.clone()))
    }
}

impl<'a> ReadoutList<'a> {
    fn create_item_constraints(
        &self,
        max_key_width: usize,
        themed_separator: &Text,
        readout_data: &Text,
    ) -> [u16; 4] {
        [
            max_key_width as u16 + self.theme.get_padding() as u16,
            themed_separator.width() as u16,
            self.theme.get_spacing() as u16,
            readout_data.width() as u16,
        ]
    }
}

impl<'a> ReadoutList<'a> {
    fn create_layout(area: &Rect, constraints: &[u16; 4]) -> Vec<Rect> {
        let mut layout: Vec<Rect> = Vec::with_capacity(constraints.len());
        layout.push(Rect {
            x: area.x,
            y: area.y,
            width: constraints[0],
            height: area.height,
        });

        for (i, &constraint) in constraints.iter().enumerate().skip(1) {
            let previous = layout[i - 1];
            layout.push(Rect {
                x: previous.x + previous.width + 1,
                y: previous.y,
                width: constraint,
                height: area.height,
            });
        }

        layout
    }
}
