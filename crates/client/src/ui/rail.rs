//! Server rail surface.

use super::*;

pub(crate) fn rail(s: &App, p: Palette) -> El {
    let mut rail = col()
        .key("rail")
        .w(72.0)
        .h_full()
        .fill(p.rail)
        .items_center()
        .pt(8.0)
        .gap(8.0);

    let home_selected = s.screen == Screen::Dms;
    let home = rail_icon(
        "home",
        &icons::RUNECORD,
        28.0,
        home_selected,
        p.accent,
        p.rail_hover,
        "Direct Messages",
    )
    .on_press(|s: &mut App| {
        s.screen = Screen::Dms;
        s.dm_home = true;
        s.add_friend_open = false;
    });
    rail = rail.child(with_indicator(home, home_selected, false, p));

    rail = rail.child(
        el().key("rail-sep")
            .w(32.0)
            .h(2.0)
            .radius(1.0)
            .fill(p.divider),
    );

    for guild in &s.guilds {
        let active = s.screen == Screen::Guild && s.active_guild == guild.id;
        let unread = guild.unread_count > 0 || guild.mention_count > 0;
        let id = guild.id;
        let button = el()
            .key(format!("guild-{id}"))
            .size(48.0, 48.0)
            .radius(24.0)
            .center()
            .motion(motion::SNAPPY)
            .hover(Color::WHITE.alpha(0.08))
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
            })
            .child(guild_icon(s, guild, 48.0, 24.0, p));
        rail = rail.child(with_indicator(button, active, unread, p));
    }

    rail = rail.child(
        el().key("add-server")
            .size(48.0, 48.0)
            .radius(24.0)
            .fill(p.rail_hover)
            .center()
            .hover(p.green)
            .role("button")
            .label("Add a Server")
            .help("Add a Server")
            .on_press(|s: &mut App| s.modal = Modal::CreateServer)
            .child(icon(&icons::PLUS, 24.0).color(p.green)),
    );
    rail = rail.child(
        el().key("explore")
            .size(48.0, 48.0)
            .radius(24.0)
            .fill(p.rail_hover)
            .center()
            .hover(p.green)
            .role("button")
            .label("Explore Public Servers")
            .help("Explore Public Servers")
            .on_press(|s: &mut App| s.open_explore())
            .child(icon(&icons::COMPASS, 26.0).color(p.green)),
    );
    rail = rail.child(el().grow(1.0));
    rail.child(el().w(0.0).h(0.0))
}
