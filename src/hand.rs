use crate::piece::PieceView;

use shogi::{Color, Piece, PieceType};
use yew::prelude::*;

pub struct Hand {
    props: HandProps,
}

#[derive(Properties, Clone, PartialEq)]
pub struct HandProps {
    pub color: Color,
    pub pieces: Vec<HandPiece>,
    pub selection: Option<PieceType>,
    pub can_select: bool,
    pub on_piece_click: Callback<PieceType>,
}

#[derive(Clone, PartialEq)]
pub struct HandPiece {
    pub piece_type: PieceType,
    pub count: u8,
}

impl Component for Hand {
    type Message = ();
    type Properties = HandProps;

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
        let mut hand_classes = classes!("hand", self.props.color.to_string().to_lowercase());
        if self.props.can_select {
            hand_classes.push("selectable");
        }
        html! {
            <div class=hand_classes>
                {
                    for self.props.pieces.iter().enumerate().map(|(key, hand_piece)| {
                        let piece = Piece {
                            piece_type: hand_piece.piece_type,
                            color: self.props.color,
                        };
                        let mut hand_piece_classes = classes!("hand-piece");
                        if hand_piece.count == 0 {
                            hand_piece_classes.push("none");
                        }
                        if let Some(selected_piece_type) = self.props.selection {
                            if hand_piece.piece_type == selected_piece_type {
                                hand_piece_classes.push("selected");
                            }
                        }
                        html! {
                            <div
                                class=hand_piece_classes
                                key=key
                                onclick=self.props.on_piece_click.reform(move |_| piece.piece_type)
                            >
                                <PieceView piece=Some(piece) />
                                <div class="count">
                                    {hand_piece.count}
                                </div>
                            </div>
                        }
                    })
                }
            </div>
        }
    }
}
