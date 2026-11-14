//! Threads.

use super::super::prelude::*;
use super::super::*;

pub(crate) fn threads_sheet(s: &App, p: Palette) -> El {
    let parent = s.active_channel;
    let mut threads: Vec<Channel> = s
        .guild_channels(s.active_guild)
        .iter()
        .filter(|c| {
            c.kind == ChannelKind::Thread && c.thread.as_ref().map(|t| t.parent_id) == Some(parent)
        })
        .cloned()
        .collect();
    threads.sort_by_key(|c| c.id);
    let mut list = col().w_full().gap(4.0);
    for thread in &threads {
        let id = thread.id;
        let info = thread.thread.clone();
        list = list.child(
            row()
                .key(format!("thread-row-{}", id))
                .w_full()
                .h(42.0)
                .px(8.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .hover(p.selected)
                .on_press(move |s: &mut App| {
                    s.open_thread(id);
                    s.modal = Modal::None;
                })
                .child(icon(&icons::THREAD, 18.0).color(p.text_muted))
                .child(
                    text(thread.name.clone())
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .children(info.as_ref().map(|info| {
                    text(format!("{}", info.message_count))
                        .font(theme::TINY)
                        .color(p.text_faint)
                }))
                .children(
                    info.as_ref()
                        .filter(|info| info.archived)
                        .map(|_| text("ARCHIVED").font(theme::TINY).color(p.yellow)),
                ),
        );
    }
    if threads.is_empty() {
        list = list.child(
            text("No threads yet.")
                .font(theme::SMALL)
                .color(p.text_faint)
                .padding(8.0),
        );
    }
    col()
        .key("threads-sheet")
        .w(480.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(16.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .items_center()
                .child(
                    text("Threads")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            list.scroll(ScrollSpec {
                stick_bottom: false,
            })
            .max_h(300.0)
            .shrink(),
        )
        .child(
            el().h(36.0)
                .px(14.0)
                .radius(4.0)
                .fill(p.accent)
                .hover(p.accent_hover)
                .center()
                .on_press(move |s: &mut App| {
                    s.profile_edit.clear();
                    s.modal = Modal::ThreadCreate {
                        parent: s.active_channel,
                        message: None,
                    };
                })
                .child(
                    text("New Thread")
                        .font(theme::SMALL_MED)
                        .color(Color::WHITE),
                ),
        )
}

pub(crate) fn thread_create_sheet(s: &App, p: Palette, parent: Id, message: Option<Id>) -> El {
    let forum = s
        .channel(parent)
        .is_some_and(|channel| channel.kind == ChannelKind::Forum);
    let mut form = col()
        .key("thread-create")
        .w(440.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(20.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .items_center()
                .child(
                    text(if forum {
                        "Create Post"
                    } else {
                        "Create Thread"
                    })
                    .font(theme::TITLE)
                    .color(p.text_bright)
                    .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(if forum {
                "Give the post a title and say what it is about."
            } else {
                "Give the thread a name. It opens under this channel."
            })
            .font(theme::SMALL)
            .color(p.text_muted),
        )
        .child(
            text_input(&s.profile_edit, theme::BODY)
                .key("thread-name")
                .w_full()
                .h(40.0)
                .radius(4.0)
                .fill(p.rail)
                .color(p.text)
                .role("textbox")
                .label(if forum { "Post title" } else { "Thread name" })
                .placeholder(
                    if forum { "Post title" } else { "Thread name" },
                    p.text_faint,
                )
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.profile_edit)
                .on_submit(move |s: &mut App, _: &mut rune::Tx| {
                    s.create_thread_named(parent, message);
                }),
        );
    if forum {
        form = form.child(
            text_input(&s.thread_body, theme::BODY)
                .key("thread-body")
                .w_full()
                .h(110.0)
                .radius(4.0)
                .fill(p.rail)
                .color(p.text)
                .role("textbox")
                .label("Post body")
                .placeholder("What do you want to share?", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.thread_body),
        );
    }
    form.child(
        row()
            .justify_end()
            .gap(8.0)
            .child(
                text("Cancel")
                    .font(theme::SMALL_MED)
                    .color(p.text_muted)
                    .padding(10.0)
                    .on_press(|s: &mut App| s.modal = Modal::None),
            )
            .child(
                el().h(36.0)
                    .px(14.0)
                    .radius(4.0)
                    .key("thread-create-submit")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .center()
                    .on_press(move |s: &mut App| {
                        s.create_thread_named(parent, message);
                    })
                    .child(
                        text(if forum { "Create Post" } else { "Create" })
                            .font(theme::SMALL_MED)
                            .color(Color::WHITE),
                    ),
            ),
    )
}
