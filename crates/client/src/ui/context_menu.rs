//! In-app context menus for channels, members and messages.
//!
//! Rune's native pop-up menu follows the OS theme, so the app draws its own dark menu.
//! These elements reproduce that menu inside the app, so the styling matches the
//! rest of the client on every platform.

use super::*;
use crate::app::{ContextMenu, ContextMenuAnchor, ContextMenuKind};

const MENU_WIDTH: f32 = 232.0;
const MESSAGE_MENU_WIDTH: f32 = 264.0;
/// The quick reactions offered at the top of a message menu, as Discord does.
const QUICK_REACTIONS: [&str; 4] = ["😂", "❤️", "👍", "🔥"];

fn menu_width(kind: &ContextMenuKind) -> f32 {
    match kind {
        ContextMenuKind::Message(_) => MESSAGE_MENU_WIDTH,
        ContextMenuKind::Channel(_) | ContextMenuKind::Member(_) | ContextMenuKind::Thread(_) => {
            MENU_WIDTH
        }
    }
}

/// The menu's tallest expected height, so a menu opened near the bottom edge
/// is moved up rather than clipped.
fn menu_height(kind: &ContextMenuKind) -> f32 {
    match kind {
        ContextMenuKind::Channel(_) => 300.0,
        ContextMenuKind::Member(_) => 390.0,
        ContextMenuKind::Message(_) => 470.0,
        ContextMenuKind::Thread(_) => 300.0,
    }
}

pub(crate) fn context_menu_layer(s: &App, cx: &Cx<App>, menu: &ContextMenu, p: Palette) -> El {
    if let ContextMenuKind::Message(id) = menu.kind
        && s.message(id).is_none()
    {
        return el();
    }
    if let ContextMenuKind::Thread(id) = menu.kind
        && s.channel(id).is_none()
    {
        return el();
    }
    let (window_w, window_h) = cx
        .layout("app")
        .map(|rect| (rect[2], rect[3]))
        .unwrap_or((1360.0, 860.0));
    let width = menu_width(&menu.kind);
    let (anchor_x, anchor_y) = match &menu.anchor {
        ContextMenuAnchor::Pointer { x, y } => (*x, *y),
        // A button that opens a row's menu anchors it to that row's bottom-right.
        ContextMenuAnchor::Element(key) => cx
            .layout(key)
            .map(|rect| (rect[0] + rect[2] - width, rect[1] + rect[3] + 4.0))
            .unwrap_or((window_w - width - 24.0, 60.0)),
    };
    let estimated_h = menu_height(&menu.kind);
    let x = anchor_x.clamp(8.0, (window_w - width - 8.0).max(8.0));
    let y = anchor_y.clamp(8.0, (window_h - estimated_h - 8.0).max(8.0));
    let body = match menu.kind {
        ContextMenuKind::Channel(id) => channel_menu(s, id, p),
        ContextMenuKind::Member(id) => member_menu(s, id, p),
        ContextMenuKind::Message(id) => message_menu(s, id, p),
        ContextMenuKind::Thread(id) => thread_menu(s, id, p),
    };
    el().key("context-menu-layer")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .fill(Color::TRANSPARENT)
        .on_press(|s: &mut App| s.context_menu = None)
        .child(
            col()
                .key("context-menu")
                .pin(Some(x), Some(y), None, None)
                .w(width)
                .radius(4.0)
                .clip_rounded(4.0)
                .fill(theme::color("#111214"))
                .border(1.0, theme::color("#2b2d31"))
                .padding(6.0)
                .gap(1.0)
                .shadow(16.0, 40.0, p.shadow)
                .child(body),
        )
}

/// The shell shared by every menu entry: full-width row, rounded hover fill
/// and the button role that makes the entry reachable from the keyboard.
fn menu_row(key: &'static str, label: &'static str, height: f32, hover: Color) -> El {
    row()
        .key(key)
        .w_full()
        .h(height)
        .px(8.0)
        .radius(4.0)
        .gap(8.0)
        .center_y()
        .role("button")
        .label(label)
        .hover(hover)
        .hover_group()
}

fn menu_label(label: &'static str, danger: bool, p: Palette) -> El {
    text(label)
        .font(theme::SMALL_MED)
        .color(if danger { p.danger } else { p.text })
        .grow(1.0)
        .group_color(if danger { Color::WHITE } else { p.text_bright })
}

fn menu_item(
    key: &'static str,
    label: &'static str,
    icon_id: Option<&'static Icon>,
    danger: bool,
    p: Palette,
    run: impl Fn(&mut App) + 'static,
) -> El {
    let hover = if danger { p.danger } else { p.accent };
    let row_el = menu_row(key, label, 32.0, hover)
        .on_press(run)
        .child(menu_label(label, danger, p));
    match icon_id {
        Some(icon_id) => row_el.child(
            icon(icon_id, 16.0)
                .color(if danger { p.danger } else { p.text_muted })
                .group_color(Color::WHITE),
        ),
        None => row_el,
    }
}

fn menu_copy_item(
    key: &'static str,
    label: &'static str,
    icon_id: Option<&'static Icon>,
    copy: String,
    p: Palette,
) -> El {
    let row_el = menu_row(key, label, 32.0, p.accent)
        .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
            tx.copy(copy.clone());
            s.context_menu = None;
        })
        .child(menu_label(label, false, p));
    match icon_id {
        Some(icon_id) => row_el.child(
            icon(icon_id, 16.0)
                .color(p.text_muted)
                .group_color(Color::WHITE),
        ),
        None => row_el,
    }
}

fn separator(p: Palette) -> El {
    el().w_full().h(1.0).my(4.0).fill(p.divider_soft)
}

fn channel_menu(s: &App, id: Id, p: Palette) -> El {
    let channel = s.channel(id).cloned().unwrap_or_default();
    let (kind, icon_id) = match channel.kind {
        ChannelKind::Voice => ("Voice Channel", &icons::VOICE),
        ChannelKind::Stage => ("Stage Channel", &icons::VOICE),
        ChannelKind::Announcement => ("Announcement Channel", &icons::MEGAPHONE),
        ChannelKind::Forum => ("Forum Channel", &icons::FORUM),
        ChannelKind::Thread => ("Thread", &icons::THREAD),
        ChannelKind::Category => ("Category", &icons::FOLDER),
        _ => ("Text Channel", &icons::HASH),
    };
    let manage =
        has(channel.my_permissions, perm::MANAGE_CHANNELS) || s.guild_can(perm::MANAGE_CHANNELS);
    let can_perms = s.guild_can(perm::MANAGE_ROLES);
    let muted = s.channel_notify.get(&id).copied().unwrap_or_default() == NotificationLevel::None;
    let mut menu = col()
        .w_full()
        .child(
            row()
                .w_full()
                .h(34.0)
                .px(8.0)
                .gap(8.0)
                .center_y()
                .child(icon(icon_id, 18.0).color(p.text_muted))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(channel.name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(text(kind).font(theme::TINY).color(p.text_faint)),
                ),
        )
        .child(separator(p))
        .child(menu_item(
            "ctx-channel-open",
            "Open Channel",
            None,
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.screen = Screen::Guild;
                s.switch_channel(id);
            },
        ))
        .child(menu_item(
            "ctx-channel-read",
            "Mark As Read",
            Some(&icons::CHECK),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.mark_read(id);
            },
        ))
        .child(menu_item(
            "ctx-channel-mute",
            if muted {
                "Unmute Channel"
            } else {
                "Mute Channel"
            },
            Some(if muted {
                &icons::BELL
            } else {
                &icons::BELL_OFF
            }),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.mute_channel(id);
            },
        ))
        .child(menu_item(
            "ctx-channel-notify",
            "Notification Settings",
            Some(&icons::BELL),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.open_channel_notifications(id);
            },
        ))
        .child(menu_copy_item(
            "ctx-channel-copy",
            "Copy Channel Link",
            Some(&icons::LINK),
            format!("<#{id}>"),
            p,
        ));
    if manage {
        menu = menu.child(separator(p)).child(menu_item(
            "ctx-channel-edit",
            "Edit Channel",
            Some(&icons::EDIT),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.open_channel_edit(id);
            },
        ));
    }
    if can_perms {
        menu = menu.child(menu_item(
            "ctx-channel-perms",
            "Edit Permissions",
            Some(&icons::LOCK),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.open_channel_permissions(id);
            },
        ));
    }
    if manage {
        menu = menu.child(menu_item(
            "ctx-channel-delete",
            "Delete Channel",
            Some(&icons::TRASH),
            true,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.confirm_delete_channel(id);
            },
        ));
    }
    menu
}

fn member_menu(s: &App, user: Id, p: Palette) -> El {
    let account = s.user(user).cloned().unwrap_or_default();
    let guild = s.active_guild;
    let rank = s.member_rank(guild, user);
    let my_rank = s.member_rank(guild, s.me.id);
    let can_act = user != s.me.id && my_rank > rank;
    let can_kick = can_act && s.guild_can(perm::KICK_MEMBERS);
    let can_ban = can_act && s.guild_can(perm::BAN_MEMBERS);
    let can_timeout = can_act && s.guild_can(perm::MODERATE_MEMBERS);
    let is_friend = s
        .relationships
        .iter()
        .any(|rel| rel.user.id == user && rel.kind == RelationshipKind::Friend);
    let mut menu = col()
        .w_full()
        .child(
            row()
                .w_full()
                .h(46.0)
                .px(8.0)
                .gap(8.0)
                .center_y()
                .child(avatar_status(
                    &account,
                    32.0,
                    p,
                    theme::color("#111214"),
                    false,
                ))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(account.display_name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(account.handle())
                                .font(theme::TINY)
                                .color(p.text_faint)
                                .truncate(),
                        ),
                ),
        )
        .child(separator(p))
        .child(menu_item(
            "ctx-member-profile",
            "View Profile",
            Some(&icons::USERS),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.modal = Modal::Profile(user);
            },
        ))
        .child(menu_item(
            "ctx-member-message",
            "Send Message",
            Some(&icons::CHAT),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.start_dm(user);
            },
        ))
        .child(menu_item(
            "ctx-member-mention",
            "Mention",
            Some(&icons::REPLY),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.composer.insert(&format!("<@{user}> "));
            },
        ));
    if !is_friend && user != s.me.id {
        menu = menu.child(menu_item(
            "ctx-member-friend",
            "Add Friend",
            Some(&icons::PLUS),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.friend_user(user);
            },
        ));
    }
    if user != s.me.id {
        let muted = crate::voice::user_muted(user);
        menu = menu.child(menu_item(
            "ctx-member-local-mute",
            if muted {
                "Unmute for Me"
            } else {
                "Mute for Me"
            },
            Some(&icons::BELL_OFF),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                let muted = crate::voice::user_muted(user);
                crate::voice::set_user_volume(user, crate::voice::user_volume(user), !muted);
                s.toast(if muted {
                    "Unmuted for you"
                } else {
                    "Muted for you"
                });
            },
        ));
        menu = menu.child(menu_item(
            "ctx-member-volume",
            "User Volume",
            Some(&icons::HEADPHONES),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.open_user_volume(user);
            },
        ));
    }
    menu = menu.child(menu_copy_item(
        "ctx-member-copy",
        "Copy User ID",
        Some(&icons::COPY),
        user.to_string(),
        p,
    ));
    if can_kick || can_ban || can_timeout {
        menu = menu.child(separator(p));
        if can_timeout {
            menu = menu.child(menu_item(
                "ctx-member-timeout",
                "Timeout",
                Some(&icons::CLOCK),
                false,
                p,
                move |s: &mut App| {
                    s.context_menu = None;
                    s.open_timeout(user);
                },
            ));
        }
        if can_kick {
            menu = menu.child(menu_item(
                "ctx-member-kick",
                "Kick",
                Some(&icons::CLOSE),
                true,
                p,
                move |s: &mut App| {
                    s.context_menu = None;
                    s.confirm_kick(user);
                },
            ));
        }
        if can_ban {
            menu = menu.child(menu_item(
                "ctx-member-ban",
                "Ban",
                Some(&icons::BAN),
                true,
                p,
                move |s: &mut App| {
                    s.context_menu = None;
                    s.confirm_ban(user);
                },
            ));
        }
    }
    menu
}

/// One row of the message menu: the icon leads, the label follows, the way
/// Discord's message menu reads.
fn message_menu_item(
    key: &'static str,
    label: &'static str,
    icon_id: &'static Icon,
    danger: bool,
    p: Palette,
    run: impl Fn(&mut App, &mut rune::Tx) + 'static,
) -> El {
    let hover = if danger {
        p.danger.alpha(0.12)
    } else {
        p.card_hover
    };
    menu_row(key, label, 36.0, hover)
        .on_press_tx(run)
        .child(
            icon(icon_id, 18.0)
                .color(if danger { p.danger } else { p.text_muted })
                .group_color(if danger { p.danger } else { p.text_bright }),
        )
        .child(menu_label(label, danger, p))
}

/// The emoji strip at the top of the message menu. Discord offers a row of
/// quick reactions above "Add Reaction" for the same reason: one click is
/// faster than opening a picker.
fn quick_reaction_strip(mid: Id, p: Palette) -> El {
    let mut strip = row().w_full().gap(2.0);
    for emoji in QUICK_REACTIONS {
        strip = strip.child(
            el().key(format!("message-react-{emoji}"))
                .grow(1.0)
                .h(42.0)
                .radius(4.0)
                .hover(p.card_hover)
                .center()
                .role("button")
                .label(format!("React with {emoji}"))
                .help(format!("React with {emoji}"))
                .on_press(move |s: &mut App| {
                    s.context_menu = None;
                    s.react(mid, emoji);
                })
                .child(text(emoji).font(TextStyle::new(22.0, 400.0)).color(p.text)),
        );
    }
    strip
}

fn add_reaction_row(mid: Id, p: Palette) -> El {
    menu_row(
        "message-menu-add-reaction",
        "Add Reaction",
        36.0,
        p.card_hover,
    )
    .on_press(move |s: &mut App| {
        s.context_menu = None;
        s.open_reaction(mid);
    })
    .child(
        icon(&icons::SMILE, 18.0)
            .color(p.text_muted)
            .group_color(p.text_bright),
    )
    .child(menu_label("Add Reaction", false, p))
    .child(icon(&icons::CHEVRON_RIGHT, 16.0).color(p.text_faint))
}

fn message_menu(s: &App, id: Id, p: Palette) -> El {
    let Some(msg) = s.message(id).cloned() else {
        return el();
    };
    let mine = msg.author_id == s.me.id;
    let perms = s
        .channel(msg.channel_id)
        .map(|channel| channel.my_permissions)
        .unwrap_or(0);
    let can_send = s
        .channel(msg.channel_id)
        .is_some_and(|channel| s.can_send_in(channel));
    let can_manage = has(perms, perm::MANAGE_MESSAGES);
    let can_react = has(perms, perm::ADD_REACTIONS);
    let channel = s.channel(msg.channel_id).cloned();
    let can_thread = channel.as_ref().is_some_and(|channel| {
        channel.guild_id.is_some()
            && has(channel.my_permissions, perm::CREATE_THREADS)
            && matches!(channel.kind, ChannelKind::Text | ChannelKind::Announcement)
    });
    let mid = id;
    let author_id = msg.author_id;
    let channel_id = msg.channel_id;
    let preview = msg.preview();
    let content = msg.content.clone();
    let edit_body = content.clone();
    let thread_name = plain_thread_name(&content);
    let pinned = msg.pinned;

    let mut menu = col().w_full().gap(1.0);
    if can_react {
        menu = menu
            .child(quick_reaction_strip(mid, p))
            .child(add_reaction_row(mid, p))
            .child(separator(p));
    }
    if mine {
        menu = menu.child(message_menu_item(
            "message-menu-edit",
            "Edit Message",
            &icons::EDIT,
            false,
            p,
            move |s: &mut App, _tx: &mut rune::Tx| {
                s.context_menu = None;
                s.open_edit(mid, &edit_body);
            },
        ));
    }
    if can_send {
        menu = menu
            .child(message_menu_item(
                "message-menu-reply",
                "Reply",
                &icons::REPLY,
                false,
                p,
                move |s: &mut App, _tx: &mut rune::Tx| {
                    s.context_menu = None;
                    s.set_reply(mid, author_id, &preview);
                },
            ))
            .child(message_menu_item(
                "message-menu-forward",
                "Forward",
                &icons::FORWARD,
                false,
                p,
                move |s: &mut App, _tx: &mut rune::Tx| {
                    s.context_menu = None;
                    s.open_forward(mid);
                },
            ));
    }
    if can_thread {
        menu = menu.child(message_menu_item(
            "message-menu-thread",
            "Create Thread",
            &icons::THREAD,
            false,
            p,
            move |s: &mut App, _tx: &mut rune::Tx| {
                s.context_menu = None;
                s.profile_edit = rune::TextEdit::new(thread_name.clone());
                s.thread_tag_ids.clear();
                s.thread_body.clear();
                s.modal = Modal::ThreadCreate {
                    parent: channel_id,
                    message: Some(mid),
                };
            },
        ));
    }
    if can_manage {
        menu = menu.child(separator(p)).child(message_menu_item(
            "message-menu-pin",
            if pinned {
                "Unpin Message"
            } else {
                "Pin Message"
            },
            &icons::PIN,
            false,
            p,
            move |s: &mut App, _tx: &mut rune::Tx| {
                s.context_menu = None;
                s.toggle_pin(mid);
            },
        ));
    }
    let link = format!("runecord://channel/{channel_id}/{mid}");
    menu = menu
        .child(separator(p))
        .child(message_menu_item(
            "message-menu-copy-link",
            "Copy Message Link",
            &icons::LINK,
            false,
            p,
            move |s: &mut App, tx: &mut rune::Tx| {
                tx.copy(link.clone());
                s.context_menu = None;
            },
        ))
        .child(message_menu_item(
            "message-menu-copy-text",
            "Copy Text",
            &icons::COPY,
            false,
            p,
            move |s: &mut App, tx: &mut rune::Tx| {
                tx.copy(content.clone());
                s.context_menu = None;
            },
        ));
    if mine || can_manage {
        menu = menu.child(separator(p)).child(message_menu_item(
            "message-menu-delete",
            "Delete Message",
            &icons::TRASH,
            true,
            p,
            move |s: &mut App, _tx: &mut rune::Tx| {
                s.context_menu = None;
                s.delete_message(mid);
            },
        ));
    }
    menu.child(separator(p)).child(message_menu_item(
        "message-menu-copy-id",
        "Copy Message ID",
        &icons::ID_CARD,
        false,
        p,
        move |s: &mut App, tx: &mut rune::Tx| {
            tx.copy(mid.to_string());
            s.context_menu = None;
        },
    ))
}

fn thread_menu(s: &App, id: Id, p: Palette) -> El {
    let Some(thread) = s.channel(id).cloned() else {
        return el();
    };
    let Some(info) = thread.thread.clone() else {
        return el();
    };
    let forum = s
        .channel(info.parent_id)
        .is_some_and(|parent| parent.kind == ChannelKind::Forum);
    let joined = s.thread_is_joined(id);
    let can_manage = s.can_manage_thread(&thread);
    let can_edit_tags = forum
        && (thread.owner_id == Some(s.me.id) || has(thread.my_permissions, perm::MANAGE_THREADS));
    // The server removes threads through the channel endpoint.
    let can_delete = has(thread.my_permissions, perm::MANAGE_CHANNELS);
    let archived = info.archived;
    let open_elsewhere = s.active_channel != id;
    let parent_id = info.parent_id;

    let mut menu = col().w_full().gap(1.0);
    if open_elsewhere {
        menu = menu.child(menu_item(
            "ctx-thread-open",
            if forum { "Open Post" } else { "Open Thread" },
            Some(&icons::THREAD),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.open_thread(id);
            },
        ));
    }
    menu = menu.child(menu_item(
        "ctx-thread-join",
        if joined {
            "Leave Thread"
        } else {
            "Join Thread"
        },
        Some(&icons::USERS),
        false,
        p,
        move |s: &mut App| {
            s.context_menu = None;
            s.toggle_thread_join(id);
        },
    ));
    if can_edit_tags {
        menu = menu.child(menu_item(
            "ctx-thread-tags",
            "Edit Tags",
            Some(&icons::EDIT),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.open_thread_tags(id);
            },
        ));
    }
    menu = menu.child(menu_copy_item(
        "ctx-thread-link",
        if forum {
            "Copy Post Link"
        } else {
            "Copy Thread Link"
        },
        Some(&icons::LINK),
        format!("runecord://channel/{parent_id}/{id}"),
        p,
    ));
    if can_manage {
        menu = menu.child(separator(p)).child(menu_item(
            "ctx-thread-archive",
            match (forum, archived) {
                (true, true) => "Open Post",
                (true, false) => "Close Post",
                (false, true) => "Open Thread",
                (false, false) => "Close Thread",
            },
            Some(&icons::CLOCK),
            false,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.archive_thread(id, !archived);
            },
        ));
    }
    if can_delete {
        menu = menu.child(separator(p)).child(menu_item(
            "ctx-thread-delete",
            if forum {
                "Delete Post"
            } else {
                "Delete Thread"
            },
            Some(&icons::TRASH),
            true,
            p,
            move |s: &mut App| {
                s.context_menu = None;
                s.delete_thread(id);
            },
        ));
    }
    menu
}
