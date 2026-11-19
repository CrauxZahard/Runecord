//! Primary navigation, docked as a bar across the top of the window.

use super::*;

/// One slot in the dock: the control, with a rule beneath it that fills in as
/// the slot's state sharpens. The rule is a slice of the sunset, so the dock
/// glows amber into violet along its foot.
fn dock_slot(button: El, active: bool, unread: bool, p: Palette) -> El {
    col()
        .gap(3.0)
        .items_center()
        .justify_end()
        .child(button)
        .child(
            el().w(if active {
                22.0
            } else if unread {
                8.0
            } else {
                0.0
            })
            .h(3.0)
            .radius(2.0)
            .fill(if active {
                theme::sunset_magenta(p.dark)
            } else {
                theme::sunset_amber(p.dark).alpha(0.55)
            })
            .gradient(theme::sunset_amber(p.dark), theme::sunset_violet(p.dark)),
        )
}

pub(crate) fn rail(s: &App, p: Palette) -> El {
    let mut dock = row()
        .key("rail")
        .w_full()
        .h(58.0)
        .px(12.0)
        .gap(6.0)
        .center_y()
        .fill(p.rail)
        .border_sides([0.0, 0.0, 1.0, 0.0], theme::gloss_edge(p));

    let home_selected = s.screen == Screen::Dms;
    let home = el()
        .key("home")
        .size(42.0, 42.0)
        .radius(14.0)
        .center()
        .motion(motion::SNAPPY)
        .hover(p.rail_hover)
        .role("button")
        .label("Direct Messages")
        .help("Direct Messages")
        .fill(if home_selected {
            theme::sunset_magenta(p.dark)
        } else {
            p.rail_hover
        })
        .gradient(
            if home_selected {
                theme::sunset_magenta(p.dark)
            } else {
                p.rail_hover
            },
            if home_selected {
                theme::sunset_violet(p.dark)
            } else {
                p.rail_hover
            },
        )
        .on_press(|s: &mut App| {
            s.screen = Screen::Dms;
            s.dm_home = true;
            s.add_friend_open = false;
        })
        .child(icon(&icons::RUNECORD, 22.0).color(if home_selected {
            Color::WHITE
        } else {
            p.text
        }));
    dock = dock.child(dock_slot(home, home_selected, false, p));

    dock = dock.child(
        el().key("rail-sep")
            .w(1.0)
            .h(26.0)
            .radius(1.0)
            .fill(p.divider),
    );

    // Servers, the add button and Explore share one growing lane so the
    // conversation slots keep the right-hand end of the dock.
    let mut lane = row().gap(6.0).center_y().min_w(0.0).grow(1.0);
    for guild in &s.guilds {
        let active = s.screen == Screen::Guild && s.active_guild == guild.id;
        let unread = guild.unread_count > 0 || guild.mention_count > 0;
        let id = guild.id;
        let mut tile = el()
            .key(format!("guild-{id}"))
            .size(42.0, 42.0)
            .radius(14.0)
            .center()
            .motion(motion::SNAPPY)
            .hover(p.accent.alpha(0.18))
            .role("button")
            .label(guild.name.clone())
            .help(guild.name.clone())
            .on_press(move |s: &mut App| {
                s.switch_guild(id);
                s.dm_home = true;
                let channel = s.active_channel;
                if channel != 0 {
                    s.mark_read(channel);
                }
            });
        if active {
            tile = tile.border(2.0, theme::sunset_amber(p.dark));
        }
        lane = lane.child(dock_slot(
            tile.child(guild_icon(s, guild, 34.0, 12.0, p)),
            active,
            unread,
            p,
        ));
    }

    lane = lane.child(dock_slot(
        el().key("add-server")
            .size(42.0, 42.0)
            .radius(14.0)
            .fill(p.rail_hover)
            .border(1.0, theme::gloss_edge(p))
            .center()
            .hover(p.accent.alpha(0.28))
            .role("button")
            .label("Add a Server")
            .help("Add a Server")
            .on_press(|s: &mut App| s.modal = Modal::CreateServer)
            .child(icon(&icons::PLUS, 20.0).color(p.accent)),
        false,
        false,
        p,
    ));
    lane = lane.child(dock_slot(
        el().key("explore")
            .size(42.0, 42.0)
            .radius(14.0)
            .fill(p.rail_hover)
            .border(1.0, theme::gloss_edge(p))
            .center()
            .hover(p.accent.alpha(0.28))
            .role("button")
            .label("Explore Public Servers")
            .help("Explore Public Servers")
            .on_press(|s: &mut App| s.open_explore())
            .child(icon(&icons::COMPASS, 22.0).color(p.accent)),
        false,
        false,
        p,
    ));
    dock = dock.child(lane);

    // Direct messages ride the same dock, on the far side of the servers, so
    // they are reachable without leaving the guild you are reading.
    if !s.dms.is_empty() {
        dock = dock.child(
            el().key("rail-dm-sep")
                .w(1.0)
                .h(26.0)
                .radius(1.0)
                .fill(p.divider),
        );
        for dm in s.dms.iter().take(6) {
            let id = dm.channel.id;
            let active = s.screen == Screen::Dms && !s.dm_home && s.active_dm == id;
            let unread = s.unread.get(&id).copied().unwrap_or(0) > 0;
            let title = dm_title(dm);
            let button = el()
                .key(format!("rail-dm-{id}"))
                .size(38.0, 38.0)
                .radius(13.0)
                .center()
                .motion(motion::SNAPPY)
                .hover(p.accent.alpha(0.18))
                .role("button")
                .label(if unread {
                    format!("{title}, unread")
                } else {
                    title.clone()
                })
                .help(title)
                .on_press(move |s: &mut App| {
                    s.add_friend_open = false;
                    s.open_dm(id);
                })
                .child(dm_avatar(s, dm, 30.0, p, p.rail, false));
            dock = dock.child(dock_slot(button, active, unread, p));
        }
        if s.dms.len() > 6 {
            let more = s.dms.len() - 6;
            dock = dock.child(
                el().key("rail-dm-more")
                    .h(22.0)
                    .px(8.0)
                    .radius(11.0)
                    .center()
                    .role("button")
                    .label("More direct messages")
                    .help("More direct messages")
                    .on_press(|s: &mut App| {
                        s.screen = Screen::Dms;
                        s.dm_home = true;
                    })
                    .child(
                        text(format!("+{more}"))
                            .font(theme::TINY)
                            .color(p.text_link),
                    ),
            );
        }
    } else {
        // Keep the slot visible when there are none: the dock should say where
        // conversations appear, not silently end at Explore.
        dock = dock.child(
            el().key("rail-dm-sep")
                .w(1.0)
                .h(26.0)
                .radius(1.0)
                .fill(p.divider),
        );
        dock = dock.child(dock_slot(
            el().key("rail-dm-empty")
                .size(38.0, 38.0)
                .radius(13.0)
                .center()
                .hover(p.accent.alpha(0.18))
                .role("button")
                .label("Direct messages, none yet")
                .help("Direct messages: none yet")
                .on_press(|s: &mut App| {
                    s.screen = Screen::Dms;
                    s.dm_home = true;
                })
                .child(icon(&icons::CHAT, 20.0).color(p.text_faint)),
            false,
            false,
            p,
        ));
    }
    dock.child(el().w(0.0).h(0.0))
}
