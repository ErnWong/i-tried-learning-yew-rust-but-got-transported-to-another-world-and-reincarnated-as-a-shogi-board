#![feature(option_result_contains)]
#![feature(bool_to_option)]

use base64::{decode, encode};
use gloo::events::EventListener;
use gloo::timers::callback::Timeout;
use shogi::{
    bitboard::Factory as BBFactory, square::Square, Color, Move, MoveRecord, Piece, PieceType,
    Position,
};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsValue;
use yew::web_sys::{Element, HtmlAudioElement};
use yew::{prelude::*, utils::window};

mod board;
mod hand;
mod piece;
mod shareable_link;

use board::Board;
use hand::{Hand, HandPiece};
use shareable_link::ShareableLink;

fn coord_index_to_full_width_latin(index: u8) -> &'static str {
    match index {
        0 => "１",
        1 => "２",
        2 => "３",
        3 => "４",
        4 => "５",
        5 => "６",
        6 => "７",
        7 => "８",
        8 => "９",
        _ => unreachable!(),
    }
}

fn coord_index_to_japanese_numeral(index: u8) -> &'static str {
    match index {
        0 => "一",
        1 => "二",
        2 => "三",
        3 => "四",
        4 => "五",
        5 => "六",
        6 => "七",
        7 => "八",
        8 => "九",
        _ => unreachable!(),
    }
}

#[derive(Clone, Copy)]
enum Origin {
    SquarePiece(Square),
    HeldPiece(PieceType),
}

impl Origin {
    pub fn square(self) -> Option<Square> {
        match self {
            Self::SquarePiece(square) => Some(square),
            Self::HeldPiece(..) => None,
        }
    }

    pub fn hand_piece_type(self) -> Option<PieceType> {
        match self {
            Self::SquarePiece(..) => None,
            Self::HeldPiece(piece_type) => Some(piece_type),
        }
    }

    pub fn piece(self, position: &Position) -> Option<Piece> {
        match self {
            Origin::SquarePiece(from_square) => *position.piece_at(from_square),
            Origin::HeldPiece(piece_type) => Some(Piece {
                piece_type,
                color: position.side_to_move(),
            }),
        }
    }
}

enum Msg {
    ClickSquare(Square),
    ClickHeldPiece(PieceType, Color),
    ChoosePromote(bool),
    Restart,
    Undo,
    LoadFromUrl,
}

#[derive(Clone, Copy)]
enum MoveIntentBuilder {
    NoIntent,
    WithOrigin { from: Origin },
    WithDestination { from: Origin, to: Square },
}

impl MoveIntentBuilder {
    fn create_sandbox(position: &Position) -> Position {
        let mut sandbox_position = Position::new();
        sandbox_position.set_sfen(&position.to_sfen()).unwrap();
        sandbox_position
    }

    pub fn can_move_to(self, square: Square, position: &Position) -> bool {
        let mut sandbox_position = Self::create_sandbox(&position);
        match self {
            MoveIntentBuilder::WithOrigin { from } => {
                let try_moves = match from {
                    Origin::SquarePiece(from_square) => {
                        vec![
                            Move::Normal {
                                from: from_square,
                                to: square,
                                promote: true,
                            },
                            Move::Normal {
                                from: from_square,
                                to: square,
                                promote: false,
                            },
                        ]
                    }
                    Origin::HeldPiece(piece_type) => {
                        vec![Move::Drop {
                            piece_type,
                            to: square,
                        }]
                    }
                };
                for try_move in try_moves {
                    if sandbox_position.make_move(try_move).is_ok() {
                        return true;
                    }
                }
                false
            }
            _ => panic!(),
        }
    }

    pub fn must_promote(self, position: &Position) -> bool {
        match self {
            MoveIntentBuilder::WithDestination {
                from: Origin::SquarePiece(from),
                to,
            } => {
                let mut sandbox_position = Self::create_sandbox(&position);
                sandbox_position
                    .make_move(Move::Normal {
                        from,
                        to,
                        promote: false,
                    })
                    .is_err()
            }
            MoveIntentBuilder::WithDestination {
                from: Origin::HeldPiece(_),
                to: _,
            } => false,
            _ => panic!(),
        }
    }

    pub fn cant_promote(self, position: &Position) -> bool {
        match self {
            MoveIntentBuilder::WithDestination {
                from: Origin::SquarePiece(from),
                to,
            } => {
                let mut sandbox_position = Self::create_sandbox(&position);
                sandbox_position
                    .make_move(Move::Normal {
                        from,
                        to,
                        promote: true,
                    })
                    .is_err()
            }
            MoveIntentBuilder::WithDestination {
                from: Origin::HeldPiece(_),
                to: _,
            } => true,
            _ => panic!(),
        }
    }

    pub fn move_origin_candidates(self, position: &Position) -> HashSet<Square> {
        match self {
            Self::NoIntent => Square::iter()
                .filter(|square| {
                    position
                        .piece_at(*square)
                        .filter(|piece| piece.color == position.side_to_move())
                        .is_some()
                })
                .collect(),
            Self::WithOrigin { .. } => Default::default(),
            Self::WithDestination { .. } => Default::default(),
        }
    }

    pub fn move_destination_candidates(self, position: &Position) -> HashSet<Square> {
        match self {
            Self::NoIntent => Default::default(),
            Self::WithOrigin { .. } => Square::iter()
                .filter(|square| self.can_move_to(*square, position))
                .collect(),
            Self::WithDestination { .. } => Default::default(),
        }
    }

    pub fn move_origin_square(self) -> Option<Square> {
        match self {
            Self::NoIntent => None,
            Self::WithOrigin { from } => from.square(),
            Self::WithDestination { from, .. } => from.square(),
        }
    }

    pub fn move_origin_hand_piece_type(self) -> Option<PieceType> {
        match self {
            Self::NoIntent => None,
            Self::WithOrigin { from } => from.hand_piece_type(),
            Self::WithDestination { from, .. } => from.hand_piece_type(),
        }
    }

    pub fn move_origin_piece(self, position: &Position) -> Option<Piece> {
        match self {
            Self::NoIntent => None,
            Self::WithOrigin { from } => from.piece(position),
            Self::WithDestination { from, .. } => from.piece(position),
        }
    }

    pub fn move_destination(self) -> Option<Square> {
        match self {
            Self::NoIntent => None,
            Self::WithOrigin { .. } => None,
            Self::WithDestination { to, .. } => Some(to),
        }
    }

    pub fn is_asking_promotion_with_piece(self, position: &Position) -> Option<Piece> {
        match self {
            Self::NoIntent => None,
            Self::WithOrigin { .. } => None,
            Self::WithDestination { from, .. } => from.piece(position),
        }
    }
}

struct Model {
    link: ComponentLink<Self>,
    position: Position,
    move_intent: MoveIntentBuilder,
    move_audio_ref: NodeRef,
    history_bottom_ref: NodeRef,
    _hash_change_listener: EventListener,
}

impl Model {
    fn reset(&mut self) {
        self.position = Position::new();
        self.position
            .set_sfen("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1")
            .expect("Starting position should be valid");
        self.play_move_sound();
    }

    fn try_load_from_url(&mut self) -> Result<(), String> {
        let hash = window()
            .location()
            .hash()
            .map_err(|err| err.as_string().unwrap_or_default())?;
        if hash.is_empty() {
            return Err("No hash".to_string());
        }
        let hash_without_prefix = &hash[1..];
        let decoded = decode(hash_without_prefix).map_err(|err| err.to_string())?;
        let sfen = std::str::from_utf8(&decoded).map_err(|err| err.to_string())?;
        self.position = Position::new();
        self.position
            .set_sfen(sfen)
            .map_err(|err| err.to_string())?;
        self.play_move_sound();
        Ok(())
    }

    fn undo(&mut self) {
        self.position.unmake_move().unwrap();
        self.play_move_sound();
    }

    fn pieces(&self) -> HashMap<Square, Piece> {
        Square::iter()
            .filter_map(|square| Some(square).zip(*self.position.piece_at(square)))
            .collect()
    }

    fn clear_choice(&mut self) {
        self.move_intent = MoveIntentBuilder::NoIntent;
    }

    fn choose_origin(&mut self, from: Origin) {
        self.move_intent = match self.move_intent {
            MoveIntentBuilder::NoIntent => MoveIntentBuilder::WithOrigin { from },
            MoveIntentBuilder::WithOrigin { from: _ } => MoveIntentBuilder::WithOrigin { from },
            _ => panic!(),
        };
    }

    fn choose_destination(&mut self, to: Square) {
        self.move_intent = match self.move_intent {
            MoveIntentBuilder::WithOrigin { from } => {
                MoveIntentBuilder::WithDestination { from, to }
            }
            MoveIntentBuilder::WithDestination { from, to: _ } => {
                MoveIntentBuilder::WithDestination { from, to }
            }
            _ => panic!(),
        };

        // Skip asking whether to promote if there's only one legal option.
        if self.move_intent.cant_promote(&self.position) {
            self.choose_promote(false);
        } else if self.move_intent.must_promote(&self.position) {
            self.choose_promote(true);
        }
    }

    fn choose_promote(&mut self, promote: bool) {
        match self.move_intent {
            MoveIntentBuilder::WithDestination { from, to } => {
                self.play_move_sound();
                let next_move = match from {
                    Origin::SquarePiece(from_square) => Move::Normal {
                        from: from_square,
                        to,
                        promote,
                    },
                    Origin::HeldPiece(piece_type) => Move::Drop { piece_type, to },
                };

                // Scroll after update.
                let history_bottom_ref = self.history_bottom_ref.clone();
                Timeout::new(0, move || {
                    if let Some(history_bottom) = history_bottom_ref.cast::<Element>() {
                        let _ = history_bottom.scroll_into_view();
                    }
                })
                .forget();

                self.position.make_move(next_move).unwrap();
                self.move_intent = MoveIntentBuilder::NoIntent;
            }
            _ => panic!(),
        }
    }

    fn play_move_sound(&self) {
        if let Some(audio) = self.move_audio_ref.cast::<HtmlAudioElement>() {
            let _ = audio.play();
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        BBFactory::init();
        let link_clone = link.clone();
        let mut model = Self {
            link,
            position: Position::new(),
            move_intent: MoveIntentBuilder::NoIntent,
            move_audio_ref: Default::default(),
            history_bottom_ref: Default::default(),
            _hash_change_listener: EventListener::new(&window(), "hashchange", move |_| {
                link_clone.send_message(Msg::LoadFromUrl);
            }),
        };
        if let Err(_error) = model.try_load_from_url() {
            model.reset();
        }
        model
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ClickSquare(square) => match self.move_intent {
                MoveIntentBuilder::NoIntent => {
                    if let Some(piece) = self.position.piece_at(square) {
                        if piece.color == self.position.side_to_move() {
                            self.choose_origin(Origin::SquarePiece(square));
                        }
                    }
                }
                MoveIntentBuilder::WithOrigin { .. } => {
                    if self.move_intent.can_move_to(square, &self.position) {
                        self.choose_destination(square);
                    } else {
                        self.clear_choice();
                    }
                }
                MoveIntentBuilder::WithDestination { .. } => {
                    self.clear_choice();
                }
            },
            Msg::ClickHeldPiece(piece_type, color) => match self.move_intent {
                MoveIntentBuilder::NoIntent => {
                    if color == self.position.side_to_move()
                        && self.position.hand(Piece { piece_type, color }) > 0
                    {
                        self.choose_origin(Origin::HeldPiece(piece_type));
                    } else {
                        self.clear_choice();
                    }
                }
                MoveIntentBuilder::WithOrigin { .. } => self.clear_choice(),
                MoveIntentBuilder::WithDestination { .. } => self.clear_choice(),
            },
            Msg::ChoosePromote(promote) => {
                self.choose_promote(promote);
            }
            Msg::Restart => self.reset(),
            Msg::Undo => self.undo(),
            Msg::LoadFromUrl => {
                let _ = self.try_load_from_url();
            }
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if let Ok(history) = window().history() {
            let new_url = format!("#{}", encode(self.position.to_sfen()));
            let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&new_url));
        }

        let white_hand_pieces: Vec<HandPiece> = PieceType::iter()
            .filter(|piece_type| piece_type.is_hand_piece())
            .map(|piece_type| HandPiece {
                piece_type,
                count: self.position.hand(Piece {
                    piece_type,
                    color: Color::White,
                }),
            })
            .collect();

        let black_hand_pieces: Vec<HandPiece> = PieceType::iter()
            .filter(|piece_type| piece_type.is_hand_piece())
            .map(|piece_type| HandPiece {
                piece_type,
                count: self.position.hand(Piece {
                    piece_type,
                    color: Color::Black,
                }),
            })
            .collect();

        let white_hand_selection = if self.position.side_to_move() == Color::White {
            self.move_intent.move_origin_hand_piece_type()
        } else {
            None
        };

        let black_hand_selection = if self.position.side_to_move() == Color::Black {
            self.move_intent.move_origin_hand_piece_type()
        } else {
            None
        };

        let white_hand_can_select = self.position.side_to_move() == Color::White
            && matches!(self.move_intent, MoveIntentBuilder::NoIntent);
        let black_hand_can_select = self.position.side_to_move() == Color::Black
            && matches!(self.move_intent, MoveIntentBuilder::NoIntent);

        let previous_move_origin = self
            .position
            .move_history()
            .last()
            .and_then(|previous_move| match previous_move {
                MoveRecord::Normal { from, .. } => Some(*from),
                MoveRecord::Drop { .. } => None,
            });

        let previous_move_destination =
            self.position
                .move_history()
                .last()
                .map(|previous_move| match previous_move {
                    MoveRecord::Normal { to, .. } => *to,
                    MoveRecord::Drop { to, .. } => *to,
                });

        html! {
            <>
                <audio preload="auto" ref=self.move_audio_ref.clone()>
                    <source src="./assets/sounds/Move.ogg" type="audio/ogg" />
                    <source src="./assets/sounds/Move.mp3" type="audio/mpeg" />
                </audio>
                <h1>
                    {"I tried learning yew+rust but got transported to another world and reincarnated as a shogi board. "}
                    <a href="https://yew.rs/">
                        {"Yew front-end framework"}
                    </a>
                    {" + "}
                    <a href="https://github.com/nozaq/shogi-rs">
                        {"shogi-rs library"}
                    </a>
                    {" + "}
                    <a href="https://github.com/WandererXII/lishogi">
                        {"lishogi assets"}
                    </a>
                    {" + "}
                    <a href="https://github.com/ErnWong/i-tried-learning-yew-rust-but-got-transported-to-another-world-and-reincarnated-as-a-shogi-board">
                        {"source code"}
                    </a>
                </h1>
                <div class=classes!("game")>
                    <div class="left">
                        <Hand
                            color=Color::White
                            pieces=white_hand_pieces
                            selection=white_hand_selection
                            can_select=white_hand_can_select
                            on_piece_click=self.link.callback(|piece_type|Msg::ClickHeldPiece(piece_type, Color::White))
                        />
                        <div class="fill" />
                        <button
                            disabled=self.position.move_history().is_empty()
                            onclick=self.link.callback(|_| Msg::Undo)
                        >
                            {"Undo"}
                        </button>
                        <button
                            onclick=self.link.callback(|_| Msg::Restart)
                        >
                            {"Restart"}
                        </button>
                        <ShareableLink
                            link_to_share=window().location().href().unwrap_or_default()
                        />
                    </div>
                    <Board
                        pieces=self.pieces()
                        ghost_piece=self.move_intent.move_origin_piece(&self.position)
                        move_origin_candidates=self.move_intent.move_origin_candidates(&self.position)
                        move_destination_candidates=self.move_intent.move_destination_candidates(&self.position)
                        move_origin=self.move_intent.move_origin_square()
                        move_destination=self.move_intent.move_destination()
                        previous_move_origin=previous_move_origin
                        previous_move_destination=previous_move_destination
                        is_asking_promotion_with_piece=self.move_intent
                            .is_asking_promotion_with_piece(&self.position)
                        is_white_in_check=self.position.in_check(Color::White)
                        is_black_in_check=self.position.in_check(Color::Black)
                        on_square_click=self.link.callback(|square| Msg::ClickSquare(square))
                        on_choose_promote=self.link.callback(|promote| Msg::ChoosePromote(promote))
                    />
                    <div class="right">
                        <div class="history">
                            <div class="history-preamble">{ "手合割：平手" }</div>
                            {
                                for self.position.move_history().iter().enumerate().map(|(turn, move_record)| {
                                    let previous_move_destination = self.position.move_history().get(turn - 1).map(|previous_move| match previous_move {
                                        MoveRecord::Normal { to, .. } => to,
                                        MoveRecord::Drop { to, ..} => to,
                                    });
                                    let color = if turn % 2 == 0 {
                                        Color::Black
                                    } else {
                                        Color::White
                                    };
                                    let side = match color {
                                        //Color::Black => "▲",
                                        Color::Black => "☗",
                                        //Color::White => "△",
                                        Color::White => "☖",
                                    };
                                    let destination_square = match move_record {
                                        MoveRecord::Normal { to, .. } => to,
                                        MoveRecord::Drop { to, ..} => to,
                                    };
                                    let destination = if previous_move_destination == Some(destination_square) {
                                        "同　".to_owned()
                                    } else {
                                        let file = coord_index_to_full_width_latin(destination_square.file());
                                        let rank = coord_index_to_japanese_numeral(destination_square.rank());
                                        format!("{}{}", file, rank)
                                    };
                                    let piece_type = match move_record {
                                        MoveRecord::Normal { placed, .. } => placed.piece_type,
                                        MoveRecord::Drop { piece, .. } => piece.piece_type,
                                    };
                                    let piece = match piece_type {
                                        PieceType::King => "玉　",
                                        PieceType::Rook => "飛　",
                                        PieceType::Bishop => "角　",
                                        PieceType::Gold => "金　",
                                        PieceType::Silver => "銀　",
                                        PieceType::Knight => "桂　",
                                        PieceType::Lance => "香　",
                                        PieceType::Pawn => "歩　",
                                        PieceType::ProRook => "龍　",
                                        PieceType::ProBishop => "馬　",
                                        PieceType::ProSilver => "成銀",
                                        PieceType::ProKnight => "成桂",
                                        PieceType::ProLance => "成香",
                                        PieceType::ProPawn => "と　",
                                    };
                                    let movement = match move_record {
                                        MoveRecord::Normal { from, .. } => {
                                            // Pseudo KIF notation
                                            let file = coord_index_to_full_width_latin(from.file());
                                            let rank = coord_index_to_full_width_latin(from.rank());
                                            format!("（{}{}）", file, rank)
                                        },
                                        MoveRecord::Drop { .. } => "　打".to_owned(),
                                    };
                                    let promotion = match move_record {
                                        MoveRecord::Normal { promoted, .. } => {
                                            if *promoted {
                                                "成"
                                            } else {
                                                "　"
                                            }
                                        }
                                        MoveRecord::Drop { .. } => "　"
                                    };
                                    html! {
                                        <div class="history-item" key=turn>
                                            { format!("{}{}{}{}{}\n", side, destination, piece, promotion, movement) }
                                        </div>
                                    }
                                })
                            }
                            <div class="bottom" ref=self.history_bottom_ref.clone() key="bottom" />
                        </div>
                        <Hand
                            color={Color::Black}
                            pieces={black_hand_pieces}
                            selection=black_hand_selection
                            can_select=black_hand_can_select
                            on_piece_click=self.link.callback(|piece_type|Msg::ClickHeldPiece(piece_type, Color::Black))
                        />
                    </div>
                </div>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
