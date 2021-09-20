use std::{convert::TryInto, fmt, fs::File};
use iced::{Align, Button, Column, Element, HorizontalAlignment, Radio, Row, Sandbox, Settings, Text, VerticalAlignment, button, executor, futures::io::{Empty, Window}, image::{Handle, viewer::Renderer}};
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

    fn view(&mut self) -> Element<Message> {
        Button::new(&mut self.button, Text::new(self.value.to_string()).horizontal_alignment(HorizontalAlignment::Center).vertical_alignment(VerticalAlignment::Center))
            .height(iced::Length::Units(100))
            .width(iced::Length::Units(100))
            .on_press(Message::CellPressed(self.position))
            .into()
    }
}

#[derive(Debug, Clone)]
enum FirstPlayer {
    Human,
    Computer,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Hardness {
    Random,
    Hard,
}

impl Hardness {
    fn view(&mut self) -> Element<Message> {
        let hardness = [Hardness::Hard, Hardness::Random];
        hardness.iter().cloned().fold(
            Column::new(),
            |choices, h| {
                choices.push(Radio::new(
                    h,
                    h.to_string(),
                    Some(*self),
                    Message::ToggleHardness,
                ))
            }).into()
    }
}

impl fmt::Display for Hardness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Hardness::Hard => write!(f, "Hard"),
            Hardness::Random => write!(f, "Random"),
        }
    }
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
    moves: u8,
}

#[derive(Debug, Clone)]
enum Message {
    CellPressed((usize, usize)),
    ToggleHardness(Hardness),
    ToggleFirstPlayer(FirstPlayer),
}

impl Tictactoe {
    fn neighbours((i,j): (usize, usize)) -> Vec<[(usize, usize); 3]>{
        let mut neighbours = vec![];
        let row: [(usize, usize); 3] = (0..3).map(|j| {(i,j)}).collect::<Vec<_>>().try_into().unwrap();
        neighbours.push(row);
        let col: [(usize, usize); 3] = (0..3).map(|i| {(i,j)}).collect::<Vec<_>>().try_into().unwrap();
        neighbours.push(col);

        if i == j {
            let diag: [(usize, usize); 3] = (0..3).map(|i| {(i,i)}).collect::<Vec<_>>().try_into().unwrap();
            neighbours.push(diag);
        }
        if  i + j == 2 {
            let diag: [(usize, usize); 3] = (0..3).map(|i| {(i, 2-i)}).collect::<Vec<_>>().try_into().unwrap();
            neighbours.push(diag);
        }
        neighbours
    }

    fn check(&mut self, (i,j): (usize,usize), cell_value: CellValue) {
        let neighbours: Vec<[(usize, usize); 3]> = Tictactoe::neighbours((i,j));
        let won = neighbours.iter().any(|line| {
            line.iter().all(|(i,j)| {
                self.cells[*i][*j].value == cell_value
            })
        });

        if won {
            self.finished = true;
            self.winner = match cell_value {
                CellValue::X => Winner::X,
                CellValue::O => Winner::O,
                _ => Winner::None,
            };
        } else if self.moves >= 9 {
            self.finished = true;
            self.winner = Winner::None;
        }
    }

    fn next(&mut self) {
        let self_value = match self.first_player {
            FirstPlayer::Human => CellValue::O,
            FirstPlayer::Computer => CellValue::X,
        };
        let opp_value = match self.first_player {
            FirstPlayer::Computer => CellValue::O,
            FirstPlayer::Human => CellValue::X,
        };
        if self.hardness == Hardness::Hard {
            let mut next_move: Option<(usize, usize)> = None;
            let lines:[[(usize, usize); 3]; 8] = [
                [(0,0),(0,1),(0,2)], [(1,0),(1,1),(1,2)], [(2,0),(2,1),(2,2)],
                [(0,0),(1,0),(2,0)], [(0,1),(1,1),(2,1)], [(0,2),(1,2),(2,2)],
                [(0,0),(1,1),(2,2)], [(0,2),(1,1),(2,0)]
            ];
            // First, check if the game can be finished in one move
            for line in lines {
                let self_count: usize = line.iter().filter(|(i,j)| {
                    self.cells[*i][*j].value == self_value
                }).collect::<Vec<_>>().len();
                if self_count == 2 {
                    for (i,j) in line {
                        if self.cells[i][j].value == CellValue::Empty {
                            next_move = Some((i,j));
                        }
                    }
                    if next_move != None {
                        break;
                    }
                }
            }
            if next_move == None {
                // Next, check if any row, column or diagonal has 2 cells of the opponent's value
                // and a single Empty cell. This will be a forced move
                for line in lines {
                    let opp_count = line.iter().filter(|(i,j)| {
                        self.cells[*i][*j].value == opp_value
                    }).collect::<Vec<_>>().len();
                    if opp_count == 2 {
                        for (i,j) in line {
                            if self.cells[i][j].value == CellValue::Empty {
                                next_move = Some((i,j));
                            }
                        }
                        if next_move != None {
                            break;
                        }
                    }
                }
            }

            if next_move == None {
                // Arrange cells by priority otherwise
                let cells:[(usize, usize); 9] = [(1,1), (0,0), (0,2), (2,0), (2,2), (0,1), (1,0), (1,2), (2,1)];
                for (i,j) in cells {
                    if self.cells[i][j].value == CellValue::Empty {
                        next_move = Some((i,j));
                        break;
                    }
                }
            }

            if let Some((i,j)) = next_move {
                self.cells[i][j].update(self_value);
                self.moves += 1;
                self.check((i,j), self_value);
            }
        } else {
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
            self.cells[i][j].update(self_value);
            self.moves += 1;
            self.check((i,j), self_value);
        }
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
        Tictactoe { cells, first_player: FirstPlayer::Human, hardness: Hardness::Hard, finished: false, winner: Winner::None, moves: 0 }
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
        if let Message::CellPressed((i,j)) = message {
            if !self.finished {
                let value = match self.first_player {
                    FirstPlayer::Human => CellValue::X,
                    FirstPlayer::Computer => CellValue::O,
                };
                if self.cells[i][j].update(value) {
                    self.moves += 1;
                    self.check((i,j), value);
                    if !self.finished {
                        self.next();
                    }
                }
            }
        } else if let Message::ToggleHardness(hardness) = message {
            self.hardness = hardness;
        } else if let Message::ToggleFirstPlayer(player) = message {
            self.first_player = player;
        }
    }

    fn view(&mut self) -> Element<Self::Message> {

        Column::new()
            .push(
                self.hardness.view()
            )
            .push(
                self.cells.iter_mut().enumerate().fold(
                    Column::new().max_width(300).max_height(300).align_items(Align::Center),
                    |col, (_, row)| {
                        col.push(
                            row.iter_mut().enumerate().fold(
                                Row::new().align_items(Align::Center),
                                |row, (_, cell) | {
                                    row.push(cell.view())
                                }
                            )
                        )
                    }
                )
            ).into()
    }
}