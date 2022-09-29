extern crate hw1_chess;
extern crate piston_window;

use std::f32::consts::PI;

// use adamvib_chess as chess;
use hw1_chess::{self as chess, piece};
use piston_window::{color::hex, *};

const CHESS_SQUARE_LENGTH: u32 = 90;
const GUI_LENGTH: u32 = CHESS_SQUARE_LENGTH * 8;
const GUI_HEIGHT: u32 = CHESS_SQUARE_LENGTH * 8;

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Big chess big money", [GUI_LENGTH, GUI_HEIGHT])
            .exit_on_esc(true)
            .build()
            .unwrap();

    let images = load_images(&mut window);

    let mut board = chess::Board::default();

    let mut mouse_cursor = [0.0, 0.0];
    let mut selected_square = 65;
    let mut dragged_square = 65;
    let mut hovered_square = 65;

    let mut butt: Option<ButtonArgs> = None;
    while let Some(e) = window.next() {
        let pieces = board.get_pices();

        e.mouse_cursor(|x| {
            mouse_cursor = x;
        });

        e.button(|x| {
            if let Button::Mouse(b) = x.button {
                butt = Some(x);

                hovered_square = (mouse_cursor[0] / (CHESS_SQUARE_LENGTH as f64)).floor() as u32
                    + 8 * (mouse_cursor[1] / (CHESS_SQUARE_LENGTH as f64)).floor() as u32;

                if x.state == ButtonState::Release && dragged_square != 65 {
                    if hovered_square != dragged_square {
                        let res = board.make_move(&hw1_chess::cmove::CMove {
                            from: dragged_square as u8,
                            to: hovered_square as u8,
                            promote_to: hw1_chess::piece::PieceType::Queen,
                        });

                        selected_square = 65;
                        if res.is_err() {
                            println!("make_move error: {}", res.err().unwrap());
                        } else {
                            println!("{}", board.to_fen());
                        }
                    }

                    dragged_square = 65;
                }
                if x.state == ButtonState::Press {
                    let p = pieces[hovered_square as usize];
                    if p.is_some() && p.unwrap().get_color() == board.get_active_color() {
                        dragged_square = hovered_square;
                        selected_square = hovered_square;
                    } else if selected_square < 64 {
                        selected_square = 65;
                    }
                }
            }
        });

        window.draw_2d(&e, |c, g, _device| {
            for x in 0..8 {
                for y in 0..8 {
                    let mut color = if (x + 7 * y) % 2 == 0 {
                        hex("f0d9b5")
                    } else {
                        hex("b58863")
                    };

                    if x + y * 8 == selected_square {
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

                        let img = &images[idx as usize];
                        let size = img.get_size();

                        let mut coords = [
                            ((x as f64 + 0.5) * (CHESS_SQUARE_LENGTH as f64) - (size.0 / 2) as f64)
                                as f64,
                            ((y as f64 + 0.5) * (CHESS_SQUARE_LENGTH as f64) - (size.1 / 2) as f64)
                                as f64,
                            (size.0) as f64,
                            (size.1) as f64,
                        ];

                        if piece_idx == dragged_square {
                            coords = [
                                mouse_cursor[0] - (size.0 / 2) as f64,
                                mouse_cursor[1] - (size.1 / 2) as f64,
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

            if selected_square < 64 && pieces[selected_square as usize].is_some() {
                let mut moves = Vec::with_capacity(21);
                board.get_legal_moves_for_idx(selected_square as u8, &mut moves);

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
