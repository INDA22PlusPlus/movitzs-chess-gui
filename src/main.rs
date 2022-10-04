extern crate hw1_chess;
extern crate piston_window;

mod net;
use clap::Parser;
use net::S2cMessage;
use prost::Message;
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};
use tokio::sync::Mutex as TokioMutex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

// use adamvib_chess as chess;
use hw1_chess::{self as chess, cmove::CMove, Board};
use piston_window::{color::hex, *};

use crate::net::{C2sConnectRequest, C2sMessage, Move};

const CHESS_SQUARE_LENGTH: u32 = 90;
const GUI_LENGTH: u32 = CHESS_SQUARE_LENGTH * 8;
const GUI_HEIGHT: u32 = CHESS_SQUARE_LENGTH * 8;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    mode: String,

    #[arg(long, required(false), required_if_eq("mode", "client"))]
    server_addr: Option<String>,

    #[arg(long, required(false), required_if_eq("mode", "client"))]
    server_port: Option<u16>,
}

#[tokio::main]
async fn main() {
    println!("start");

    let args = Args::parse();

    let board = Arc::new(Mutex::new(Board::new()));

    let board2 = board.clone();

    match args.mode.as_str() {
        "server" => {
            let listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();

            let (socket, _) = listener.accept().await.unwrap();

            let socket = Arc::new(TokioMutex::new(socket));
            let socket_clone = socket.clone();

            tokio::spawn(async move {
                big_server_big_money(socket_clone, board2).await;
            });

            let mut g = Game::new(board, socket);
            g.loopa();
        }
        "client" => {
            let socket = TcpStream::connect(format!(
                "{}:{}",
                args.server_addr.unwrap(),
                args.server_port.unwrap()
            ))
            .await
            .unwrap();

            let socket = Arc::new(TokioMutex::new(socket));

            let socket_clone = socket.clone();
            tokio::spawn(async move {
                small_client_small_money(socket, board2).await;
            });

            let mut g = Game::new(board, socket_clone);
            g.loopa();
        }
        _ => {
            panic!("sad");
        }
    }
}

async fn small_client_small_money(socket: Arc<TokioMutex<TcpStream>>, board: Arc<Mutex<Board>>) {
    let x = C2sMessage {
        msg: Some(net::c2s_message::Msg::ConnectRequest(C2sConnectRequest {
            game_id: 69,
            spectate: false,
        })),
    };
    let mut buf: Vec<u8> = Vec::new();
    prost::Message::encode(&x, &mut buf).unwrap();

    let mut s = socket.lock().await;
    s.write(&buf).await.unwrap();

    loop {
        let mut buf = [0_u8; 512];
        let n = socket.lock().await.read(&mut buf).await.unwrap();

        let x: Result<S2cMessage, _> = prost::Message::decode(&buf[0..n]);

        if x.is_err() {
            println!("invalid msg, dropping client");

            socket
                .lock()
                .await
                .write_buf(&mut "vafan håller du på med\n".as_bytes())
                .await
                .unwrap();

            socket.lock().await.shutdown().await;
            return;
        }

        match x.unwrap().msg.unwrap() {
            net::s2c_message::Msg::Move(x) => {
                board
                    .lock()
                    .unwrap()
                    .make_move(&CMove {
                        from: x.from_square as u8,
                        to: x.to_square as u8,
                        promote_to: hw1_chess::piece::PieceType::Queen,
                    })
                    .unwrap();
            }
            net::s2c_message::Msg::ConnectAck(_) => todo!(),
            net::s2c_message::Msg::MoveAck(_) => todo!(),
        }
    }
}

async fn big_server_big_money(socket: Arc<TokioMutex<TcpStream>>, board: Arc<Mutex<Board>>) {
    loop {
        let mut buf = [0_u8; 512];

        let n = socket.lock().await.read(&mut buf).await.unwrap();

        let x: Result<C2sMessage, _> = prost::Message::decode(&buf[0..n]);

        if x.is_err() {
            println!("invalid msg, dropping client");
            return;
        }

        match x.unwrap().msg.unwrap() {
            net::c2s_message::Msg::Move(x) => {
                println!("got move");
                board.lock().unwrap().make_move(&CMove {
                    from: x.from_square as u8,
                    to: x.to_square as u8,
                    promote_to: hw1_chess::piece::PieceType::Queen,
                });
            }
            net::c2s_message::Msg::ConnectRequest(x) => {
                println!("got conn");
                if x.spectate {
                    return;
                }
            }
        }
    }
}

struct Game {
    board: Arc<Mutex<Board>>,
    socket: Arc<TokioMutex<TcpStream>>,
    mouse_cursor: [f64; 2],
    selected_square: u32,
    dragged_square: u32,
    hovered_square: u32,
    window: PistonWindow,
    images: Vec<G2dTexture>,
}

impl Game {
    fn new(board: Arc<Mutex<Board>>, socket: Arc<TokioMutex<TcpStream>>) -> Self {
        let mut window: PistonWindow =
            WindowSettings::new("Big chess big money", [GUI_LENGTH, GUI_HEIGHT])
                .exit_on_esc(true)
                .build()
                .unwrap();

        let images = load_images(&mut window);

        Game {
            board,
            socket,
            mouse_cursor: [0.0, 0.0],
            hovered_square: 65,
            dragged_square: 65,
            selected_square: 65,
            window,
            images,
        }
    }

    fn loopa(&mut self) {
        while let Some(e) = self.window.next() {
            self.draw(e);
        }
    }

    async fn draw(&mut self, e: Event) {
        let mut board = self.board.lock().unwrap();

        let pieces = board.get_pices();

        e.mouse_cursor(|x| {
            self.mouse_cursor = x;
        });

        e.button(|x| {
            if let Button::Mouse(_b) = x.button {
                self.hovered_square = (self.mouse_cursor[0] / (CHESS_SQUARE_LENGTH as f64)).floor()
                    as u32
                    + 8 * (self.mouse_cursor[1] / (CHESS_SQUARE_LENGTH as f64)).floor() as u32;

                if x.state == ButtonState::Release && self.dragged_square != 65 {
                    if self.hovered_square != self.dragged_square {
                        let mov = chess::cmove::CMove {
                            from: self.dragged_square as u8,
                            to: self.hovered_square as u8,
                            promote_to: chess::piece::PieceType::Queen,
                        };
                        let res = board.make_move(&mov);

                        let ss = self.socket.clone();

                        /*tokio::spawn(async move {
                            let x = C2sMessage {
                                msg: Some(net::c2s_message::Msg::Move(Move {
                                    from_square: mov.from as u32,
                                    to_square: mov.to as u32,
                                    promotion: None,
                                })),
                            };

                            let mut buf = Vec::new();
                            x.encode(&mut buf).unwrap();
                            ss.lock().await.write(&buf).await;
                        });*/

                        self.selected_square = 65;
                        if res.is_err() {
                            println!("make_move error: {}", res.err().unwrap());
                        } else {
                            println!("{}", board.to_fen());
                        }
                    }

                    self.dragged_square = 65;
                }
                if x.state == ButtonState::Press {
                    let p = pieces[self.hovered_square as usize];
                    if p.is_some() && p.unwrap().get_color() == board.get_active_color() {
                        self.dragged_square = self.hovered_square;
                        self.selected_square = self.hovered_square;
                    } else if self.selected_square < 64 {
                        self.selected_square = 65;
                    }
                }
            }
        });

        self.window.draw_2d(&e, |c, g, _device| {
            for x in 0..8 {
                for y in 0..8 {
                    let mut color = if (x + 7 * y) % 2 == 0 {
                        hex("f0d9b5")
                    } else {
                        hex("b58863")
                    };

                    if x + y * 8 == self.selected_square {
                        color = hex("00d9b5");
                    }

                    rectangle(
                        color,
                        [
                            (CHESS_SQUARE_LENGTH * x) as f64,
                            (CHESS_SQUARE_LENGTH * y) as f64,
                            (CHESS_SQUARE_LENGTH as f64),
                            (CHESS_SQUARE_LENGTH as f64),
                        ],
                        c.transform,
                        g,
                    );
                }
            }
            for x in 0..8 {
                for y in 0..8 {
                    let piece_idx = 8 * y + x;
                    let piece = pieces[piece_idx as usize];

                    if piece.is_some() {
                        let piece = piece.unwrap();
                        let idx = 6 * (piece.get_color() as u8) + piece.get_type() as u8;

                        let img = &self.images[idx as usize];
                        let size = img.get_size();

                        let mut coords = [
                            ((x as f64 + 0.5) * (CHESS_SQUARE_LENGTH as f64) - (size.0 / 2) as f64)
                                as f64,
                            ((y as f64 + 0.5) * (CHESS_SQUARE_LENGTH as f64) - (size.1 / 2) as f64)
                                as f64,
                            (size.0) as f64,
                            (size.1) as f64,
                        ];

                        if piece_idx == self.dragged_square {
                            coords = [
                                self.mouse_cursor[0] - (size.0 / 2) as f64,
                                self.mouse_cursor[1] - (size.1 / 2) as f64,
                                (size.0) as f64,
                                (size.1) as f64,
                            ];
                        }

                        Image::new()
                            .rect(coords)
                            .draw(img, &Default::default(), c.transform, g);
                    }
                }
            }

            if self.selected_square < 64 && pieces[self.selected_square as usize].is_some() {
                let mut moves = Vec::with_capacity(21);
                board.get_legal_moves_for_idx(self.selected_square as u8, &mut moves);

                for mv in moves {
                    let (x, y) = (mv.to % 8, mv.to / 8);

                    let size = 5.0;
                    let offset = CHESS_SQUARE_LENGTH as f64 / 2.0 - size / 2.0;
                    circle_arc(
                        hex("4a7055"),
                        size / 2.0,
                        0.0,
                        (PI * 2.0).into(),
                        [
                            (CHESS_SQUARE_LENGTH * x as u32) as f64 + offset,
                            (CHESS_SQUARE_LENGTH * y as u32) as f64 + offset,
                            size,
                            size,
                        ],
                        c.transform,
                        g,
                    );
                }
            }
        });
    }
}

fn load_images(window: &mut PistonWindow) -> Vec<G2dTexture> {
    let paths = [
        "wP.png", "wR.png", "wB.png", "wN.png", "wQ.png", "wK.png", //
        "bP.png", "bR.png", "bB.png", "bN.png", "bQ.png", "bK.png", //
    ];

    let mut result = Vec::with_capacity(12);

    for path in paths {
        result.push(
            Texture::from_path(
                &mut window.create_texture_context(),
                "pieces_png/".to_owned() + path,
                Flip::None,
                &TextureSettings::new(),
            )
            .unwrap(),
        );
    }

    result
}
