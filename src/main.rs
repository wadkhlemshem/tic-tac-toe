use std::fmt;
use iced::{Align, Button, Column, Element, HorizontalAlignment, Radio, Row, Sandbox, Settings, Text, VerticalAlignment, button, executor, futures::io::{Empty, Window}, image::viewer::Renderer};
use rand::prelude::*;

pub fn main() -> iced::Result {
    Tictactoe::run(Settings::default())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CellValue {
    Empty,
    X,
    O,
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CellValue::Empty => write!(f, ""),
            CellValue::X => write!(f, "X"),
            CellValue::O => write!(f, "O"),
        }
    }
}

#[derive(Clone, Copy)]
struct TCell {
    value: CellValue,
    position: (usize, usize),
    button: button::State,
}

type CellMessage = (usize, usize);

impl TCell {
    fn new(p: (usize, usize)) -> Self {
        TCell { value: CellValue::Empty, position: p, button: button::State::new() }
    }

    fn update(&mut self, cell_value: CellValue) -> bool {
        if self.value == CellValue::Empty {
            self.value = cell_value;
            true
        } else {
            false
        }
    }

    fn view(&mut self) -> Element<CellMessage> {
        Button::new(&mut self.button, Text::new(self.value.to_string()).horizontal_alignment(HorizontalAlignment::Center).vertical_alignment(VerticalAlignment::Center))
            .height(iced::Length::Units(100))
            .width(iced::Length::Units(100))
            .on_press(self.position)
            .into()
    }
}

enum FirstPlayer {
    Human,
    Computer,
}

enum Hardness {
    Random,
    Hard,
}

#[derive(Debug)]
enum Winner {
    X,
    O,
    None,
}

struct Tictactoe {
    cells: [[TCell; 3]; 3],
    first_player: FirstPlayer,
    hardness: Hardness,
    finished: bool,
    winner: Winner,
}

#[derive(Debug)]
enum Message {
    CellPressed((usize, usize)),
}

impl Tictactoe {
    fn check(&mut self) {
        let rows:[[(usize,usize); 3]; 3] = [[(0,0),(0,1),(0,2)], [(1,0),(1,1),(1,2)], [(2,0),(2,1),(2,2)]];
        let cols:[[(usize,usize); 3]; 3] = [[(0,0),(1,0),(2,0)], [(0,1),(1,1),(2,1)], [(0,2),(1,2),(2,2)]];
        let diags: [[(usize,usize); 3]; 2] = [[(0,0),(1,1),(2,2)], [(0,2),(1,1),(2,0)]];

        let x_won = rows.iter().any(|row| {
            row.iter().all(|(i,j)| {
                self.cells[*i][*j].value == CellValue::X
            }) ||
            cols.iter().any(|col| {
                col.iter().all(|(i,j)| {
                    self.cells[*i][*j].value == CellValue::X
                })
            }) ||
            diags.iter().any(|diag| {
                diag.iter().all(|(i,j)| {
                    self.cells[*i][*j].value == CellValue::X
                })
            })
        });

        let o_won = rows.iter().any(|row| {
            row.iter().all(|(i,j)| {
                self.cells[*i][*j].value == CellValue::O
            }) ||
            cols.iter().any(|col| {
                col.iter().all(|(i,j)| {
                    self.cells[*i][*j].value == CellValue::O
                })
            }) ||
            diags.iter().any(|diag| {
                diag.iter().all(|(i,j)| {
                    self.cells[*i][*j].value == CellValue::O
                })
            })
        });

        let finished = self.cells.iter().all(|row| {
            row.iter().all(|c| {
                c.value == CellValue::X || c.value == CellValue::O
            })
        });

        if x_won {
            self.finished = true;
            self.winner = Winner::X;
        } else if o_won {
            self.finished = true;
            self.winner = Winner::O;
        } else if finished {
            self.finished = true;
            self.winner = Winner::None;
        }
    }

    fn next(&mut self) {
        let mut empty_cells = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                if self.cells[i][j].value == CellValue::Empty {
                    empty_cells.push((i,j));
                }
            }
        }
        let mut rng = thread_rng();
        let e = rng.gen_range(0..empty_cells.len());
        let (i,j) = empty_cells[e];
        let value = match self.first_player {
            FirstPlayer::Human => CellValue::O,
            FirstPlayer::Computer => CellValue::X,
        };
        self.cells[i][j].update(value);
    }
}

impl Sandbox for Tictactoe {
    // type Executor = executor::Default;
    type Message = Message;
    // type Flags = ();

    fn new() -> Self {
        let mut cells: [[TCell; 3]; 3] = [[TCell::new((0,0)); 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let p: (usize, usize) = (i, j);
                cells[i][j] = TCell::new(p);
            }
        }
        Tictactoe { cells, first_player: FirstPlayer::Human, hardness: Hardness::Hard, finished: false, winner: Winner::None }
    }

    fn title(&self) -> String {
        if self.finished {
            match self.winner {
                Winner::None => String::from("Game Over - Draw"),
                Winner::X => String::from("Game Over - X Won"),
                Winner::O => String::from("Game Over - O Won"),
            }
        } else {
            String::from("Tic Tac Toe")
        }
    }

    fn update(&mut self, message: Message) {
        let Message::CellPressed((i,j)) = message;
        if !self.finished {
            let value = match self.first_player {
                FirstPlayer::Human => CellValue::X,
                FirstPlayer::Computer => CellValue::O,
            };
            if self.cells[i][j].update(value) {
                self.check();
                if !self.finished {
                    self.next();
                    self.check();
                }
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.cells.iter_mut().enumerate().fold(
            Column::new().max_width(300).max_height(300).align_items(Align::Center),
            |col, (usize, row)| {
                col.push(
                    row.iter_mut().enumerate().fold(
                        Row::new().align_items(Align::Center),
                        |row, (usize, cell) | {
                            row.push(cell.view().map(move |message| Message::CellPressed(message)))
                        }
                    )
                )
            }
        ).into()
    }
}