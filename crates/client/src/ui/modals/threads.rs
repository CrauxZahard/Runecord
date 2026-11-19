//! Threads: the browser, the create sheet and the post tag editor.

use super::super::*;

/// The thread browser Discord opens from the thread icon in a channel header.
/// It lists the active threads first, archived ones behind a tab, and filters
/// by name.
pub(crate) fn threads_sheet(s: &App, p: Palette) -> El {
    let parent_id = s
        .channel(s.active_channel)
        .and_then(|channel| channel.thread.as_ref().map(|info| info.parent_id))
        .unwrap_or(s.active_channel);
    let parent = s.channel(parent_id).cloned();
    let parent_name = parent
        .as_ref()
        .map(|channel| channel.name.clone())
        .unwrap_or_else(|| "channel".to_string());
    let forum = parent
        .as_ref()
        .is_some_and(|channel| channel.kind == ChannelKind::Forum);
    let archived_tab = s.thread_tab == 1;
    let query = s.thread_search.text.trim().to_lowercase();
    let mut threads: Vec<Channel> = s
        .guild_channels(s.active_guild)
        .iter()
        .filter(|channel| {
            channel.kind == ChannelKind::Thread
                && channel.thread.as_ref().map(|info| info.parent_id) == Some(parent_id)
        })
        .filter(|channel| {
            channel
                .thread
                .as_ref()
                .is_some_and(|info| info.archived == archived_tab)
        })
        .filter(|channel| query.is_empty() || channel.name.to_lowercase().contains(&query))
        .cloned()
        .collect();
    threads.sort_by_key(|thread| std::cmp::Reverse(thread.last_message_id.unwrap_or(thread.id)));

    let mut list = col().w_full().gap(2.0);
    for thread in &threads {
        let id = thread.id;
        let info = thread.thread.clone();
        let messages = info.as_ref().map(|info| info.message_count).unwrap_or(0);
        let preview = info
            .as_ref()
            .map(|info| info.preview.clone())
            .filter(|preview| !preview.is_empty())
            .unwrap_or_else(|| "No messages yet.".to_string());
        let archived = info.as_ref().is_some_and(|info| info.archived);
        let unread = s.unread.get(&id).copied().unwrap_or(0) > 0;
        list = list.child(
            row()
                .key(format!("thread-row-{id}"))
                .w_full()
                .h(56.0)
                .px(8.0)
                .radius(6.0)
                .gap(10.0)
                .center_y()
                .hover(p.card_hover)
                .role("button")
                .label(format!("Open thread {}", thread.name))
                .on_press(move |s: &mut App| {
                    s.open_thread(id);
                    s.modal = Modal::None;
                })
                .on_secondary_press(move |s: &mut App, press, _: &mut rune::Tx| {
                    s.context_menu = Some(crate::app::ContextMenu {
                        kind: crate::app::ContextMenuKind::Thread(id),
                        anchor: crate::app::ContextMenuAnchor::Pointer {
                            x: press.x,
                            y: press.y,
                        },
                    });
                })
                .child(
                    el().size(36.0, 36.0)
                        .radius(18.0)
                        .center()
                        .fill(p.input)
                        .child(
                            icon(
                                if archived {
                                    &icons::LOCK
                                } else {
                                    &icons::THREAD
                                },
                                18.0,
                            )
                            .color(p.text_faint),
                        ),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            row()
                                .gap(6.0)
                                .center_y()
                                .children(
                                    unread.then(|| el().size(7.0, 7.0).radius(4.0).fill(p.accent)),
                                )
                                .child(
                                    text(thread.name.clone())
                                        .font(theme::SMALL_MED)
                                        .color(if archived {
                                            p.text_muted
                                        } else {
                                            p.text_bright
                                        })
                                        .truncate(),
                                ),
                        )
                        .child(
                            text(preview)
                                .font(theme::TINY)
                                .color(p.text_faint)
                                .truncate(),
                        ),
                )
                .child(
                    row()
                        .gap(8.0)
                        .center_y()
                        .children(
                            thread
                                .owner_id
                                .and_then(|owner| s.user(owner))
                                .map(|owner| avatar_status(owner, 20.0, p, p.modal, false)),
                        )
                        .child(
                            col()
                                .items_end()
                                .gap(3.0)
                                .child(
                                    text(format!(
                                        "{messages} message{}",
                                        if messages == 1 { "" } else { "s" }
                                    ))
                                    .font(theme::TINY)
                                    .color(p.text_faint),
                                )
                                .children(archived.then(|| {
                                    text(if forum { "CLOSED" } else { "ARCHIVED" })
                                        .font(theme::TINY)
                                        .color(p.yellow)
                                })),
                        ),
                ),
        );
    }
    if threads.is_empty() {
        list = list.child(
            col()
                .w_full()
                .py(40.0)
                .items_center()
                .gap(6.0)
                .child(icon(&icons::THREAD, 36.0).color(p.text_faint))
                .child(
                    text(if archived_tab {
                        if query.is_empty() {
                            "No archived threads"
                        } else {
                            "No archived threads match"
                        }
                    } else if query.is_empty() {
                        "No active threads"
                    } else {
                        "No threads match"
                    })
                    .font(theme::SMALL)
                    .color(p.text_muted),
                )
                .child(
                    text(if archived_tab {
                        "Closed threads land here."
                    } else {
                        "Start one from a message or the button below."
                    })
                    .font(theme::TINY)
                    .color(p.text_faint),
                ),
        );
    }

    let can_start = parent
        .as_ref()
        .is_some_and(|channel| !forum && s.can_start_thread(channel));
    let mut sheet = col()
        .key("threads-sheet")
        .w(520.0)
        .h(520.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(16.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .items_center()
                .gap(8.0)
                .child(
                    col()
                        .grow(1.0)
                        .gap(1.0)
                        .child(text("Threads").font(theme::TITLE).color(p.text_bright))
                        .child(
                            text(format!("#{parent_name}"))
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                )
                .child(close_button(p)),
        )
        .child(
            row()
                .w_full()
                .h(34.0)
                .radius(4.0)
                .fill(p.input)
                .px(8.0)
                .gap(6.0)
                .center_y()
                .child(icon(&icons::SEARCH, 14.0).color(p.text_faint))
                .child(
                    text_input(&s.thread_search, theme::SMALL)
                        .key("thread-search")
                        .grow(1.0)
                        .role("textbox")
                        .label("Search threads")
                        .placeholder("Search threads", p.text_faint)
                        .on_edit(|s: &mut App| &mut s.thread_search),
                ),
        )
        .child(
            row()
                .w_full()
                .gap(6.0)
                .child(thread_tab(
                    "Active",
                    s.thread_tab == 0,
                    "thread-tab-active",
                    0,
                    p,
                ))
                .child(thread_tab(
                    "Archived",
                    s.thread_tab == 1,
                    "thread-tab-archived",
                    1,
                    p,
                )),
        )
        .child(
            list.scroll(ScrollSpec {
                stick_bottom: false,
            })
            .grow(1.0)
            .min_h(0.0)
            .shrink(),
        );
    if can_start && !archived_tab {
        let parent = parent_id;
        sheet = sheet.child(
            row().w_full().justify_end().child(
                el().key("threads-new")
                    .h(34.0)
                    .px(14.0)
                    .radius(4.0)
                    .gap(6.0)
                    .center()
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .role("button")
                    .label("Create a thread")
                    .on_press(move |s: &mut App| {
                        s.profile_edit.clear();
                        s.thread_body.clear();
                        s.thread_tag_ids.clear();
                        s.modal = Modal::ThreadCreate {
                            parent,
                            message: None,
                        };
                    })
                    .child(icon(&icons::PLUS, 16.0).color(Color::WHITE))
                    .child(
                        text("New Thread")
                            .font(theme::SMALL_MED)
                            .color(Color::WHITE),
                    ),
            ),
        );
    }
    sheet
}

fn thread_tab(label: &str, selected: bool, key: &'static str, index: u8, p: Palette) -> El {
    el().key(key)
        .h(28.0)
        .px(12.0)
        .radius(14.0)
        .center()
        .role("tab")
        .label(label)
        .checked(selected)
        .fill(if selected { p.accent } else { p.input })
        .hover(if selected {
            p.accent_hover
        } else {
            p.card_hover
        })
        .on_press(move |s: &mut App| {
            s.thread_tab = index;
            s.thread_search.clear();
        })
        .child(text(label).font(theme::TINY).color(if selected {
            Color::WHITE
        } else {
            p.text_muted
        }))
}

/// A tag row used by the create and edit sheets. `prefix` keeps the layout
/// keys apart when both sheets have been open.
fn tag_picker(
    s: &App,
    tags: &[runecord_protocol::ForumTag],
    prefix: &'static str,
    p: Palette,
) -> El {
    let mut chips = row().gap(6.0).flex_wrap();
    for tag in tags {
        let id = tag.id;
        let selected = s.thread_tag_ids.contains(&id);
        let tint = theme::avatar_color(id.rem_euclid(10) as u32);
        chips = chips.child(
            el().key(format!("{prefix}-{id}"))
                .h(28.0)
                .px(10.0)
                .radius(14.0)
                .center()
                .role("checkbox")
                .label(format!("Tag post with {}", tag.name))
                .checked(selected)
                .fill(if selected { tint.alpha(0.3) } else { p.input })
                .hover(if selected {
                    tint.alpha(0.4)
                } else {
                    p.card_hover
                })
                .on_press(move |s: &mut App| s.toggle_thread_tag(id))
                .child(
                    text(format!("{}{}", tag.emoji, tag.name))
                        .font(theme::TINY)
                        .color(if selected {
                            p.text_bright
                        } else {
                            p.text_muted
                        }),
                ),
        );
    }
    chips
}

pub(crate) fn thread_create_sheet(s: &App, p: Palette, parent: Id, message: Option<Id>) -> El {
    let parent_channel = s.channel(parent).cloned().unwrap_or_default();
    let forum = parent_channel.kind == ChannelKind::Forum;
    let guidelines = forum && !parent_channel.topic.trim().is_empty();
    let tags: Vec<runecord_protocol::ForumTag> = s
        .forum_tags
        .iter()
        .filter(|tag| tag.channel_id == parent)
        .cloned()
        .collect();
    let name = s.profile_edit.text.trim();
    let body = s.thread_body.text.trim();
    // A thread started from a message already has its first message; only the
    // name is left to write, the way Discord's Create Thread popover works.
    let from_message = message.is_some();
    let ready = !name.is_empty()
        && (!forum || from_message || !body.is_empty())
        && (!guidelines || s.forum_guidelines_ok);
    let mut form = col()
        .key("thread-create")
        .w(480.0)
        .max_h(640.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(20.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .items_center()
                .gap(8.0)
                .child(
                    col()
                        .grow(1.0)
                        .gap(1.0)
                        .child(
                            text(if forum { "Create a Post" } else { "New Thread" })
                                .font(theme::TITLE)
                                .color(p.text_bright),
                        )
                        .child(
                            text(format!("#{}", parent_channel.name))
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                )
                .child(close_button(p)),
        );
    if guidelines {
        form = form.child(
            col()
                .w_full()
                .radius(6.0)
                .fill(p.input)
                .padding(12.0)
                .gap(6.0)
                .child(
                    text("POST GUIDELINES")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    text(parent_channel.topic.clone())
                        .font(theme::SMALL)
                        .color(p.text)
                        .max_lines(4),
                )
                .child(
                    row()
                        .w_full()
                        .h(28.0)
                        .gap(8.0)
                        .center_y()
                        .role("checkbox")
                        .label("I have read the post guidelines")
                        .checked(s.forum_guidelines_ok)
                        .on_press(|s: &mut App| s.forum_guidelines_ok = !s.forum_guidelines_ok)
                        .child(
                            el().size(16.0, 16.0)
                                .radius(3.0)
                                .border(
                                    1.0,
                                    if s.forum_guidelines_ok {
                                        p.accent
                                    } else {
                                        p.divider
                                    },
                                )
                                .fill(if s.forum_guidelines_ok {
                                    p.accent
                                } else {
                                    Color::TRANSPARENT
                                })
                                .center()
                                .children(
                                    s.forum_guidelines_ok
                                        .then(|| icon(&icons::CHECK, 11.0).color(Color::WHITE)),
                                ),
                        )
                        .child(
                            text("I have read the post guidelines")
                                .font(theme::TINY)
                                .color(p.text_muted),
                        ),
                ),
        );
    }
    form = form
        .child(
            text(if forum { "TITLE" } else { "THREAD NAME" })
                .font(theme::SECTION)
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
                    if forum {
                        "What is this post about?"
                    } else {
                        "Thread name"
                    },
                    p.text_faint,
                )
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.profile_edit)
                .on_submit(move |s: &mut App, _: &mut rune::Tx| {
                    s.create_thread_named(parent, message);
                }),
        );
    if !from_message {
        form = form
            .child(
                text(if forum { "MESSAGE" } else { "FIRST MESSAGE" })
                    .font(theme::SECTION)
                    .color(p.text_muted),
            )
            .child(
                text_input(&s.thread_body, theme::BODY)
                    .key("thread-body")
                    .w_full()
                    .h(if forum { 140.0 } else { 96.0 })
                    .radius(4.0)
                    .fill(p.rail)
                    .color(p.text)
                    .role("textbox")
                    .label("First message")
                    .placeholder(
                        if forum {
                            "Share the details of your post"
                        } else {
                            "Optional first message"
                        },
                        p.text_faint,
                    )
                    .padding(10.0)
                    .on_edit(|s: &mut App| &mut s.thread_body),
            );
    }
    if forum && !tags.is_empty() {
        form = form
            .child(text("TAGS").font(theme::SECTION).color(p.text_muted))
            .child(tag_picker(s, &tags, "create-thread-tag", p));
    }
    form.child(
        row()
            .w_full()
            .justify_end()
            .gap(8.0)
            .center_y()
            .child(
                el().key("thread-create-cancel")
                    .h(34.0)
                    .px(12.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Cancel")
                    .hover(p.rail)
                    .on_press(|s: &mut App| s.modal = Modal::None)
                    .child(text("Cancel").font(theme::SMALL_MED).color(p.text_muted)),
            )
            .child(
                el().key("thread-create-submit")
                    .h(34.0)
                    .px(14.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label(if forum {
                        "Create post"
                    } else {
                        "Create thread"
                    })
                    .fill(if ready { p.accent } else { p.input })
                    .hover(if ready { p.accent_hover } else { p.input })
                    .on_press(move |s: &mut App| s.create_thread_named(parent, message))
                    .child(
                        text(if forum {
                            "Create Post"
                        } else {
                            "Create Thread"
                        })
                        .font(theme::SMALL_MED)
                        .color(if ready {
                            Color::WHITE
                        } else {
                            p.text_faint
                        }),
                    ),
            ),
    )
}

/// Edit the tags on an existing post, which Discord offers as "Edit Tags" in
/// a post's context menu.
pub(crate) fn thread_tags_sheet(s: &App, p: Palette, thread: Id) -> El {
    let parent = s
        .channel(thread)
        .and_then(|channel| channel.thread.as_ref().map(|info| info.parent_id))
        .unwrap_or(0);
    let tags: Vec<runecord_protocol::ForumTag> = s
        .forum_tags
        .iter()
        .filter(|tag| tag.channel_id == parent)
        .cloned()
        .collect();
    let mut form = col()
        .key("thread-tags-sheet")
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
                    text("Edit Tags")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Tags help people find this post. Choose up to five.")
                .font(theme::SMALL)
                .color(p.text_muted),
        );
    if tags.is_empty() {
        form = form.child(
            text("This forum has no tags yet.")
                .font(theme::SMALL)
                .color(p.text_faint)
                .padding(8.0),
        );
    } else {
        form = form.child(tag_picker(s, &tags, "edit-thread-tag", p));
    }
    form.child(
        row()
            .w_full()
            .justify_end()
            .gap(8.0)
            .child(
                el().key("thread-tags-cancel")
                    .h(34.0)
                    .px(12.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Cancel")
                    .hover(p.rail)
                    .on_press(|s: &mut App| {
                        s.thread_tag_ids.clear();
                        s.modal = Modal::None;
                    })
                    .child(text("Cancel").font(theme::SMALL_MED).color(p.text_muted)),
            )
            .child(
                el().key("thread-tags-save")
                    .h(34.0)
                    .px(14.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Save tags")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .on_press(|s: &mut App| s.save_thread_tags())
                    .child(text("Save").font(theme::SMALL_MED).color(Color::WHITE)),
            ),
    )
}
