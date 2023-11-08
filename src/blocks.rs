use crate::ratatui::style::Style;
use crate::ratatui::widgets::block::Block;
use crate::ratatui::widgets::block::BorderType;
use crate::ratatui::widgets::block::Padding;
use crate::ratatui::widgets::Borders;

use crate::ratatui::prelude::Alignment;
use crate::ratatui::widgets::block::title::Position;
#[derive(Debug, Clone)]
pub struct TextAreaTitle {
    pub content: String,
    pub alignment: Option<crate::ratatui::prelude::Alignment>,
    pub position: Option<crate::ratatui::widgets::block::title::Position>,
}
#[derive(Debug, Clone)]
pub struct TextAreaBlock {
    /// List of titles
    titles: Vec<TextAreaTitle>,
    /// The style to be patched to all titles of the block
    titles_style: Style,
    /// The default alignment of the titles that don't have one
    titles_alignment: Alignment,
    /// The default position of the titles that don't have one
    titles_position: Position,

    /// Visible borders
    borders: Borders,
    /// Border style
    border_style: Style,
    /// Type of the border. The default is plain lines but one can choose to have rounded or
    /// doubled lines instead.
    border_type: BorderType,

    /// Widget style
    style: Style,
    /// Block padding
    padding: Padding,
}

impl TextAreaBlock {
    // pub fn from_block(block: Block) -> TextAreaBlock {
    //     TextAreaBlock {
    //         titles: block.titles(),
    //         titles_style: block.titles_style(),
    //         titles_alignment: block.titles_alignment(),
    //         titles_position: block.titles_position(),
    //         borders: block.borders(),
    //         border_style: block.border_style(),
    //         border_type: block.border_type(),
    //         style: block.style(),
    //         padding: block.padding(),
    //     }
    // }
    pub fn default() -> TextAreaBlock {
        TextAreaBlock {
            titles: Vec::new(),
            titles_style: Style::default(),
            titles_alignment: Alignment::default(),
            titles_position: Position::default(),
            borders: Borders::NONE,
            border_style: Style::default(),
            border_type: BorderType::Plain,
            style: Style::default(),
            padding: Padding::default(),
        }
    }
    pub fn to_block(&self) -> Block {
        let mut bl = Block::default()
            .borders(self.borders)
            .border_style(self.border_style)
            .border_type(self.border_type)
            .style(self.style)
            .padding(self.padding)
            .title_style(self.titles_style)
            .title_alignment(self.titles_alignment)
            .title_position(self.titles_position);
        if self.titles.len() > 0 {
            bl = bl.title(self.titles[0].content.clone());
        };
        bl
    }
    pub fn borders(mut self, borders: Borders) -> TextAreaBlock {
        self.borders = borders;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> TextAreaBlock {
        self.titles.push(TextAreaTitle {
            content: title.into(),
            alignment: None,
            position: None,
        });
        self
    }
    pub fn style(mut self, style: Style) -> TextAreaBlock {
        self.style = style;
        self
    }
}
