use std::collections::HashMap;
use std::usize;
use std::{io, time::Duration, fs};

use crossterm::event::{Event, self, KeyCode, EnableMouseCapture, DisableMouseCapture};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen};
use crossterm::execute;
use serde::Deserialize;
use tui::backend::{CrosstermBackend, Backend};
use tui::Terminal;

mod renderer;

type Recipe<'a> = HashMap<&'a String, u16>;

// decides what needs to be displayed
// 'Crafting' is a subsection of 'ItemDetails'
pub enum Display<'a> {
    Query,
    ItemDetails(&'a Item),
    Crafting(&'a Item, Recipe<'a>),
}

pub struct Ui<'a> {
    pub search: String,
    pub results: Vec<&'a Item>,
    pub query_i: usize,
    pub display: Display<'a>,
    
    // the index of the inner item
    pub item_i: usize,
    pub crafting_i: usize,
}
impl Ui<'_> {
    fn new() -> Self {
        Self {
            search: String::new(),
            results: Vec::new(),
            query_i: 0,
            display: Display::Query,
            item_i: 0,
            crafting_i: 0,
        }
    }
}


#[derive(Clone, Debug, Deserialize)]
pub struct Item {
    pub id: u16,
    pub name: String,
    pub display_name: String,
    pub station: String,
    pub ingredients: Vec<[u16; 2]>,
    pub quantity: u16,
    pub acquisition: String,
    pub wiki: String,
}

fn load_items(path: &str) -> Vec<Item> {
    let data = fs::read_to_string(path).unwrap();
    serde_json::from_str(&data).unwrap()
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = main_loop(&mut terminal);

    if let Err(e) = res {
        println!("{:?}", e);
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn main_loop<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), io::Error> {
    let all_items = load_items("items.json");
    let mut ui = Ui::new();

    ui.results = search_items("", &all_items);

    loop {
        terminal.draw(|f| renderer::render(f, &ui))?;
        if !crossterm::event::poll(Duration::from_millis(20))? {
            continue;
        }
        let key = if let Event::Key(key) = event::read()? {
            key
        } else {
            continue;
        };

        match key.code {
            KeyCode::Enter => {
                // open up craft recipe
                if let Display::ItemDetails(i) = &ui.display {
                    if ui.item_i == 2 {
                        let recipe = get_recipe(&all_items, &i.ingredients);
                        ui.display = Display::Crafting(i, recipe)
                    }
                    continue;
                }

                if ui.results.is_empty() {
                    continue;
                }
                ui.display = Display::ItemDetails(&ui.results[ui.query_i]);
                ui.query_i = 0;
            }
            KeyCode::Backspace => {
                if let Display::Query = &ui.display {
                    ui.search.pop();
                    ui.results = search_items(&ui.search, &all_items);
                    ui.query_i = 0;
                }
            }
            KeyCode::Char(c) => {
                if let Display::Query = &ui.display {
                    ui.search.push(c);
                    ui.results = search_items(&ui.search, &all_items);
                    ui.query_i = 0;
                }
            }
            KeyCode::Down => {
                let (index, length): (&mut usize, usize) = match ui.display {
                    Display::Crafting(i, _) => (&mut ui.crafting_i, i.ingredients.len()),
                    Display::ItemDetails(_) => (&mut ui.item_i, 6),
                    Display::Query => (&mut ui.query_i, ui.results.len()),
                };

                if length == 0 {
                    continue;
                }
                *index += 1;
                if length == *index {
                    *index = 0;
                } 
            }
            KeyCode::Up => {
                let (index, length): (&mut usize, usize) = match ui.display {
                    Display::Crafting(i, _) => (&mut ui.crafting_i, i.ingredients.len()),
                    Display::ItemDetails(_) => (&mut ui.item_i, 6),
                    Display::Query => (&mut ui.query_i, ui.results.len()),
                };

                if length == 0 {
                    continue;
                }
                if *index == 0 {
                    *index = length;
                }
                *index -= 1;
                
            }
            KeyCode::Esc => {
                match ui.display {
                    Display::Crafting(i, _) => ui.display = Display::ItemDetails(i),
                    Display::ItemDetails(_) => ui.display = Display::Query,
                    Display::Query => break, 
                }
            },
            _ => {}
        }
    }

    Ok(())
}

fn search_items<'a>(query: &str, items: &'a Vec<Item>) -> Vec<&'a Item> {
    let lower = &query.to_lowercase();
    items.iter().filter(|i| i.display_name.to_lowercase().contains(lower)).collect()
}

fn get_recipe<'a>(all_items: &'a Vec<Item>, item: &Vec<[u16; 2]>) -> Recipe<'a> {
    let mut ingredients = HashMap::new();

    for ingredient in item {
        let item_i = ingredient[0] as usize;
        let quantity = ingredient[1];

        let name = &all_items[item_i].display_name;

        ingredients.insert(name, quantity);
    }
    ingredients
}