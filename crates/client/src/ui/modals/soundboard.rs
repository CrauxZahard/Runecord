//! Soundboard panel.

use super::super::prelude::*;
use super::super::*;

pub(crate) fn soundboard_sheet(s: &App, p: Palette) -> El {
    let in_voice = s.my_voice_channel().is_some();
    let mut grid = col().w_full().gap(8.0);
    for pair in crate::voice::soundboard_sounds().chunks(2) {
        let mut line = row().w_full().gap(8.0);
        for (name, label, seconds) in pair {
            let name = *name;
            let label = *label;
            let seconds = *seconds;
            line = line.child(
                el().key(format!("soundboard-{name}"))
                    .h(64.0)
                    .grow(1.0)
                    .min_w(0.0)
                    .radius(6.0)
                    .fill(p.rail)
                    .hover(p.rail_hover)
                    .padding(10.0)
                    .gap(4.0)
                    .role("button")
                    .label(format!("Play {label}"))
                    .on_press(move |s: &mut App| {
                        if s.my_voice_channel().is_none() {
                            s.toast("Join a voice channel first");
                            return;
                        }
                        crate::voice::play_soundboard(name);
                        s.toast(format!("Playing {label}"));
                    })
                    .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                    .child(
                        text(format!("{seconds:.1}s"))
                            .font(theme::TINY)
                            .color(p.text_muted),
                    ),
            );
        }
        if pair.len() == 1 {
            line = line.child(el().grow(1.0));
        }
        grid = grid.child(line);
    }

    col()
        .key("soundboard-panel")
        .w(420.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(16.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .h(52.0)
                .px(16.0)
                .gap(10.0)
                .center_y()
                .child(
                    text("Soundboard")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            col()
                .w_full()
                .padding(14.0)
                .gap(10.0)
                .child(
                    text(if in_voice {
                        "Everyone in this voice channel hears these, even while your mic is muted."
                    } else {
                        "Join a voice channel to play these to everyone in it."
                    })
                    .font(theme::TINY)
                    .color(p.text_muted),
                )
                .child(grid),
        )
}
