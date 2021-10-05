use shogi::{square::Square, Color, Piece, PieceType};
use std::collections::{HashMap, HashSet};
use yew::prelude::*;

mod square;
use square::SquareView;

pub struct Board {
    props: BoardProps,
}

#[derive(Properties, Clone, PartialEq)]
pub struct BoardProps {
    pub pieces: HashMap<Square, Piece>,
    pub ghost_piece: Option<Piece>,
    pub move_origin_candidates: HashSet<Square>,
    pub move_destination_candidates: HashSet<Square>,
    pub move_origin: Option<Square>,
    pub move_destination: Option<Square>,
    pub previous_move_origin: Option<Square>,
    pub previous_move_destination: Option<Square>,
    pub is_asking_promotion_with_piece: Option<Piece>,
    pub is_white_in_check: bool,
    pub is_black_in_check: bool,
    pub on_square_click: Callback<Square>,
    pub on_choose_promote: Callback<bool>,
}

impl Component for Board {
    type Message = ();
    type Properties = BoardProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let changed = self.props != props;
        self.props = props;
        changed
    }

    fn view(&self) -> Html {
        html! {
            <div class=classes!("board")>
                <div class=classes!("board-dot", "top-left")></div>
                <div class=classes!("board-dot", "top-right")></div>
                <div class=classes!("board-dot", "bottom-left")></div>
                <div class=classes!("board-dot", "bottom-right")></div>
                {
                    for Square::iter().enumerate().map(|(key, square)| {
                        let is_move_origin_candidate = self.props.move_origin_candidates.contains(&square);
                        let is_move_destination_candidate = self.props.move_destination_candidates.contains(&square);
                        let is_move_origin=self.props.move_origin.contains(&square);
                        let is_move_destination=self.props.move_destination.contains(&square);
                        let is_previous_move_origin=self.props.previous_move_origin.contains(&square);
                        let is_previous_move_destination=self.props.previous_move_destination.contains(&square);
                        let is_asking_promotion_with_piece=is_move_destination.then_some(()).and(self.props.is_asking_promotion_with_piece);
                        let is_in_check = self.props.pieces.get(&square)
                            .filter(|piece| piece.piece_type == PieceType::King)
                            .filter(|piece| match piece.color {
                                Color::White => self.props.is_white_in_check,
                                Color::Black => self.props.is_black_in_check,
                            })
                            .is_some();
                        html! {
                            <SquareView
                                key=key
                                piece=self.props.pieces.get(&square).map(|p| *p)
                                ghost_piece=self.props.ghost_piece
                                is_move_origin_candidate=is_move_origin_candidate
                                is_move_destination_candidate=is_move_destination_candidate
                                is_move_origin=is_move_origin
                                is_move_destination=is_move_destination
                                is_previous_move_origin=is_previous_move_origin
                                is_previous_move_destination=is_previous_move_destination
                                is_asking_promotion_with_piece=is_asking_promotion_with_piece
                                is_in_check=is_in_check
                                on_click=self.props.on_square_click.reform(move |_| square)
                                on_choose_promote=self.props.on_choose_promote.clone()
                            />
                        }
                    })
                }
            </div>
        }
    }
}
