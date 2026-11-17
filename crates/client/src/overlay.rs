//! In-game overlay: a small always-on-top card listing who is in your voice
//! channel. It is a Rune overlay window, so it stays out of the way and never
//! takes keyboard focus.

use rune::{Align, Cx, El, OverlayHandle, OverlaySpec, ScreenEdge};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OverlayUser {
    pub name: String,
    pub speaking: bool,
    pub muted: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OverlayModel {
    pub channel: String,
    pub users: Vec<OverlayUser>,
    pub muted: bool,
}

impl OverlayModel {
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }
}

static OVERLAY: std::sync::OnceLock<OverlayHandle<OverlayModel>> = std::sync::OnceLock::new();

/// Registers the overlay with the running app. Called once during bootstrap.
pub fn install(app: &mut rune::App<crate::app::App>) {
    let handle = app.overlay(
        OverlaySpec {
            name: "Runecord Overlay",
            size: (300.0, 92.0),
            edge: ScreenEdge::Top,
            margin: 18.0,
            align: Align::End,
            inset: 18.0,
            interactive: false,
            view,
        },
        OverlayModel::default(),
    );
    let _ = OVERLAY.set(handle);
}

/// Pushes the current voice state into the overlay and shows or hides it.
pub fn sync(model: OverlayModel, visible: bool) {
    let Some(overlay) = OVERLAY.get() else {
        return;
    };
    let hide = !visible || model.is_empty();
    overlay.update(move |state, _| {
        *state = model;
    });
    if hide {
        overlay.hide();
    } else {
        overlay.show();
    }
}

pub fn availability() -> &'static str {
    match OVERLAY.get().map(|overlay| overlay.availability()) {
        None => "Not created",
        Some(rune::Availability::Available) => "Ready",
        Some(rune::Availability::Unavailable) => "Unavailable",
        Some(rune::Availability::Unknown) => "Waiting",
    }
}

pub fn view(s: &OverlayModel, _cx: &mut Cx<OverlayModel>) -> El {
    use rune::TextStyle;
    let text_style = TextStyle::new(13.0, 500.0);
    let mut card = rune::col()
        .key("overlay")
        .w_full()
        .radius(10.0)
        .fill(rune::color::oklcha(0.24, 0.01, 270.0, 0.89))
        .border(1.0, rune::Color::WHITE.alpha(0.10))
        .padding(10.0)
        .gap(6.0)
        .child(
            rune::row()
                .w_full()
                .gap(6.0)
                .center_y()
                .child(rune::el().size(7.0, 7.0).radius(4.0).fill(if s.muted {
                    rune::color::oklch(0.63, 0.19, 22.0)
                } else {
                    rune::color::oklch(0.66, 0.15, 150.0)
                }))
                .child(
                    rune::text(if s.channel.is_empty() {
                        "Voice".to_string()
                    } else {
                        s.channel.clone()
                    })
                    .font(text_style)
                    .color(rune::Color::WHITE),
                ),
        );
    for user in s.users.iter().take(4) {
        card = card.child(
            rune::row()
                .w_full()
                .gap(6.0)
                .center_y()
                .child(
                    rune::el()
                        .size(6.0, 6.0)
                        .radius(3.0)
                        .fill(if user.speaking {
                            rune::color::oklch(0.66, 0.15, 150.0)
                        } else {
                            rune::Color::WHITE.alpha(0.16)
                        }),
                )
                .child(
                    rune::text(user.name.clone())
                        .font(text_style)
                        .color(rune::color::oklch(0.90, 0.005, 270.0)),
                )
                .children(user.muted.then(|| {
                    rune::text("muted".to_string())
                        .font(TextStyle::new(11.0, 400.0))
                        .color(rune::color::oklch(0.70, 0.01, 270.0))
                })),
        );
    }
    card
}

#[cfg(test)]
mod tests {
    use super::OverlayModel;

    #[test]
    fn empty_model_means_hidden() {
        assert!(OverlayModel::default().is_empty());
        let model = OverlayModel {
            channel: "General".into(),
            users: vec![super::OverlayUser {
                name: "Nova".into(),
                speaking: true,
                muted: false,
            }],
            muted: false,
        };
        assert!(!model.is_empty());
    }
}
