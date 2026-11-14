//! Server settings: role CRUD/reordering and member role assignment.

use super::prelude::*;
use super::*;
use runecord_protocol::{AuditEntry, Role, RoleLink};

mod audit;
mod bans;
mod emojis;
mod members;
mod overview;
mod roles;
pub(crate) use audit::*;
pub(crate) use bans::*;
pub(crate) use emojis::*;
pub(crate) use members::*;
pub(crate) use overview::*;
pub(crate) use roles::*;

pub(crate) fn server_settings_page(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    let trailing = cx.chrome_insets().trailing;
    let content = match s.server_tab {
        crate::app::ServerTab::Overview => overview_view(s, p),
        crate::app::ServerTab::Bans => bans_view(s, p),
        crate::app::ServerTab::Roles => roles_view(s, p),
        crate::app::ServerTab::Members => members_view(s, p),
        crate::app::ServerTab::Emojis => emojis_view(s, p),
        crate::app::ServerTab::Audit => audit_view(s, p),
    };
    let title = match s.server_tab {
        crate::app::ServerTab::Overview => "Server Overview",
        crate::app::ServerTab::Bans => "Bans",
        crate::app::ServerTab::Roles => "Roles",
        crate::app::ServerTab::Members => "Members",
        crate::app::ServerTab::Emojis => "Emojis",
        crate::app::ServerTab::Audit => "Audit Log",
    };

    let nav = server_settings_nav(s, p);
    col()
        .key("server-settings-page")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(
            row().w_full().h_full().child(nav).child(
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .h_full()
                    .child(
                        row()
                            .key("server-settings-top")
                            .w_full()
                            .h(64.0)
                            .pl(32.0)
                            .pr(12.0 + trailing)
                            .gap(8.0)
                            .items_center()
                            .window_drag()
                            .child(
                                text(title)
                                    .font(theme::TITLE)
                                    .color(p.text_bright)
                                    .grow(1.0),
                            )
                            .children(
                                (s.server_tab == crate::app::ServerTab::Members
                                    && s.guild_can(perm::KICK_MEMBERS))
                                .then(|| {
                                    el().key("prune-open")
                                        .h(30.0)
                                        .px(12.0)
                                        .radius(4.0)
                                        .center()
                                        .role("button")
                                        .label("Prune Members")
                                        .fill(p.rail)
                                        .hover(p.rail_hover)
                                        .on_press(|s: &mut App| s.open_prune())
                                        .child(
                                            text("Prune Members")
                                                .font(theme::SMALL_MED)
                                                .color(p.text_bright),
                                        )
                                }),
                            )
                            .children((s.server_tab == crate::app::ServerTab::Members).then(|| {
                                el().key("bans-open")
                                    .h(30.0)
                                    .px(12.0)
                                    .radius(4.0)
                                    .center()
                                    .role("button")
                                    .label("Banned Users")
                                    .fill(p.rail)
                                    .hover(p.rail_hover)
                                    .on_press(|s: &mut App| s.open_bans())
                                    .child(
                                        text("Banned Users")
                                            .font(theme::SMALL_MED)
                                            .color(p.text_bright),
                                    )
                            }))
                            .child(close_button(p)),
                    )
                    .child(
                        col()
                            .w_full()
                            .grow(1.0)
                            .min_h(0.0)
                            .px(32.0)
                            .pb(32.0)
                            .scroll(ScrollSpec {
                                stick_bottom: false,
                            })
                            .child(content),
                    )
                    .children(
                        (s.server_tab == crate::app::ServerTab::Overview)
                            .then(|| overview_footer(s, p)),
                    ),
            ),
        )
}

fn nav_group(label: &'static str, p: Palette) -> El {
    text(label)
        .font(theme::TINY)
        .color(p.text_faint)
        .pt(10.0)
        .pb(2.0)
        .pl(8.0)
}

fn server_settings_nav(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let nav_item = |label: &'static str, tab: crate::app::ServerTab, key: &'static str| {
        let selected = s.server_tab == tab;
        el().key(key)
            .w_full()
            .h(32.0)
            .px(8.0)
            .radius(4.0)
            .center_y()
            .role("tab")
            .label(label)
            .fill(if selected {
                p.selected
            } else {
                Color::TRANSPARENT
            })
            .hover(p.rail)
            .motion(motion::SNAPPY)
            .on_press(move |s: &mut App| s.select_server_tab(tab))
            .child(text(label).font(theme::SMALL_MED).color(if selected {
                p.text_bright
            } else {
                p.text_muted
            }))
    };

    col()
        .key("server-settings-nav")
        .w(232.0)
        .h_full()
        .fill(p.modal_sidebar)
        .px(8.0)
        .pt(16.0)
        .gap(2.0)
        .child(
            row()
                .w_full()
                .h(48.0)
                .px(8.0)
                .gap(10.0)
                .center_y()
                .child(guild_icon(s, &guild, 32.0, 10.0, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(guild.name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text("Server Settings")
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .truncate(),
                        ),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider).my(8.0))
        .child(nav_group("GENERAL", p))
        .child(nav_item(
            "Overview",
            crate::app::ServerTab::Overview,
            "role-tab-overview",
        ))
        .child(nav_item(
            "Roles",
            crate::app::ServerTab::Roles,
            "role-tab-roles",
        ))
        .child(nav_item(
            "Emojis",
            crate::app::ServerTab::Emojis,
            "role-tab-emojis",
        ))
        .child(nav_group("MODERATION", p))
        .child(nav_item(
            "Members",
            crate::app::ServerTab::Members,
            "server-nav-members",
        ))
        .child(nav_item(
            "Bans",
            crate::app::ServerTab::Bans,
            "role-tab-bans",
        ))
        .child(nav_item(
            "Audit Log",
            crate::app::ServerTab::Audit,
            "role-tab-audit",
        ))
        .child(el().grow(1.0))
        .child(el().h(8.0))
}

fn toggle(key: &'static str, label: &'static str, on: bool, p: Palette, run: fn(&mut App)) -> El {
    row()
        .key(key)
        .h(30.0)
        .px(8.0)
        .radius(4.0)
        .gap(8.0)
        .center_y()
        .role("checkbox")
        .label(label)
        .checked(on)
        .hover(p.card_hover)
        .on_press(run)
        .child(
            el().size(14.0, 14.0)
                .radius(3.0)
                .fill(if on { p.green } else { p.input })
                .border(1.0, p.divider),
        )
        .child(text(label).font(theme::SMALL).color(p.text))
}
