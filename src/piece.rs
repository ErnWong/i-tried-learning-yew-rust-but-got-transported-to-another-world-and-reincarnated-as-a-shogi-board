use shogi::{Color, Piece, PieceType};
use yew::prelude::*;

pub struct PieceView {
    props: PieceProps,
}

#[derive(Properties, Clone, PartialEq)]
pub struct PieceProps {
    pub piece: Option<Piece>,
}

impl Component for PieceView {
    type Message = ();
    type Properties = PieceProps;

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
        if let Some(piece) = self.props.piece {
            let piece_type_classname = match piece.piece_type {
                PieceType::King => "king",
                PieceType::Rook => "rook",
                PieceType::Bishop => "bishop",
                PieceType::Gold => "gold",
                PieceType::Silver => "silver",
                PieceType::Knight => "knight",
                PieceType::Lance => "lance",
                PieceType::Pawn => "pawn",
                PieceType::ProRook => "promoted-rook",
                PieceType::ProBishop => "promoted-bishop",
                PieceType::ProSilver => "promoted-silver",
                PieceType::ProKnight => "promoted-knight",
                PieceType::ProLance => "promoted-lance",
                PieceType::ProPawn => "promoted-pawn",
            };
            let color_name = match piece.color {
                Color::White => "white",
                Color::Black => "black",
            };
            html! {
                <div class=classes!("piece", piece_type_classname, color_name)>
                </div>
            }
        } else {
            html! {}
        }
    }
}
