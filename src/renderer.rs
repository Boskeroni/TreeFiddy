use std::cmp::min;
use std::ops::Range;

use tui::Frame;
use tui::backend::Backend;
use tui::style::{Style, Color};
use tui::text::{Span, Text, Spans};
use tui::layout::{Layout, Direction, Constraint, Rect};
use tui::widgets::{ListItem, List, Block, Borders, BorderType, Paragraph, Clear};

use crate::{Ui, Item, Recipe};
use crate::Display;

fn to_listitem(t: String) -> ListItem<'static> {
    ListItem::new(Text::from(Spans::from(t)))
}

fn render_item<B: Backend>(f: &mut Frame<B>, item: &Item, index: usize) {
    // both of these dont have to show, so it cleans it up
    let station = if item.station.is_empty() {
        "None"
    } else {
        &item.station
    };
    let acquisiton = if item.acquisition.is_empty() {
        "crafted"
    } else {
        &item.acquisition
    };
    let title = Span::styled(item.display_name.to_uppercase(), Style::default().fg(Color::Red));

    // ingredients need to be selected in order to show
    // will change so that only one ingredient items will show
    let items = vec![
        format!("id: {}", &item.id),
        format!("station: {}", station),
        format!("ingredients: SELECT FOR INGREDIENTS"),
        format!("quantity: {}", &item.quantity),
        format!("obtaining: {}", acquisiton),
        format!("wiki: {}", &item.wiki),
    ];

    let mut fields: Vec<ListItem> = vec![
        ListItem::new(title),
    ];

    fields.append(&mut items.iter().map(|m| to_listitem(m.clone())).collect::<Vec<ListItem>>());

    fields[index+1] = ListItem::new(Text::from(Span::styled(items[index].clone(), Style::default().fg(Color::Green))));
    let block = List::new(fields)
        .block(
            Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .title(item.display_name.clone())
        );
    f.render_widget(block, f.size())
}

fn get_centered_rect(full_size: Rect, x: u16, y: u16, height: u16, width: u16) -> Rect {
    let column = Layout::default()
        .margin(0)
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(x),
            Constraint::Percentage(width),
            Constraint::Percentage(100-(x+width)),
        ]).split(full_size);

    let row = Layout::default()
        .margin(0)
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(y),
            Constraint::Percentage(height),
            Constraint::Percentage(100-(y+height)),
        ]).split(column[1]);

    return row[1]
}

fn render_crafting<B: Backend>(f: &mut Frame<B>, item: &Recipe) {
    let area = get_centered_rect(f.size(), 10, 10, 80, 80);



    let block = Paragraph::new("sdkhfsdkf")
        .block(
            Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
        );

    f.render_widget(Clear, area);
    f.render_widget(block, area);
}

pub fn render<B: Backend>(f: &mut Frame<B>, ui: &Ui) {
    match &ui.display {
        Display::Query => render_search(f, ui),
        Display::ItemDetails(i) => render_item(f, i, ui.item_i),
        Display::Crafting(i, recipe) => {
            // want to render the item desc as a background cause why not
            render_item(f, i, ui.crafting_i);
            render_crafting(f, recipe);
        }
    }
}

fn format_range(index: usize, length: usize, rows: usize) -> (Range<usize>, usize) {
    let rows = rows - 2;
    // all the songs can be shown normally from bottom -> top
    if length < rows {
        return (0..length, index);
    }
    // index isnt big enough to warrant shifting the songs
    if index < rows / 2 {
        let end = min(rows, length);
        return (0..end, index);
    }
    // shifting the songs is necessary
    let top = min(index + ((rows/2) + 2), length);
    let bottom = top - rows;
    let shown_index = index - bottom;
    return (bottom..top, shown_index);
}

fn render_input_block<B: Backend>(f: &mut Frame<B>, text: &str, area: Rect) {
    let block = Paragraph::new(text)
        .block(
            Block::default()
            .border_type(BorderType::Rounded)
            .title("search bar")
            .borders(Borders::ALL)
        );
    
    f.render_widget(block, area);
}

fn render_search<B: Backend>(f: &mut Frame<B>, ui: &Ui) {
    // currently just focus on one screen
    let all_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Min(3),
            Constraint::Percentage(100),
        ]).split(f.size());


    let (shown, normal_i) = format_range(ui.query_i, ui.results.len(), f.size().height as usize);
    let shown_playlists = &ui.results[shown];

    let mut list_items: Vec<ListItem> = shown_playlists.iter().enumerate().map(|(i, l)| {
        let selected_sym = if normal_i == i {
            ">"
        } else {
            " "
        };

        let line_text = vec![
            Span::styled(selected_sym, Style::default().fg(Color::Red)),
            Span::raw("|"),
            Span::styled(l.display_name.clone(), Style::default().fg(Color::Green)),
        ];

        ListItem::new(Text::from(Spans::from(line_text)))
    }).collect();

    if !ui.search.is_empty() {
        render_input_block(f, &ui.search, all_chunks[0]);
        list_items.insert(0, to_listitem("\n".to_string()));
        list_items.insert(0, to_listitem("\n".to_string()));
    }

    let list_block = List::new(list_items)
        .block(
            Block::default()
            .borders(Borders::ALL)
            .title("results")
            .border_type(BorderType::Rounded)
        );

    f.render_widget(list_block, f.size());
}