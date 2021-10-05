use crate::piece::PieceView;

use shogi::Piece;
use yew::prelude::*;

pub struct SquareView {
    props: SquareProps,
}

#[derive(Properties, Clone, PartialEq)]
pub struct SquareProps {
    pub piece: Option<Piece>,
    pub ghost_piece: Option<Piece>,
    pub is_move_origin_candidate: bool,
    pub is_move_destination_candidate: bool,
    pub is_move_origin: bool,
    pub is_move_destination: bool,
    pub is_previous_move_origin: bool,
    pub is_previous_move_destination: bool,
    pub is_asking_promotion_with_piece: Option<Piece>,
    pub is_in_check: bool,
    pub on_click: Callback<()>,
    pub on_choose_promote: Callback<bool>,
}

impl Component for SquareView {
    type Message = ();
    type Properties = SquareProps;

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
        let mut square_classes = classes!("square");
        if self.props.is_move_destination_candidate {
            square_classes.push("move-destination-candidate");
        }
        if self.props.is_move_origin_candidate {
            square_classes.push("move-origin-candidate");
        }
        if self.props.is_move_origin {
            square_classes.push("move-origin");
        }
        if self.props.is_move_destination {
            square_classes.push("move-destination");
        }
        if self.props.is_previous_move_origin {
            square_classes.push("previous-move-origin");
        }
        if self.props.is_previous_move_destination {
            square_classes.push("previous-move-destination");
        }
        if self.props.is_in_check {
            square_classes.push("in-check");
        }

        let displayed_piece = if let Some(piece) = self.props.piece {
            Some(piece)
        } else if self.props.is_move_destination_candidate {
            square_classes.push("ghost");
            self.props.ghost_piece
        } else {
            None
        };

        html! {
            <div
                class=square_classes
                onclick=self.props.on_click.reform(|_| ())
            >
                <PieceView piece=displayed_piece />
                {
                    if let Some(piece) = self.props.is_asking_promotion_with_piece {
                        html!{
                            <div class="promote-prompt">
                                <div
                                    class="promote-option"
                                    onclick=self.props.on_choose_promote.reform(|_| true)
                                >
                                    <div>
                                        <PieceView piece=piece.promote().expect("Piece can be promoted") />
                                    </div>
                                </div>
                                <div
                                    class="promote-option"
                                    onclick=self.props.on_choose_promote.reform(|_| false)
                                >
                                    <div>
                                        <PieceView piece=piece />
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html!{}
                    }
                }
            </div>
        }
    }
}
