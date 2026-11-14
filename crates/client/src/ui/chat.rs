//! Chat surface.

use super::*;

pub(crate) fn chat(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    let Some(channel) = s.current_channel().cloned() else {
        return col().grow(1.0).fill(p.chat);
    };
    let insets = cx.chrome_insets();
    let header = chat_header(s, &channel, p, insets.trailing);
    if channel.kind == ChannelKind::Voice {
        return voice_view(s, &channel, p, header);
    }
    if channel.kind == ChannelKind::Forum {
        return forum_view(s, &channel, p, header);
    }
    let loaded = s.messages.contains_key(&channel.id);
    let empty = s
        .messages
        .get(&channel.id)
        .is_some_and(|messages| messages.is_empty());
    let list = if !loaded {
        skeleton_message_rows(4, p, s.clock)
    } else if empty {
        empty_channel_intro(s, &channel, p)
    } else {
        message_list(s, &channel, p)
    };
    let composer = composer(s, &channel, p);
    let archived = channel
        .thread
        .as_ref()
        .is_some_and(|thread| thread.archived);
    let mut chat = col()
        .key("chat")
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .h_full()
        .fill(p.chat)
        .child(header);
    if channel.kind == ChannelKind::Thread {
        chat = chat.child(thread_context(s, &channel, p));
    }
    chat = chat.child(list);
    if archived {
        chat = chat.child(
            row()
                .key("thread-archived")
                .w_full()
                .h(44.0)
                .px(20.0)
                .center_y()
                .fill(p.sidebar)
                .child(
                    text("This thread is archived.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        );
    } else {
        chat = chat.child(composer);
    }
    chat
}

pub(crate) fn video_tile_size(count: usize, chat_open: bool) -> (f32, f32) {
    if chat_open {
        return match count {
            0 | 1 => (440.0, 247.5),
            2..=4 => (300.0, 168.75),
            _ => (230.0, 129.4),
        };
    }
    match count {
        0 | 1 => (760.0, 427.5),
        2..=4 => (470.0, 264.4),
        _ => (320.0, 180.0),
    }
}

fn voice_view(s: &App, channel: &Channel, p: Palette, header: El) -> El {
    let id = channel.id;
    let joined = s
        .voice
        .get(&s.me.id)
        .is_some_and(|v| v.channel_id == Some(id));
    let people: Vec<Id> = s
        .voice
        .values()
        .filter(|v| v.channel_id == Some(id))
        .map(|v| v.user_id)
        .collect();

    let mut stage = row()
        .key("voice-stage")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(10.0)
        .flex_wrap()
        .justify_center()
        .items_center()
        .padding(24.0);
    if people.is_empty() {
        stage = stage.child(
            col()
                .w(360.0)
                .h(203.0)
                .radius(8.0)
                .fill(p.rail)
                .center()
                .gap(8.0)
                .child(icon(&icons::VOICE, 40.0).color(p.text_faint))
                .child(
                    text("No one is here yet")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        );
    }
    let (tile_width, tile_height) = video_tile_size(people.len(), s.voice_chat);
    for user_id in &people {
        let user = s.user(*user_id).cloned().unwrap_or_default();
        let muted = s.voice.get(user_id).is_some_and(|v| v.self_mute);
        let body = if let Some(pixels) = s.voice_video.get(user_id) {
            el().w_full()
                .grow(1.0)
                .min_h(0.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fit)
        } else {
            el().w_full()
                .grow(1.0)
                .min_h(0.0)
                .center()
                .child(avatar_status(&user, 64.0, p, p.rail, true))
        };
        stage = stage.child(
            col()
                .key(format!("voice-tile-{user_id}"))
                .w(tile_width)
                .h(tile_height)
                .radius(8.0)
                .clip_rounded(8.0)
                .fill(p.rail)
                .child(body)
                .child(
                    row()
                        .w_full()
                        .h(38.0)
                        .px(10.0)
                        .gap(6.0)
                        .center_y()
                        .fill(Color::BLACK.alpha(0.25))
                        .child(
                            text(if *user_id == s.me.id {
                                format!("{} (You)", user.display_name)
                            } else {
                                user.display_name.clone()
                            })
                            .font(theme::SMALL_MED)
                            .color(p.text_bright)
                            .grow(1.0)
                            .truncate(),
                        )
                        .children(s.voice_video_info.get(user_id).map(|(_, height, fps)| {
                            let label = if *fps > 0 {
                                format!("{}p · {}fps", height, fps)
                            } else {
                                format!("{}p", height)
                            };
                            text(label).font(theme::TINY).color(p.text_muted)
                        }))
                        .child(icon(&icons::MIC, 14.0).color(if muted {
                            p.danger
                        } else {
                            p.text_muted
                        })),
                ),
        );
    }

    let control = |key: &'static str,
                   icon_id: &'static rune::Icon,
                   label: String,
                   active: bool,
                   danger: bool,
                   run: fn(&mut App)| {
        let fill = if danger {
            p.danger
        } else if active {
            p.green
        } else {
            p.rail
        };
        el().key(key)
            .size(40.0, 40.0)
            .radius(20.0)
            .center()
            .role("button")
            .label(label.clone())
            .help(label)
            .fill(fill)
            .hover(if danger {
                p.danger.alpha(0.85)
            } else if active {
                p.green.alpha(0.85)
            } else {
                p.rail_hover
            })
            .on_press(run)
            .child(icon(icon_id, 18.0).color(if active || danger {
                Color::WHITE
            } else {
                p.text_bright
            }))
    };

    let mut controls = row().w_full().h(64.0).gap(8.0).center();
    if joined {
        controls = controls
            .child(control(
                "video-quality-open",
                &icons::SETTINGS,
                format!("Video quality: {}", s.video_quality.label()),
                false,
                false,
                |s: &mut App| s.open_video_quality(),
            ))
            .child(control(
                "voice-mute",
                &icons::MIC,
                if s.voice_mute {
                    "Unmute".into()
                } else {
                    "Mute".into()
                },
                s.voice_mute,
                false,
                |s: &mut App| s.toggle_voice_mute(),
            ))
            .child(control(
                "voice-deafen",
                &icons::HEADPHONES,
                if s.voice_deaf {
                    "Undeafen".into()
                } else {
                    "Deafen".into()
                },
                s.voice_deaf,
                false,
                |s: &mut App| s.toggle_voice_deafen(),
            ))
            .child(control(
                "camera-toggle",
                &icons::VIDEO,
                if s.voice_camera {
                    "Stop Camera".into()
                } else {
                    "Camera".into()
                },
                s.voice_camera,
                false,
                |s: &mut App| s.toggle_voice_camera(),
            ))
            .child(control(
                "screen-toggle",
                &icons::MONITOR,
                if s.voice_screen {
                    "Stop Share".into()
                } else {
                    "Share Screen".into()
                },
                s.voice_screen,
                false,
                |s: &mut App| s.toggle_voice_screen(),
            ))
            .child(control(
                "voice-join",
                &icons::PHONE,
                "Leave".into(),
                false,
                true,
                |s: &mut App| s.set_voice(None),
            ));
    } else {
        controls = controls.child(
            el().key("voice-join")
                .h(38.0)
                .px(18.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Join Voice")
                .fill(p.accent)
                .hover(p.accent_hover)
                .on_press(move |s: &mut App| s.set_voice(Some(id)))
                .child(
                    text("Join Voice")
                        .font(theme::SMALL_MED)
                        .color(Color::WHITE),
                ),
        );
    }

    let stage_column = col()
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .child(stage)
        .child(controls);
    let mut body = row().w_full().grow(1.0).min_h(0.0).child(stage_column);
    if s.voice_chat {
        body = body.child(voice_chat_panel(s, channel, p));
    }
    col()
        .key("chat")
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .h_full()
        .fill(p.chat)
        .child(header)
        .child(body)
}

fn voice_chat_panel(s: &App, channel: &Channel, p: Palette) -> El {
    let empty = s
        .messages
        .get(&channel.id)
        .is_none_or(|messages| messages.is_empty());
    let body = if empty {
        col()
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .center()
            .gap(6.0)
            .child(icon(&icons::CHAT, 28.0).color(p.text_faint))
            .child(
                text("No messages yet")
                    .font(theme::SMALL_MED)
                    .color(p.text_bright),
            )
            .child(
                text("Messages sent here stay with the voice channel.")
                    .font(theme::TINY)
                    .color(p.text_muted),
            )
    } else {
        message_list(s, channel, p)
    };
    col()
        .key("voice-chat-panel")
        .w(360.0)
        .h_full()
        .min_h(0.0)
        .fill(p.chat)
        .border_sides([1.0, 0.0, 0.0, 0.0], p.divider)
        .child(
            row()
                .w_full()
                .h(42.0)
                .px(12.0)
                .gap(6.0)
                .center_y()
                .child(icon(&icons::HASH, 16.0).color(p.text_muted))
                .child(
                    text(channel.name.clone())
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                )
                .child(
                    el().key("voice-chat-close")
                        .size(26.0, 26.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Close voice chat")
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.voice_chat = false)
                        .child(icon(&icons::CLOSE, 13.0).color(p.text_muted)),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(body)
        .child(composer_with(s, channel, p, true))
}

fn thread_context(s: &App, channel: &Channel, p: Palette) -> El {
    let Some(info) = channel.thread.clone() else {
        return el();
    };
    let parent = s
        .channel(info.parent_id)
        .map(|parent| parent.name.clone())
        .unwrap_or_else(|| "channel".to_string());
    let archived = info.archived;
    let can_manage = has(channel.my_permissions, perm::MANAGE_THREADS)
        || has(channel.my_permissions, perm::MANAGE_CHANNELS)
        || channel.owner_id == Some(s.me.id);
    row()
        .key("thread-context")
        .w_full()
        .h(38.0)
        .px(20.0)
        .gap(8.0)
        .center_y()
        .fill(p.sidebar)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider)
        .child(icon(&icons::THREAD, 16.0).color(p.text_muted))
        .child(
            text(format!("Thread in #{}", parent))
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            text(format!("{} messages", info.message_count))
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .children(archived.then(|| {
            text("ARCHIVED")
                .font(theme::TINY)
                .color(p.yellow)
                .border(1.0, p.yellow)
                .radius(4.0)
                .px(5.0)
        }))
        .child(el().grow(1.0))
        .child(
            el().h(26.0)
                .px(10.0)
                .radius(4.0)
                .key("thread-close")
                .role("button")
                .label("Back to parent channel")
                .fill(p.rail)
                .hover(p.rail_hover)
                .center()
                .on_press(|s: &mut App| s.close_thread())
                .child(text("Back").font(theme::LABEL).color(p.text_bright)),
        )
        .children(can_manage.then(|| {
            el().h(26.0)
                .px(10.0)
                .radius(4.0)
                .key("thread-archive")
                .role("button")
                .label(if archived {
                    "Unarchive thread"
                } else {
                    "Archive thread"
                })
                .fill(p.rail)
                .hover(p.rail_hover)
                .center()
                .on_press(move |s: &mut App| s.archive_thread(!archived))
                .child(
                    text(if archived { "Unarchive" } else { "Archive" })
                        .font(theme::LABEL)
                        .color(if archived { p.green } else { p.text_bright }),
                )
        }))
}

pub(crate) fn chat_header(s: &App, channel: &Channel, p: Palette, trailing: f32) -> El {
    let is_dm = channel.guild_id.is_none();
    let title_row = if is_dm {
        let dm = s
            .dms
            .iter()
            .find(|d| d.channel.id == channel.id)
            .cloned()
            .unwrap_or_default();
        let title = crate::ui::dms::dm_title(&dm);
        let group = dm.channel.kind == ChannelKind::GroupDm;
        row()
            .gap(8.0)
            .center_y()
            .child(crate::ui::dms::dm_avatar(s, &dm, 24.0, p, p.header, true))
            .child(text(title).font(theme::NAME).color(p.text_bright))
            .child(
                text(if group {
                    format!("{} members", dm.recipients.len() + 1)
                } else {
                    dm.recipients
                        .first()
                        .map(|user| user.handle())
                        .unwrap_or_default()
                })
                .font(theme::SMALL)
                .color(p.text_muted),
            )
    } else {
        let manage = s.guild_can(perm::MANAGE_CHANNELS);
        let channel_icon = match channel.kind {
            ChannelKind::Forum => &icons::FORUM,
            ChannelKind::Announcement => &icons::MEGAPHONE,
            ChannelKind::Voice => &icons::VOICE,
            ChannelKind::Thread => &icons::THREAD,
            _ => &icons::HASH,
        };
        let mut title = row()
            .gap(6.0)
            .center_y()
            .child(icon(channel_icon, 24.0).color(p.text_faint))
            .child(
                text(channel.name.clone())
                    .font(theme::NAME)
                    .color(p.text_bright),
            );
        if manage {
            let id = channel.id;
            title = title
                .role("button")
                .label(format!("Edit channel #{}", channel.name))
                .help("Edit channel")
                .on_press(move |s: &mut App| s.open_channel_edit(id));
        }
        title
            .child(el().w(1.0).h(24.0).fill(p.divider).ml(6.0).mr(6.0))
            .child(
                text(channel.topic.clone())
                    .font(theme::SMALL)
                    .color(p.text_muted)
                    .grow(1.0)
                    .min_w(0.0)
                    .truncate(),
            )
    };

    let channel_id = channel.id;
    let group_dm = is_dm && channel.kind == ChannelKind::GroupDm;
    let mut tools = row().gap(2.0).center_y();
    if group_dm {
        tools = tools.child(
            tool_button("dm-settings", &icons::USERS, p)
                .on_press(move |s: &mut App| s.open_dm_settings(channel_id)),
        );
    }
    if is_dm {
        tools = tools
            .child(
                tool_button("dm-call", &icons::PHONE, p)
                    .on_press(|s: &mut App| s.toast("Voice calls need audio transport")),
            )
            .child(
                tool_button("dm-video", &icons::VIDEO, p)
                    .on_press(|s: &mut App| s.toast("Video calls need audio transport")),
            );
    } else {
        tools = tools
            .child(
                tool_button("threads", &icons::THREAD, p)
                    .on_press(|s: &mut App| s.modal = Modal::Threads),
            )
            .child(
                tool_button("notifications", &icons::BELL, p)
                    .on_press(move |s: &mut App| s.open_channel_notifications(channel_id)),
            )
            .child(tool_button("pins", &icons::PIN, p).on_press(|s: &mut App| s.open_pins()))
            .child(
                tool_button("members", &icons::USERS, p)
                    .on_press(|s: &mut App| s.show_members = !s.show_members),
            );
    }
    if channel.kind == ChannelKind::Voice {
        tools = tools.child(
            tool_button("voice-chat-toggle", &icons::CHAT, p)
                .on_press(|s: &mut App| s.voice_chat = !s.voice_chat),
        );
    }
    tools = tools
        .child(
            row()
                .key("search")
                .w(144.0)
                .h(24.0)
                .radius(4.0)
                .fill(p.rail)
                .px(6.0)
                .gap(4.0)
                .center_y()
                .role("button")
                .label("Search messages")
                .help("Search messages")
                .hover(p.rail_hover)
                .on_press(|s: &mut App| s.modal = Modal::Search)
                .child(icon(&icons::SEARCH, 14.0).color(p.text_faint))
                .child(
                    text("Search")
                        .font(TextStyle::new(13.0, 400.0))
                        .color(p.text_faint),
                ),
        )
        .child(tool_button("inbox", &icons::INBOX, p).on_press(|s: &mut App| s.open_inbox()))
        .child(
            tool_button("help", &icons::HELP, p).on_press(|s: &mut App| s.toast("Runecord 0.1")),
        );

    row()
        .key("chat-header")
        .w_full()
        .h(50.0)
        .pl(20.0)
        .pr(8.0 + trailing)
        .gap(10.0)
        .center_y()
        .fill(p.header)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
        .window_drag()
        .child(title_row)
        .child(el().grow(1.0))
        .child(tools)
}

pub(crate) fn message_list(s: &App, channel: &Channel, p: Palette) -> El {
    let messages = s
        .messages
        .get(&channel.id)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let count = messages.len();
    let channel_id = channel.id;
    let mut scroll = col()
        .key("messages-scroll")
        .grow(1.0)
        .min_h(0.0)
        .scroll(ScrollSpec { stick_bottom: true })
        .scroll_edge_fade(8.0);
    if channel.guild_id.is_some()
        && matches!(
            channel.kind,
            ChannelKind::Text | ChannelKind::Announcement | ChannelKind::Forum
        )
    {
        scroll = scroll.child(intro(s, channel, p));
    }
    scroll.child(
        col()
            .key("message-rows")
            .w_full()
            .pb(8.0)
            .growth(GrowthSpec::default())
            .virtual_rows(
                count,
                if s.compact { 52.0 } else { 64.0 },
                move |s: &App, i| {
                    s.messages
                        .get(&channel_id)
                        .and_then(|list| list.get(i))
                        .map(|m| format!("msg-{}", m.id).into())
                        .unwrap_or_else(|| format!("msg-missing-{i}").into())
                },
                move |s: &App, i| {
                    let Some(messages) = s.messages.get(&channel_id) else {
                        return el();
                    };
                    let Some(msg) = messages.get(i) else {
                        return el();
                    };
                    let grouped = i > 0
                        && messages.get(i - 1).is_some_and(|prev| {
                            prev.author_id == msg.author_id
                                && msg.timestamp - prev.timestamp < 7 * 60_000
                                && msg.reply_to.is_none()
                        });
                    let day = if grouped {
                        None
                    } else if i == 0 {
                        Some(day_divider_label(msg.timestamp))
                    } else {
                        messages.get(i - 1).and_then(|prev| {
                            (!same_day(prev.timestamp, msg.timestamp))
                                .then(|| day_divider_label(msg.timestamp))
                        })
                    };
                    message_row(s, msg, grouped, p, day)
                },
            )
            .on_visible_rows(move |s: &mut App, range| {
                s.visible_messages
                    .insert(channel_id, (range.start, range.end));
                s.request_images_for_range(channel_id, range.start, range.end);
                if range.start < 8
                    && s.messages
                        .get(&channel_id)
                        .is_some_and(|messages| messages.len() >= 8)
                {
                    s.load_older();
                }
                false
            }),
    )
}

fn empty_channel_intro(s: &App, channel: &Channel, p: Palette) -> El {
    col()
        .key("empty-intro")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .justify_end()
        .px(20.0)
        .pb(16.0)
        .child(intro(s, channel, p))
}

pub(crate) fn intro(s: &App, channel: &Channel, p: Palette) -> El {
    let channel_id = channel.id;
    let can_edit = channel.guild_id.is_some()
        && channel.kind != ChannelKind::Thread
        && s.guild_can(perm::MANAGE_CHANNELS);
    col()
        .key("intro")
        .w_full()
        .px(4.0)
        .pt(16.0)
        .pb(8.0)
        .gap(8.0)
        .child(
            el().size(68.0, 68.0)
                .radius(34.0)
                .fill(p.input)
                .center()
                .child(icon(&icons::HASH, 42.0).color(p.text_bright)),
        )
        .child(
            text(format!("Welcome to #{}!", channel.name))
                .font(theme::INTRO)
                .color(p.text_bright),
        )
        .child(
            text(format!(
                "This is the start of the #{} channel.",
                channel.name
            ))
            .font(theme::BODY)
            .color(p.text_muted),
        )
        .children(can_edit.then(|| {
            row()
                .key("intro-edit")
                .gap(5.0)
                .center_y()
                .role("button")
                .label("Edit Channel")
                .on_press(move |s: &mut App| s.open_channel_edit(channel_id))
                .child(icon(&icons::EDIT, 14.0).color(p.text_link))
                .child(
                    text("Edit Channel")
                        .font(theme::SMALL_MED)
                        .color(p.text_link),
                )
        }))
}

pub(crate) fn message_text(s: &App, content: &str, p: Palette) -> El {
    let mut parts: Vec<(String, bool)> = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("<@") {
        let Some(close) = rest[start + 2..].find('>') else {
            break;
        };
        let inner = &rest[start + 2..start + 2 + close];
        let (role, id) = match inner.strip_prefix('&') {
            Some(value) => (true, value),
            None => (false, inner),
        };
        let Ok(id) = id.parse::<runecord_protocol::Id>() else {
            break;
        };
        if start > 0 {
            parts.push((rest[..start].to_string(), false));
        }
        let label = if role {
            s.guild_roles(s.active_guild)
                .iter()
                .find(|item| item.id == id)
                .map(|item| item.name.clone())
                .unwrap_or_else(|| "role".into())
        } else {
            s.user(id)
                .map(|user| user.display_name.clone())
                .unwrap_or_else(|| "user".into())
        };
        parts.push((format!("@{label}"), true));
        rest = &rest[start + 2 + close + 1..];
    }
    if parts.is_empty() {
        return markdown(s, content, p);
    }
    if !rest.is_empty() {
        parts.push((rest.to_string(), false));
    }
    let mut row_el = row().key("message-text").w_full().flex_wrap().gap(2.0);
    for (part, mention) in parts {
        if mention {
            row_el = row_el.child(
                el().h(21.0)
                    .px(4.0)
                    .radius(3.0)
                    .fill(p.mention.alpha(0.35))
                    .center()
                    .child(text(part).font(theme::BODY).color(p.text_link)),
            );
        } else {
            row_el = row_el.child(text(part).font(theme::BODY).color(p.text));
        }
    }
    row_el
}

fn message_row(s: &App, msg: &Message, grouped: bool, p: Palette, day: Option<String>) -> El {
    let hook_user = msg.webhook.as_ref().map(|hook| User {
        id: hook.id,
        username: "webhook".into(),
        display_name: hook.name.clone(),
        avatar_color: hook.color,
        is_bot: true,
        ..User::default()
    });
    let fallback = User::default();
    let author = hook_user
        .as_ref()
        .or_else(|| s.user(msg.author_id))
        .unwrap_or(&fallback);
    let name_color = if let Some(hook) = &msg.webhook {
        theme::color(&format!("#{:06x}", hook.color))
    } else if msg.author_id == s.me.id {
        p.text_bright
    } else if s.screen == Screen::Guild {
        s.member_color(s.active_guild, msg.author_id)
            .map(|c| theme::color(&format!("#{:06x}", c)))
            .unwrap_or(p.text_bright)
    } else {
        p.text_bright
    };
    let mut content = col().key("message-content").grow(1.0).min_w(0.0);

    if let Some(reply) = &msg.reply_to {
        content = content.child(
            row()
                .key("reply-preview")
                .gap(4.0)
                .center_y()
                .pb(2.0)
                .child(icon(&icons::REPLY, 12.0).color(p.text_faint))
                .child(
                    text(reply.author_name.clone())
                        .font(TextStyle::new(13.0, 600.0))
                        .color(p.text_muted),
                )
                .child(
                    text(reply.content.clone())
                        .font(TextStyle::new(13.0, 400.0))
                        .color(p.text_faint)
                        .truncate()
                        .grow(1.0)
                        .min_w(0.0),
                ),
        );
    }

    if !grouped {
        content = content.child(
            row()
                .key("message-head")
                .gap(8.0)
                .center_y()
                .child(
                    text(author.display_name.clone())
                        .font(TextStyle::new(16.0, 600.0))
                        .color(name_color),
                )
                .children(author.is_bot.then(|| bot_tag(p)))
                .child(
                    text(time_label(msg.timestamp))
                        .font(TextStyle::new(12.0, 400.0))
                        .color(p.text_faint),
                ),
        );
    }

    if !msg.content.is_empty() {
        content = content.child(message_text(s, &msg.content, p));
    }

    for attachment in &msg.attachments {
        content = content.child(attachment_card(s, attachment, p));
    }

    if !msg.reactions.is_empty() {
        let mut reactions = row().key("reactions").gap(6.0).pt(7.0);
        for reaction in &msg.reactions {
            let emoji = reaction.emoji.clone();
            let toggle_emoji = emoji.clone();
            let replied = reaction.me;
            let message_id = msg.id;
            reactions = reactions.child(
                row()
                    .key(format!("reaction-{}", reaction.emoji))
                    .h(26.0)
                    .px(8.0)
                    .radius(9.0)
                    .gap(6.0)
                    .center_y()
                    .role("button")
                    .label(format!("{} reactions with {}", reaction.count, emoji))
                    .fill(if replied { p.mention } else { p.input })
                    .border(
                        1.0,
                        if replied {
                            p.accent
                        } else {
                            Color::TRANSPARENT
                        },
                    )
                    .hover(if replied {
                        p.mention_hover
                    } else {
                        p.input_hover
                    })
                    .on_press(move |s: &mut App| s.react(message_id, &toggle_emoji))
                    .child(if let Some(pixels) = custom_emoji_pixels(s, &emoji) {
                        el().size(16.0, 16.0)
                            .pixels_fit(pixels, rune::SurfaceFit::Fit)
                    } else {
                        text(emoji).font(TextStyle::new(16.0, 400.0)).color(p.text)
                    })
                    .child(
                        text(reaction.count.to_string())
                            .font(theme::LABEL)
                            .color(if replied { p.text_bright } else { p.text_muted }),
                    ),
            );
        }
        let picker_id = msg.id;
        reactions = reactions.child(
            el().size(26.0, 26.0)
                .radius(9.0)
                .center()
                .role("button")
                .label("Add reaction")
                .help("Add reaction")
                .fill(p.input)
                .hover(p.input_hover)
                .on_press(move |s: &mut App| s.open_reaction(picker_id))
                .child(icon(&icons::SMILE, 14.0).color(p.text_muted)),
        );
        content = content.child(reactions);
    }

    let perms = s
        .channel(msg.channel_id)
        .map(|c| c.my_permissions)
        .unwrap_or(0);
    let can_send = has(perms, perm::SEND_MESSAGES);
    let can_react = has(perms, perm::ADD_REACTIONS);
    let can_manage = has(perms, perm::MANAGE_MESSAGES);
    let row_msg_id = msg.id;
    let row_author = msg.author_id;
    let row_channel = msg.channel_id;
    let row_content = msg.content.clone();
    let row_preview = msg.preview();
    let row_label = format!("{}: {}", author.display_name, msg.preview());
    let compact = s.compact;
    let highlighted = s.pinned_target == Some(msg.id);
    let row_el = row()
        .key(format!("message-{}", msg.id))
        .w_full()
        .fill(if highlighted {
            p.mention.alpha(0.35)
        } else {
            Color::TRANSPARENT
        })
        .pt(if compact {
            if grouped { 1.0 } else { 8.0 }
        } else if grouped {
            1.0
        } else {
            20.0
        })
        .pb(2.0)
        .pr(52.0)
        .pl(if compact {
            if grouped { 56.0 } else { 14.0 }
        } else if grouped {
            76.0
        } else {
            20.0
        })
        .gap(if compact { 10.0 } else { 16.0 })
        .role("article")
        .label(row_label)
        .hover_group()
        .hover(p.chat_hover)
        .on_secondary_press(move |s: &mut App, press, tx: &mut rune::Tx| {
            if !rune::popup_menu_supported() {
                s.modal = Modal::MessageMenu(row_msg_id);
                return;
            }
            let mut menu = rune::PopupMenu::new()
                .item(rune::PopupItem::new("copy", "Copy Text"))
                .item(rune::PopupItem::new("reply", "Reply"))
                .item(rune::PopupItem::new("pin", "Pin / Unpin"));
            if s.channel(row_msg_id).is_some() {
                menu = menu.item(rune::PopupItem::new("thread", "Create Thread"));
            }
            if row_author == s.me.id {
                menu = menu.item(rune::PopupItem::new("edit", "Edit Message"));
            }
            if can_manage {
                menu = menu
                    .separator()
                    .item(rune::PopupItem::new("delete", "Delete Message"));
            }
            let copy = row_content.clone();
            let preview = row_preview.clone();
            let channel = row_channel;
            tx.popup_menu(
                menu,
                press,
                move |s: &mut App, picked, tx: &mut rune::Tx| match picked {
                    Some("copy") => tx.copy(copy.clone()),
                    Some("reply") => s.set_reply(row_msg_id, row_author, &preview),
                    Some("pin") => s.toggle_pin(row_msg_id),
                    Some("thread") => {
                        s.profile_edit = rune::TextEdit::new(plain_thread_name(&preview));
                        s.modal = Modal::ThreadCreate {
                            parent: channel,
                            message: Some(row_msg_id),
                        };
                    }
                    Some("edit") => s.open_edit(row_msg_id, &copy),
                    Some("delete") => s.modal = Modal::ConfirmDelete(row_msg_id),
                    _ => {}
                },
            );
        });

    let mid = msg.id;
    let author_id = msg.author_id;
    let preview = msg.preview();
    let toolbar = row()
        .key("msg-actions")
        .pin(None, Some(2.0), Some(16.0), None)
        .h(30.0)
        .w(96.0)
        .gap(0.0)
        .radius(4.0)
        .fill(p.chat)
        .border(1.0, p.divider)
        .shadow(2.0, 8.0, p.shadow)
        .group_opacity(0.0, 1.0)
        .children(
            can_react
                .then(|| action_button(&icons::SMILE, p, move |s: &mut App| s.open_reaction(mid))),
        )
        .children(can_send.then(|| {
            action_button(&icons::REPLY, p, move |s: &mut App| {
                s.set_reply(mid, author_id, &preview.clone())
            })
        }))
        .child(action_button(&icons::MORE, p, move |s: &mut App| {
            s.modal = Modal::MessageMenu(mid)
        }));

    let row_el = if grouped {
        row_el.child(content)
    } else {
        row_el
            .child(avatar(author, if compact { 28.0 } else { 40.0 }, p))
            .child(content)
    };
    let row_el = row_el.child(toolbar);
    if let Some(day) = day {
        col()
            .w_full()
            .child(
                row()
                    .key("day-divider")
                    .w_full()
                    .h(34.0)
                    .px(16.0)
                    .gap(8.0)
                    .center_y()
                    .child(el().grow(1.0).h(1.0).fill(p.divider))
                    .child(text(day).font(theme::TINY).color(p.text_muted))
                    .child(el().grow(1.0).h(1.0).fill(p.divider)),
            )
            .child(row_el)
    } else {
        row_el
    }
}

pub(crate) fn bot_tag(p: Palette) -> El {
    el().h(15.0)
        .px(4.0)
        .radius(3.0)
        .fill(p.accent)
        .center()
        .child(
            text("BOT")
                .font(TextStyle::new(10.0, 700.0))
                .color(Color::WHITE),
        )
}

pub(crate) fn forum_view(s: &App, channel: &Channel, p: Palette, header: El) -> El {
    let parent = channel.id;
    let mut cards = row().w_full().gap(12.0).flex_wrap();
    let mut count = 0;
    for thread in s.guild_channels(s.active_guild) {
        if thread.kind != ChannelKind::Thread {
            continue;
        }
        if thread.thread.as_ref().map(|t| t.parent_id) != Some(parent) {
            continue;
        }
        count += 1;
        let id = thread.id;
        let messages = s.messages.get(&id).map(Vec::as_slice).unwrap_or(&[]);
        let preview = messages
            .first()
            .map(|message| message.preview())
            .filter(|preview| !preview.is_empty())
            .or_else(|| {
                thread
                    .thread
                    .as_ref()
                    .map(|info| info.preview.clone())
                    .filter(|preview| !preview.is_empty())
            })
            .unwrap_or_else(|| "No messages yet.".to_string());
        let message_count = thread
            .thread
            .as_ref()
            .map(|info| info.message_count)
            .unwrap_or(0);
        let owner = thread
            .owner_id
            .and_then(|owner| s.user(owner))
            .map(|user| user.display_name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        cards = cards.child(
            col()
                .key(format!("forum-{}", thread.id))
                .w(300.0)
                .h(160.0)
                .radius(8.0)
                .fill(p.sidebar)
                .border(1.0, p.divider)
                .padding(14.0)
                .gap(8.0)
                .hover(p.card_hover)
                .role("button")
                .label(format!("Open post {}", thread.name))
                .motion(motion::SNAPPY)
                .on_press(move |s: &mut App| s.open_thread(id))
                .child(
                    row()
                        .gap(8.0)
                        .center_y()
                        .child(icon(&icons::THREAD, 18.0).color(p.text_faint))
                        .child(
                            text(thread.name.clone())
                                .font(theme::BODY_BOLD)
                                .color(p.text_bright)
                                .grow(1.0)
                                .min_w(0.0)
                                .max_lines(2),
                        ),
                )
                .child(
                    text(preview)
                        .font(theme::SMALL)
                        .color(p.text_muted)
                        .max_lines(2)
                        .grow(1.0)
                        .min_w(0.0),
                )
                .child(
                    row()
                        .w_full()
                        .gap(6.0)
                        .center_y()
                        .child(
                            text(owner)
                                .font(theme::TINY)
                                .color(p.text_faint)
                                .grow(1.0)
                                .truncate(),
                        )
                        .child(icon(&icons::CHAT, 12.0).color(p.text_faint))
                        .child(
                            text(message_count.to_string())
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                ),
        );
    }

    let mut body = col().w_full().grow(1.0).min_h(0.0).gap(12.0);
    if count == 0 {
        body = body.child(
            col()
                .w_full()
                .py(48.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::FORUM, 48.0).color(p.text_faint))
                .child(text("No posts yet").font(theme::BODY).color(p.text_muted))
                .child(
                    text("Start the first conversation.")
                        .font(theme::SMALL)
                        .color(p.text_faint),
                ),
        );
    } else {
        body = body.child(cards);
    }

    col()
        .key("forum")
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .fill(p.chat)
        .child(header)
        .child(
            row()
                .w_full()
                .h(52.0)
                .px(24.0)
                .center_y()
                .gap(12.0)
                .child(
                    text(format!("POSTS — {count}"))
                        .font(theme::SECTION)
                        .color(p.text_muted)
                        .grow(1.0),
                )
                .child(
                    el().key("forum-new-post")
                        .h(32.0)
                        .px(12.0)
                        .radius(4.0)
                        .gap(6.0)
                        .center()
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .role("button")
                        .label("Create a post")
                        .on_press(move |s: &mut App| {
                            s.profile_edit.clear();
                            s.thread_body.clear();
                            s.modal = Modal::ThreadCreate {
                                parent,
                                message: None,
                            };
                        })
                        .child(icon(&icons::PLUS, 16.0).color(Color::WHITE))
                        .child(text("New Post").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
        .child(
            col()
                .grow(1.0)
                .min_h(0.0)
                .px(24.0)
                .pb(24.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(body),
        )
}

#[cfg(test)]
mod tests {
    use super::video_tile_size;

    #[test]
    fn video_tiles_shrink_as_the_call_grows() {
        let (one_w, one_h) = video_tile_size(1, false);
        let (four_w, four_h) = video_tile_size(4, false);
        let (many_w, many_h) = video_tile_size(7, false);
        assert_eq!((one_w, one_h), (760.0, 427.5));
        assert!(four_w < one_w && four_h < one_h);
        assert!(many_w < four_w && many_h < four_h);
        assert!((one_w / one_h - 16.0 / 9.0).abs() < 0.01);
        assert!((many_w / many_h - 16.0 / 9.0).abs() < 0.01);
        let (split_w, split_h) = video_tile_size(1, true);
        assert!(split_w < one_w);
        assert!((split_w / split_h - 16.0 / 9.0).abs() < 0.01);
    }
}
