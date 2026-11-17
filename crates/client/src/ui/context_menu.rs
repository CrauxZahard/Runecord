//! In-app context menus for channels and members.
//!
//! Rune's native pop-up menu follows the OS theme, so the app draws its own dark menu.
//! These elements reproduce that menu inside the app, so the styling matches the
//! rest of the client on every platform.

use super::*;
use crate::app::{ContextMenu, ContextMenuKind};

const MENU_WIDTH: f32 = 232.0;

pub(crate) fn context_menu_layer(s: &App, cx: &Cx<App>, menu: &ContextMenu, p: Palette) -> El {
    let (window_w, window_h) = cx
        .layout("app")
        .map(|rect| (rect[2], rect[3]))
        .unwrap_or((1360.0, 860.0));
    let estimated_h = match menu.kind {
        ContextMenuKind::Channel(_) => 300.0,
        ContextMenuKind::Member(_) => 390.0,
    };
    let x = menu.x.clamp(8.0, (window_w - MENU_WIDTH - 8.0).max(8.0));
    let y = menu.y.clamp(8.0, (window_h - estimated_h - 8.0).max(8.0));
    let body = match menu.kind {
        ContextMenuKind::Channel(id) => channel_menu(s, id, p),
        ContextMenuKind::Member(id) => member_menu(s, id, p),
    };
    el().key("context-menu-layer")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .fill(Color::TRANSPARENT)
        .on_press(|s: &mut App| s.context_menu = None)
        .child(
            col()
                .key("context-menu")
                .pin(Some(x), Some(y), None, None)
                .w(MENU_WIDTH)
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

fn menu_item(
    key: &'static str,
    label: &'static str,
    icon_id: Option<&'static Icon>,
    danger: bool,
    p: Palette,
    run: impl Fn(&mut App) + 'static,
) -> El {
    let hover = if danger { p.danger } else { p.accent };
    let mut row_el = row()
        .key(key)
        .w_full()
        .h(32.0)
        .px(8.0)
        .radius(3.0)
        .gap(8.0)
        .center_y()
        .role("button")
        .label(label)
        .hover(hover)
        .hover_group()
        .on_press(run)
        .child(
            text(label)
                .font(theme::SMALL_MED)
                .color(if danger { p.danger } else { p.text })
                .grow(1.0)
                .group_color(if danger { Color::WHITE } else { p.text_bright }),
        );
    if let Some(icon_id) = icon_id {
        row_el = row_el.child(
            icon(icon_id, 16.0)
                .color(if danger { p.danger } else { p.text_muted })
                .group_color(Color::WHITE),
        );
    }
    row_el
}

fn menu_copy_item(
    key: &'static str,
    label: &'static str,
    icon_id: Option<&'static Icon>,
    copy: String,
    p: Palette,
) -> El {
    let mut row_el = row()
        .key(key)
        .w_full()
        .h(32.0)
        .px(8.0)
        .radius(3.0)
        .gap(8.0)
        .center_y()
        .role("button")
        .label(label)
        .hover(p.accent)
        .hover_group()
        .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
            tx.copy(copy.clone());
            s.context_menu = None;
        })
        .child(
            text(label)
                .font(theme::SMALL_MED)
                .color(p.text)
                .grow(1.0)
                .group_color(p.text_bright),
        );
    if let Some(icon_id) = icon_id {
        row_el = row_el.child(
            icon(icon_id, 16.0)
                .color(p.text_muted)
                .group_color(Color::WHITE),
        );
    }
    row_el
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
