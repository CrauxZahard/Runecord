//! Chat surface.

use super::*;

pub(crate) fn chat(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    let Some(channel) = s.current_channel().cloned() else {
        return col().grow(1.0).fill(p.chat);
    };
    let insets = cx.chrome_insets();
    let is_thread = channel.kind == ChannelKind::Thread;
    let header = if is_thread {
        thread_header(s, &channel, p, insets.trailing)
    } else {
        chat_header(s, &channel, p, insets.trailing)
    };
    if channel.kind.is_voice_like() {
        return voice_view(s, cx, &channel, p, header);
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
        message_list(s, cx, &channel, p)
    };
    let composer = composer(s, &channel, p);
    let archived = channel
        .thread
        .as_ref()
        .is_some_and(|thread| thread.archived);
    let can_reopen = is_thread && archived && s.can_manage_thread(&channel);
    let mut chat = col()
        .key("chat")
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .h_full()
        .fill(p.chat)
        .child(header);
    if channel.guild_id.is_none()
        && s.my_voice_channel() == Some(channel.id)
        && let Some(strip) = dm_call_strip(s, &channel, p)
    {
        chat = chat.child(strip);
    }
    if let Some(banner) = thread_banner(s, &channel, p) {
        chat = chat.child(banner);
    }
    chat = chat.child(list);
    if archived && !can_reopen {
        chat = chat.child(archived_composer_note(p));
    } else {
        chat = chat.child(composer);
    }
    chat
}

/// The bar that replaces the composer in an archived thread the user cannot
/// reopen. Discord reopens a closed thread when you type; ours only allows the
/// creator and moderators to do that, so the bar explains the difference.
fn archived_composer_note(p: Palette) -> El {
    col().key("composer-wrap").w_full().px(16.0).pb(16.0).child(
        row().h(44.0).radius(8.0).fill(p.input).center().child(
            text("This thread is archived. Only the creator or a moderator can reopen it.")
                .font(theme::SMALL)
                .color(p.text_muted),
        ),
    )
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

fn voice_view(s: &App, cx: &Cx<App>, channel: &Channel, p: Palette, header: El) -> El {
    let id = channel.id;
    let joined = s
        .voice
        .get(&s.me.id)
        .is_some_and(|v| v.channel_id == Some(id));
    let people: Vec<Id> = if channel.kind == ChannelKind::Stage {
        s.stages
            .get(&id)
            .map(|state| {
                state
                    .speakers
                    .iter()
                    .filter(|speaker| speaker.speaking)
                    .map(|speaker| speaker.user_id)
                    .collect()
            })
            .unwrap_or_default()
    } else {
        s.voice
            .values()
            .filter(|v| v.channel_id == Some(id))
            .map(|v| v.user_id)
            .collect()
    };

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
                .border(
                    if crate::voice::user_speaking(*user_id) {
                        2.0
                    } else {
                        0.0
                    },
                    p.green,
                )
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
            .children(s.voice_screen.then(|| {
                col()
                    .gap(2.0)
                    .center()
                    .child(control(
                        "screen-audio-toggle",
                        &icons::HEADPHONES,
                        if crate::voice::screen_audio() {
                            "Stop Audio".into()
                        } else {
                            "Share Audio".into()
                        },
                        crate::voice::screen_audio(),
                        false,
                        |s: &mut App| {
                            let next = !crate::voice::screen_audio();
                            crate::voice::set_screen_audio(next);
                            if next {
                                crate::voice::start_screen_audio();
                            } else {
                                crate::voice::stop_screen_audio();
                            }
                            s.toast(if next {
                                "Sharing screen audio"
                            } else {
                                "Stopped sharing screen audio"
                            });
                        },
                    ))
                    .child(
                        text(crate::voice::screen_audio_status())
                            .font(theme::TINY)
                            .color(p.text_faint),
                    )
            }))
            .child(control(
                "voice-join",
                &icons::PHONE,
                "Leave".into(),
                false,
                true,
                |s: &mut App| s.set_voice(None),
            ))
            .child(
                el().key("voice-activities")
                    .size(40.0, 40.0)
                    .radius(20.0)
                    .center()
                    .role("button")
                    .label("Activities")
                    .help("Activities")
                    .fill(if s.activity_sessions.contains_key(&id) {
                        p.green
                    } else {
                        p.rail
                    })
                    .hover(p.rail_hover)
                    .on_press(move |s: &mut App| s.open_activities(id))
                    .child(icon(&icons::SPARKLES, 18.0).color(p.text_bright)),
            );
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
                .on_press(move |s: &mut App| {
                    if s.channel(id)
                        .is_some_and(|ch| ch.kind == ChannelKind::Stage)
                    {
                        s.join_stage(id);
                    } else {
                        s.set_voice(Some(id));
                    }
                })
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
        body = body.child(voice_chat_panel(s, cx, channel, p));
    }
    let mut view = col()
        .key("chat")
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .h_full()
        .fill(p.chat)
        .child(header);
    if channel.kind == ChannelKind::Stage {
        view = view.child(stage_speaker_bar(s, channel, p));
    }
    if let Some(card) = activity_card(s, channel.id, p) {
        view = view.child(card);
    }
    view.child(body)
}

/// Speakers, raised hands and the audience list for a stage channel.
fn stage_speaker_bar(s: &App, channel: &Channel, p: Palette) -> El {
    let id = channel.id;
    let joined = s
        .voice
        .get(&s.me.id)
        .is_some_and(|state| state.channel_id == Some(id));
    let state = s.stages.get(&id).cloned().unwrap_or_default();
    let speakers: Vec<Id> = state
        .speakers
        .iter()
        .filter(|speaker| speaker.speaking)
        .map(|speaker| speaker.user_id)
        .collect();
    let requested: Vec<Id> = state
        .speakers
        .iter()
        .filter(|speaker| speaker.requested && !speaker.speaking)
        .map(|speaker| speaker.user_id)
        .collect();
    let mine = state
        .speakers
        .iter()
        .find(|speaker| speaker.user_id == s.me.id)
        .cloned();
    let can_manage = s.guild_can(perm::MUTE_MEMBERS) || s.guild_can(perm::MANAGE_CHANNELS);

    let mut speakers_row = row().gap(8.0).flex_wrap().items_center();
    if speakers.is_empty() {
        speakers_row = speakers_row.child(
            text("No speakers yet")
                .font(theme::TINY)
                .color(p.text_faint),
        );
    }
    for user in &speakers {
        let account = s.user(*user).cloned().unwrap_or_default();
        let mut chip = row()
            .key(format!("stage-speaker-{user}"))
            .h(34.0)
            .px(8.0)
            .radius(17.0)
            .gap(6.0)
            .center_y()
            .fill(p.card);
        chip = chip.child(avatar(&account, 22.0, p)).child(
            text(account.display_name.clone())
                .font(theme::TINY)
                .color(p.text_bright),
        );
        if can_manage && *user != s.me.id {
            let target = *user;
            chip = chip.child(
                el().key(format!("stage-move-audience-{user}"))
                    .h(20.0)
                    .px(6.0)
                    .radius(10.0)
                    .center()
                    .role("button")
                    .label(format!("Move {} to audience", account.display_name))
                    .fill(p.input)
                    .hover(p.input_hover)
                    .on_press(move |s: &mut App| s.stage_action(id, "audience", Some(target)))
                    .child(text("Audience").font(theme::TINY).color(p.text_muted)),
            );
        }
        speakers_row = speakers_row.child(chip);
    }

    let mut actions = row().gap(6.0).center_y();
    if !joined {
        actions = actions.child(
            el().key("stage-join-audience")
                .h(30.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Join the audience")
                .fill(p.accent)
                .hover(p.accent_hover)
                .on_press(move |s: &mut App| s.join_stage(id))
                .child(text("Join Audience").font(theme::TINY).color(Color::WHITE)),
        );
    } else if mine.as_ref().is_some_and(|speaker| speaker.speaking) {
        actions = actions.child(
            el().key("stage-leave-speakers")
                .h(30.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Move yourself to the audience")
                .fill(p.input)
                .hover(p.input_hover)
                .on_press(move |s: &mut App| s.stage_action(id, "audience", None))
                .child(
                    text("Move to Audience")
                        .font(theme::TINY)
                        .color(p.text_bright),
                ),
        );
    } else if mine.as_ref().is_some_and(|speaker| speaker.requested) {
        actions = actions.child(
            el().key("stage-cancel-request")
                .h(30.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Cancel your request to speak")
                .fill(p.input)
                .hover(p.input_hover)
                .on_press(move |s: &mut App| s.stage_action(id, "cancel", None))
                .child(
                    text("Requested - Cancel")
                        .font(theme::TINY)
                        .color(p.text_muted),
                ),
        );
    } else {
        actions = actions.child(
            el().key("stage-request-speak")
                .h(30.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Raise your hand to speak")
                .fill(p.input)
                .hover(p.input_hover)
                .on_press(move |s: &mut App| s.stage_action(id, "request", None))
                .child(text("Raise Hand").font(theme::TINY).color(p.text_bright)),
        );
    }

    let mut hands = row().gap(6.0).flex_wrap();
    for user in &requested {
        let account = s.user(*user).cloned().unwrap_or_default();
        let target = *user;
        let mut chip = row()
            .key(format!("stage-hand-{user}"))
            .h(28.0)
            .px(8.0)
            .radius(14.0)
            .gap(6.0)
            .center_y()
            .fill(p.yellow.alpha(0.2));
        chip = chip.child(icon(&icons::MIC, 12.0).color(p.yellow)).child(
            text(account.display_name.clone())
                .font(theme::TINY)
                .color(p.text_bright),
        );
        if can_manage {
            chip = chip.child(
                el().key(format!("stage-invite-speaker-{user}"))
                    .h(20.0)
                    .px(6.0)
                    .radius(10.0)
                    .center()
                    .role("button")
                    .label(format!("Invite {} to speak", account.display_name))
                    .fill(p.green)
                    .on_press(move |s: &mut App| s.stage_action(id, "speak", Some(target)))
                    .child(text("Invite").font(theme::TINY).color(Color::WHITE)),
            );
        }
        hands = hands.child(chip);
    }

    let audience = s
        .voice
        .values()
        .filter(|state| state.channel_id == Some(id))
        .filter(|state| !speakers.contains(&state.user_id))
        .count();

    col()
        .key("stage-bar")
        .w_full()
        .px(20.0)
        .py(8.0)
        .gap(6.0)
        .fill(p.sidebar)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
        .child(
            row()
                .w_full()
                .center_y()
                .gap(8.0)
                .child(
                    text(if state.topic.is_empty() {
                        "Stage".to_string()
                    } else {
                        state.topic.clone()
                    })
                    .font(theme::SMALL_MED)
                    .color(p.text_bright)
                    .grow(1.0),
                )
                .child(
                    text(format!(
                        "{} speakers - {audience} in the audience",
                        speakers.len()
                    ))
                    .font(theme::TINY)
                    .color(p.text_faint),
                )
                .child(actions),
        )
        .child(speakers_row)
        .children((!requested.is_empty()).then(|| {
            row()
                .w_full()
                .gap(6.0)
                .center_y()
                .child(text("HANDS").font(theme::SECTION).color(p.text_muted))
                .child(hands)
        }))
}

fn voice_chat_panel(s: &App, cx: &Cx<App>, channel: &Channel, p: Palette) -> El {
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
        message_list(s, cx, channel, p)
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

/// The thread header: title, applied tags, the parent channel and the thread
/// tools, matching the bar Discord shows above a thread.
fn thread_header(s: &App, channel: &Channel, p: Palette, trailing: f32) -> El {
    let Some(info) = channel.thread.clone() else {
        return chat_header(s, channel, p, trailing);
    };
    let parent = s.channel(info.parent_id).cloned();
    let forum = parent
        .as_ref()
        .is_some_and(|parent| parent.kind == ChannelKind::Forum);
    let parent_name = parent
        .as_ref()
        .map(|parent| parent.name.clone())
        .unwrap_or_else(|| "channel".to_string());
    let archived = info.archived;
    let id = channel.id;
    let parent_id = info.parent_id;
    let mut tag_row = row().gap(4.0).center_y();
    for tag in s
        .forum_tags
        .iter()
        .filter(|tag| tag.channel_id == parent_id && info.tags.contains(&tag.id))
    {
        tag_row = tag_row.child(forum_tag_pill(tag, p));
    }
    let mut people: Vec<Id> = Vec::new();
    if let Some(owner) = channel.owner_id {
        people.push(owner);
    }
    if let Some(messages) = s.messages.get(&id) {
        for message in messages.iter().rev() {
            if people.len() >= 5 {
                break;
            }
            if !people.contains(&message.author_id) {
                people.push(message.author_id);
            }
        }
    }
    let mut participants = row().gap(1.0).center_y();
    for user_id in people {
        if let Some(user) = s.user(user_id) {
            participants = participants.child(avatar_status(user, 22.0, p, p.header, false));
        }
    }
    let tools = row()
        .gap(2.0)
        .center_y()
        .child(participants)
        .child(
            tool_button("thread-notify", &icons::BELL, p)
                .on_press(move |s: &mut App| s.open_channel_notifications(id)),
        )
        .child(
            tool_button("thread-actions", &icons::MORE, p).on_press(move |s: &mut App| {
                s.context_menu = Some(crate::app::ContextMenu {
                    kind: crate::app::ContextMenuKind::Thread(id),
                    anchor: crate::app::ContextMenuAnchor::Element("thread-actions".into()),
                });
            }),
        )
        .child(
            tool_button("thread-close", &icons::CLOSE, p).on_press(|s: &mut App| s.close_thread()),
        );

    let header = row()
        .key("thread-header")
        .w_full()
        .h(52.0)
        .pl(20.0)
        .pr(8.0 + trailing)
        .gap(10.0)
        .center_y()
        .fill(p.header)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
        .window_drag();
    header
        .child(icon(if forum { &icons::FORUM } else { &icons::THREAD }, 22.0).color(p.text_faint))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(1.0)
                .justify_center()
                .child(
                    row()
                        .gap(8.0)
                        .center_y()
                        .child(
                            text(channel.name.clone())
                                .font(theme::NAME)
                                .color(p.text_bright)
                                .truncate()
                                .min_w(0.0),
                        )
                        .child(tag_row),
                )
                .child(
                    row()
                        .gap(5.0)
                        .center_y()
                        .child(
                            text(if forum { "Post in" } else { "Thread in" })
                                .font(theme::TINY)
                                .color(p.text_faint),
                        )
                        .child(
                            el().key("thread-parent")
                                .role("button")
                                .label(format!("Open #{parent_name}"))
                                .on_press(move |s: &mut App| s.switch_channel(parent_id))
                                .child(
                                    text(format!("#{parent_name}"))
                                        .font(theme::TINY)
                                        .color(p.text_link),
                                ),
                        )
                        .child(text("•").font(theme::TINY).color(p.text_faint))
                        .child(
                            text(format!(
                                "{} message{}",
                                info.message_count,
                                if info.message_count == 1 { "" } else { "s" }
                            ))
                            .font(theme::TINY)
                            .color(p.text_faint),
                        )
                        .children(archived.then(|| {
                            text(if forum { "CLOSED" } else { "ARCHIVED" })
                                .font(theme::TINY)
                                .color(p.yellow)
                                .border(1.0, p.yellow)
                                .radius(4.0)
                                .px(5.0)
                        })),
                ),
        )
        .child(tools)
}

/// The read-only bar Discord shows above an archived thread.
fn thread_banner(s: &App, channel: &Channel, p: Palette) -> Option<El> {
    let info = channel.thread.as_ref()?;
    if !info.archived {
        return None;
    }
    let id = channel.id;
    let forum = s
        .channel(info.parent_id)
        .is_some_and(|parent| parent.kind == ChannelKind::Forum);
    let can_reopen = s.can_manage_thread(channel);
    let mut banner = row()
        .key("thread-archived")
        .w_full()
        .h(44.0)
        .px(20.0)
        .gap(8.0)
        .center_y()
        .fill(p.sidebar)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
        .child(icon(&icons::LOCK, 16.0).color(p.yellow))
        .child(
            text(if forum {
                "This post is closed."
            } else {
                "This thread is archived."
            })
            .font(theme::SMALL_MED)
            .color(p.text_bright),
        )
        .child(
            text(if can_reopen {
                "Type a message to reopen it."
            } else {
                "Only the creator or a moderator can reopen it."
            })
            .font(theme::SMALL)
            .color(p.text_muted),
        )
        .child(el().grow(1.0));
    if can_reopen {
        banner = banner.child(
            el().key("thread-unarchive")
                .h(28.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label(if forum {
                    "Open Post"
                } else {
                    "Unarchive Thread"
                })
                .fill(p.accent)
                .hover(p.accent_hover)
                .on_press(move |s: &mut App| s.archive_thread(id, false))
                .child(
                    text(if forum {
                        "Open Post"
                    } else {
                        "Unarchive Thread"
                    })
                    .font(theme::LABEL)
                    .color(Color::WHITE),
                ),
        );
    }
    Some(banner)
}

fn dm_call_strip(s: &App, channel: &Channel, p: Palette) -> Option<El> {
    let participants: Vec<Id> = s
        .voice
        .values()
        .filter(|state| state.channel_id == Some(channel.id))
        .map(|state| state.user_id)
        .collect();
    if participants.is_empty() {
        return None;
    }
    let mut avatars = row().gap(6.0).center_y();
    for user in participants.iter().take(6) {
        let account = s.user(*user).cloned().unwrap_or_default();
        avatars = avatars.child(avatar(&account, 26.0, p));
    }
    let muted = s.voice_mute;
    let deafened = s.voice_deaf;
    Some(
        row()
            .key("dm-call")
            .w_full()
            .h(52.0)
            .px(20.0)
            .gap(10.0)
            .center_y()
            .fill(p.sidebar)
            .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
            .child(
                text("Voice connected")
                    .font(theme::SMALL_MED)
                    .color(p.green),
            )
            .child(avatars)
            .child(el().grow(1.0))
            .child(
                el().key("dm-call-mute")
                    .size(30.0, 30.0)
                    .radius(15.0)
                    .center()
                    .role("button")
                    .label(if muted { "Unmute" } else { "Mute" })
                    .fill(if muted { p.danger } else { p.input })
                    .hover(p.input_hover)
                    .on_press(|s: &mut App| s.toggle_voice_mute())
                    .child(icon(&icons::MIC, 16.0).color(Color::WHITE)),
            )
            .child(
                el().key("dm-call-deafen")
                    .size(30.0, 30.0)
                    .radius(15.0)
                    .center()
                    .role("button")
                    .label(if deafened { "Undeafen" } else { "Deafen" })
                    .fill(if deafened { p.danger } else { p.input })
                    .hover(p.input_hover)
                    .on_press(|s: &mut App| s.toggle_voice_deafen())
                    .child(icon(&icons::HEADPHONES, 16.0).color(Color::WHITE)),
            )
            .child(
                el().key("dm-call-camera")
                    .size(30.0, 30.0)
                    .radius(15.0)
                    .center()
                    .role("button")
                    .label("Toggle camera")
                    .fill(if s.voice_camera { p.accent } else { p.input })
                    .hover(p.input_hover)
                    .on_press(|s: &mut App| s.toggle_voice_camera())
                    .child(icon(&icons::VIDEO, 16.0).color(Color::WHITE)),
            )
            .child(
                el().key("dm-call-screen")
                    .size(30.0, 30.0)
                    .radius(15.0)
                    .center()
                    .role("button")
                    .label("Share your screen")
                    .fill(if s.voice_screen { p.accent } else { p.input })
                    .hover(p.input_hover)
                    .on_press(|s: &mut App| s.toggle_voice_screen())
                    .child(icon(&icons::MONITOR, 16.0).color(Color::WHITE)),
            )
            .child(
                el().key("dm-call-hangup")
                    .h(30.0)
                    .px(14.0)
                    .radius(15.0)
                    .center()
                    .role("button")
                    .label("Leave call")
                    .fill(p.danger)
                    .hover(p.danger.alpha(0.85))
                    .on_press(|s: &mut App| s.set_voice(None))
                    .child(text("Leave").font(theme::TINY).color(Color::WHITE)),
            ),
    )
}

fn chat_header(s: &App, channel: &Channel, p: Palette, trailing: f32) -> El {
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
        let mut title_row = row()
            .gap(8.0)
            .center_y()
            .child(crate::ui::dms::dm_avatar(s, &dm, 24.0, p, p.header, true))
            .child(
                text(title)
                    .font(theme::NAME)
                    .color(p.text_bright)
                    .truncate()
                    .min_w(0.0),
            );
        title_row = title_row.child(
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
        );
        title_row
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
                    .color(p.text_bright)
                    .truncate()
                    .min_w(0.0),
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
                tool_button("dm-call", &icons::PHONE, p).on_press(move |s: &mut App| {
                    if s.my_voice_channel() == Some(channel_id) {
                        s.set_voice(None);
                    } else {
                        s.set_voice(Some(channel_id));
                    }
                }),
            )
            .child(
                tool_button("dm-video", &icons::VIDEO, p).on_press(move |s: &mut App| {
                    s.set_voice(Some(channel_id));
                    if !s.voice_camera {
                        s.toggle_voice_camera();
                    }
                }),
            );
    } else {
        tools = tools
            .children((channel.kind != ChannelKind::Forum).then(|| {
                tool_button("threads", &icons::THREAD, p)
                    .on_press(|s: &mut App| s.modal = Modal::Threads)
            }))
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

    let header = row()
        .key("chat-header")
        .w_full()
        .h(50.0)
        .pl(20.0)
        .pr(8.0 + trailing)
        .gap(10.0)
        .center_y()
        .fill(p.header)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
        .window_drag();
    header.child(title_row).child(el().grow(1.0)).child(tools)
}

pub(crate) fn message_list(s: &App, cx: &Cx<App>, channel: &Channel, p: Palette) -> El {
    let messages = s
        .messages
        .get(&channel.id)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let count = messages.len();
    let channel_id = channel.id;
    // The viewport itself must not be `justify_end`: that makes overflow
    // unreachable to the wheel. The content column is at least one viewport
    // tall and bottom-aligned, so short conversations sit on the composer and
    // long ones still scroll normally.
    let viewport_height = cx
        .layout_size("messages-scroll")
        .map(|size| size[1])
        .unwrap_or(0.0);
    let scroll = col()
        .key("messages-scroll")
        .grow(1.0)
        .min_h(0.0)
        .scroll(ScrollSpec { stick_bottom: true })
        .scroll_edge_fade(8.0);
    let mut content = col()
        .key("messages-content")
        .w_full()
        .min_h(viewport_height)
        .justify_end();
    if channel.guild_id.is_some()
        && matches!(
            channel.kind,
            ChannelKind::Text | ChannelKind::Announcement | ChannelKind::Forum
        )
    {
        content = content.child(intro(s, channel, p)).child(el().grow(1.0));
    }
    content = content.child(
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
    );
    scroll.child(content)
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
                .wrap()
                .font(theme::INTRO)
                .color(p.text_bright),
        )
        .child(
            text(format!(
                "This is the start of the #{} channel.",
                channel.name
            ))
            .wrap()
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

enum MentionPiece {
    Text(String),
    User(Id, String),
    Role(Id, String),
    Channel(Id, String),
    /// `@everyone` or `@here`: plain-text words that ping the whole channel.
    Mass(String),
}

/// Where `@everyone` or `@here` starts in `text`, with its length and spelling.
/// Both are words, so a mention inside an email or a handle does not count.
fn mass_mention_at(text: &str) -> Option<(usize, usize, &'static str)> {
    let mut best: Option<(usize, usize, &'static str)> = None;
    for word in ["@everyone", "@here"] {
        let Some(at) = word_start(text, word) else {
            continue;
        };
        if best.is_none_or(|(best_at, _, _)| at < best_at) {
            best = Some((at, word.len(), word));
        }
    }
    best
}

/// The first whole-word `word` in `text`, so `@everyone` inside `someone@everyone.dev`
/// or `@everyones` is not a ping.
fn word_start(text: &str, word: &'static str) -> Option<usize> {
    let mut from = 0;
    while let Some(found) = text[from..].find(word) {
        let at = from + found;
        let before_ok = text[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric() && c != '@' && c != '_');
        let after_ok = text[at + word.len()..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');
        if before_ok && after_ok {
            return Some(at);
        }
        from = at + word.len();
    }
    None
}

/// Adds a text run to a wrap row, honouring the inline Markdown a composer
/// preview and a message share: bold, italic, strike and inline code.
fn push_inline_markdown(mut row_el: El, p: Palette, content: &str) -> El {
    let mut bold = false;
    let mut italic = false;
    let mut strike = false;
    let mut code = false;
    let mut buffer = String::new();
    let mut index = 0;
    while index < content.len() {
        let rest = &content[index..];
        let marker = if rest.starts_with("**") {
            Some((2, 0))
        } else if rest.starts_with("__") {
            Some((2, 1))
        } else if rest.starts_with("~~") {
            Some((2, 2))
        } else if rest.starts_with('`') {
            Some((1, 3))
        } else if rest.starts_with('*') || rest.starts_with('_') {
            Some((1, 4))
        } else {
            None
        };
        if let Some((size, kind)) = marker {
            row_el = flush_inline(row_el, &mut buffer, p, bold, italic, strike, code);
            match kind {
                0 | 1 => bold = !bold,
                2 => strike = !strike,
                3 => code = !code,
                _ => italic = !italic,
            }
            index += size;
            continue;
        }
        let ch = rest.chars().next().unwrap_or(' ');
        buffer.push(ch);
        index += ch.len_utf8();
    }
    flush_inline(row_el, &mut buffer, p, bold, italic, strike, code)
}

/// Unix timestamp tags (`<t:unix:style>`), rendered in local time.
fn expand_timestamps(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut rest = content;
    while let Some(start) = rest.find("<t:") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 3..];
        let Some(end) = after.find('>') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let inner = &after[..end];
        let mut parts = inner.split(':');
        let seconds = parts.next().and_then(|value| value.parse::<i64>().ok());
        let style = parts.next().unwrap_or("f");
        if let Some(seconds) = seconds
            && let Some(at) = chrono::DateTime::from_timestamp(seconds, 0)
        {
            let local = at.with_timezone(&chrono::Local);
            out.push_str(&match style {
                "t" => local.format("%H:%M").to_string(),
                "T" => local.format("%H:%M:%S").to_string(),
                "d" => local.format("%Y-%m-%d").to_string(),
                "D" => local.format("%B %-d, %Y").to_string(),
                "R" => {
                    let delta = runecord_protocol::now_ms() / 1000 - seconds;
                    if delta >= 0 {
                        format!("{} ago", relative_label(delta))
                    } else {
                        format!("in {}", relative_label(-delta))
                    }
                }
                _ => local.format("%B %-d, %Y %H:%M").to_string(),
            });
        } else {
            out.push_str(&rest[start..start + 3 + end + 1]);
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

fn relative_label(seconds: i64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3_600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86_400 {
        format!("{}h", seconds / 3_600)
    } else {
        format!("{}d", seconds / 86_400)
    }
}

fn flush_inline(
    row_el: El,
    buffer: &mut String,
    p: Palette,
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
) -> El {
    if buffer.is_empty() {
        return row_el;
    }
    let style = if code {
        theme::MONO
    } else {
        TextStyle {
            weight: if bold { 700.0 } else { 400.0 },
            italic,
            ..theme::BODY
        }
    };
    let length = buffer.len();
    let mut part =
        text(std::mem::take(buffer))
            .font(style)
            .color(if code { p.text_link } else { p.text });
    if strike {
        part = part.strike(0, length, Some(p.text_muted));
    }
    if code {
        row_el.child(el().px(3.0).radius(3.0).fill(p.code_bg).child(part))
    } else {
        row_el.child(part)
    }
}

/// Splits `||spoiler||` runs out of a message body. Spoiler runs stay hidden
/// behind a pill until the reader clicks them.
pub(crate) fn message_text_keyed(s: &App, content: &str, key: &str, p: Palette) -> El {
    let expanded = expand_timestamps(content);
    let content = expanded.as_str();
    if !content.contains("||") {
        return message_body(s, content, p);
    }
    let mut row_el = row().key("message-text").w_full().flex_wrap().gap(2.0);
    let mut rest = content;
    let mut hidden = false;
    let mut segment = 0usize;
    while let Some(at) = rest.find("||") {
        let chunk = &rest[..at];
        if !chunk.is_empty() {
            row_el = if hidden {
                row_el.child(spoiler_pill(s, chunk, key, segment, p))
            } else {
                push_inline_markdown(row_el, p, chunk)
            };
        }
        if hidden {
            segment += 1;
        }
        hidden = !hidden;
        rest = &rest[at + 2..];
    }
    if !rest.is_empty() {
        row_el = if hidden {
            row_el.child(spoiler_pill(s, rest, key, segment, p))
        } else {
            push_inline_markdown(row_el, p, rest)
        };
    }
    row_el
}

fn spoiler_pill(s: &App, content: &str, key: &str, segment: usize, p: Palette) -> El {
    let id = format!("{key}:{segment}");
    if s.revealed_spoilers.contains(&id) {
        return el()
            .px(3.0)
            .radius(3.0)
            .fill(p.code_bg)
            .child(text(content.to_string()).font(theme::BODY).color(p.text));
    }
    let reveal = id.clone();
    el().key(format!("spoiler-{id}"))
        .px(4.0)
        .radius(3.0)
        .role("button")
        .label("Reveal spoiler")
        .fill(p.input)
        .hover(p.input_hover)
        .on_press(move |s: &mut App| {
            s.revealed_spoilers.insert(reveal.clone());
        })
        .child(text(content.to_string()).font(theme::BODY).color(p.input))
}

fn message_body(s: &App, content: &str, p: Palette) -> El {
    let trimmed = content.trim();
    if trimmed.starts_with(':')
        && trimmed.ends_with(':')
        && trimmed.matches(':').count() == 2
        && let Some(emoji) = s.emojis.iter().find(|emoji| {
            emoji.sticker
                && emoji
                    .name
                    .eq_ignore_ascii_case(&trimmed[1..trimmed.len() - 1])
        })
        && let Some(pixels) = s.images.get(&emoji.url)
    {
        return el()
            .key("sticker-message")
            .size(112.0, 112.0)
            .pixels_fit(pixels, rune::SurfaceFit::Fit);
    }
    let mut parts: Vec<MentionPiece> = Vec::new();
    let mut rest = content;
    loop {
        let user_at = rest.find("<@");
        let channel_at = rest.find("<#");
        let mass = mass_mention_at(rest);
        let start = [user_at, channel_at, mass.map(|mass| mass.0)]
            .into_iter()
            .flatten()
            .min();
        let Some(start) = start else { break };
        if let Some((at, len, word)) = mass
            && at == start
        {
            if at > 0 {
                parts.push(MentionPiece::Text(rest[..at].to_string()));
            }
            parts.push(MentionPiece::Mass(word.to_string()));
            rest = &rest[at + len..];
            continue;
        }
        let Some(close) = rest[start + 2..].find('>') else {
            break;
        };
        let inner = &rest[start + 2..start + 2 + close];
        let channel_link = rest[start + 1..].starts_with('#');
        let (role, id) = if channel_link {
            (false, inner)
        } else {
            match inner.strip_prefix('&') {
                Some(value) => (true, value),
                None => (false, inner.strip_prefix('!').unwrap_or(inner)),
            }
        };
        let Ok(id) = id.parse::<runecord_protocol::Id>() else {
            break;
        };
        if start > 0 {
            parts.push(MentionPiece::Text(rest[..start].to_string()));
        }
        if channel_link {
            let label = s
                .channel(id)
                .map(|channel| channel.name.clone())
                .unwrap_or_else(|| "channel".into());
            parts.push(MentionPiece::Channel(id, label));
        } else if role {
            let label = s
                .guild_roles(s.active_guild)
                .iter()
                .find(|item| item.id == id)
                .map(|item| item.name.clone())
                .unwrap_or_else(|| "role".into());
            parts.push(MentionPiece::Role(id, label));
        } else {
            let label = s
                .user(id)
                .map(|user| user.display_name.clone())
                .unwrap_or_else(|| "user".into());
            parts.push(MentionPiece::User(id, label));
        }
        rest = &rest[start + 2 + close + 1..];
    }
    if parts.is_empty() {
        return markdown(s, content, p);
    }
    if !rest.is_empty() {
        parts.push(MentionPiece::Text(rest.to_string()));
    }
    let mut row_el = row().key("message-text").w_full().flex_wrap().gap(2.0);
    for (index, part) in parts.into_iter().enumerate() {
        match part {
            MentionPiece::Text(text_part) => {
                row_el = push_inline_markdown(row_el, p, &text_part);
            }
            MentionPiece::User(id, label) => {
                let mine = id == s.me.id;
                row_el = row_el.child(
                    el().key(format!("mention-user-{id}"))
                        .h(21.0)
                        .px(4.0)
                        .radius(3.0)
                        .fill(if mine {
                            p.mention
                        } else {
                            p.mention.alpha(0.35)
                        })
                        .hover(p.mention_hover)
                        .role("button")
                        .label(format!("View {label}'s profile"))
                        .on_press(move |s: &mut App| s.modal = Modal::Profile(id))
                        .center()
                        .child(
                            text(format!("@{label}"))
                                .font(theme::BODY)
                                .color(p.text_link),
                        ),
                );
            }
            MentionPiece::Role(id, label) => {
                let role = s
                    .guild_roles(s.active_guild)
                    .iter()
                    .find(|role| role.id == id);
                let known = role.is_some();
                // A role mention takes the role's own colour, tinted into the
                // pill background, the way Discord renders role pings.
                let tint = role
                    .filter(|role| role.color != 0)
                    .map(|role| theme::color(&format!("#{:06x}", role.color)))
                    .unwrap_or(p.mention);
                let mut pill = el()
                    .key(format!("mention-role-{id}"))
                    .h(21.0)
                    .px(4.0)
                    .radius(3.0)
                    .fill(tint.alpha(0.2))
                    .center()
                    .child(text(format!("@{label}")).font(theme::BODY).color(tint));
                if known {
                    pill = pill
                        .role("button")
                        .label(format!("Open the {label} role"))
                        .hover(p.mention_hover)
                        .on_press(move |s: &mut App| {
                            s.open_server_settings();
                            s.server_tab = crate::app::ServerTab::Roles;
                            s.select_role(id);
                        });
                }
                row_el = row_el.child(pill);
            }
            MentionPiece::Channel(id, label) => {
                row_el = row_el.child(
                    el().key(format!("mention-channel-{id}"))
                        .h(21.0)
                        .px(4.0)
                        .radius(3.0)
                        .fill(p.mention.alpha(0.35))
                        .hover(p.mention_hover)
                        .role("button")
                        .label(format!("Go to #{label}"))
                        .on_press(move |s: &mut App| {
                            match s.channel(id).map(|channel| channel.guild_id.is_some()) {
                                Some(true) => {
                                    s.screen = Screen::Guild;
                                    s.switch_channel(id);
                                }
                                Some(false) => s.open_dm(id),
                                None => s.toast("That channel is not available."),
                            }
                        })
                        .center()
                        .child(
                            text(format!("#{label}"))
                                .font(theme::BODY)
                                .color(p.text_link),
                        ),
                );
            }
            MentionPiece::Mass(word) => {
                row_el = row_el.child(
                    el().key(format!("mention-mass-{index}"))
                        .h(21.0)
                        .px(4.0)
                        .radius(3.0)
                        .fill(p.mention.alpha(0.32))
                        .center()
                        .child(text(word).font(theme::BODY).color(p.text_bright)),
                );
            }
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
    let nameplate_color = theme::nameplate_colors(&author.nameplate).map(|(primary, _, _)| primary);
    let name_color = if let Some(hook) = &msg.webhook {
        theme::color(&format!("#{:06x}", hook.color))
    } else if let Some(color) = nameplate_color {
        color
    } else if s.screen == Screen::Guild {
        s.member_color(s.active_guild, msg.author_id)
            .map(|c| theme::color(&format!("#{:06x}", c)))
            .unwrap_or(p.text_bright)
    } else {
        p.text_bright
    };
    let mut content = col().key("message-content").grow(1.0).min_w(0.0);

    if let Some(forwarded) = &msg.forwarded {
        content = content.child(
            row()
                .key("forwarded-head")
                .gap(4.0)
                .center_y()
                .pb(2.0)
                .child(icon(&icons::REPLY, 12.0).color(p.text_faint))
                .child(text("FORWARDED FROM").font(theme::TINY).color(p.text_faint))
                .child(
                    text(format!("#{}", forwarded.channel_name))
                        .font(theme::TINY)
                        .color(p.text_link),
                )
                .child(
                    text(format!("by {}", forwarded.author_name))
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .truncate()
                        .grow(1.0)
                        .min_w(0.0),
                ),
        );
    }

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
        content = content.child(message_text_keyed(s, &msg.content, &msg.id.to_string(), p));
    }

    if let Some(poll) = &msg.poll {
        content = content.child(poll_card(msg.id, poll, p));
    }

    for embed in &msg.embeds {
        content = content.child(embed_card(s, embed, p));
    }

    if !msg.components.is_empty() {
        content = content.child(components_row(msg, p));
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
    let can_send = s
        .channel(msg.channel_id)
        .is_some_and(|channel| s.can_send_in(channel));
    let can_react = has(perms, perm::ADD_REACTIONS);
    let row_msg_id = msg.id;
    let row_label = format!("{}: {}", author.display_name, msg.preview());
    let compact = s.compact;
    // Discord-style highlight: a message that mentions you, pings everyone, or
    // replies to you gets a tinted row and a left accent bar.
    let mentioned = msg.mentions.contains(&s.me.id)
        || msg.mention_everyone
        || (msg.author_id != s.me.id
            && msg
                .reply_to
                .as_ref()
                .is_some_and(|reply| reply.author_id == s.me.id));
    let pinned_highlight = s.pinned_target == Some(msg.id);
    let row_el = row()
        .key(format!("message-{}", msg.id))
        .w_full()
        .fill(if pinned_highlight {
            p.mention.alpha(0.35)
        } else if mentioned {
            p.mention.alpha(0.12)
        } else {
            Color::TRANSPARENT
        })
        .border_sides(
            [0.0, 0.0, 0.0, if mentioned { 2.0 } else { 0.0 }],
            p.mention,
        )
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
        .on_secondary_press(move |s: &mut App, press, _: &mut rune::Tx| {
            // The app draws its own menu: the native one follows the OS theme
            // and looks nothing like the rest of the client.
            s.context_menu = Some(crate::app::ContextMenu {
                kind: crate::app::ContextMenuKind::Message(row_msg_id),
                anchor: crate::app::ContextMenuAnchor::Pointer {
                    x: press.x,
                    y: press.y,
                },
            });
        });

    let mid = msg.id;
    let author_id = msg.author_id;
    let preview = msg.preview();
    let toolbar = row()
        .key(format!("msg-actions-{mid}"))
        .pin(None, Some(2.0), Some(16.0), None)
        .h(30.0)
        .w(96.0)
        .gap(0.0)
        .radius(4.0)
        .fill(p.chat)
        .border(1.0, p.divider)
        .shadow(2.0, 8.0, p.shadow)
        .group_opacity(0.0, 1.0)
        .children(can_react.then(|| {
            action_button_named(&icons::SMILE, "Add reaction", p, move |s: &mut App| {
                s.open_reaction(mid)
            })
        }))
        .children(can_send.then(|| {
            action_button_named(&icons::REPLY, "Reply", p, move |s: &mut App| {
                s.set_reply(mid, author_id, &preview.clone())
            })
        }))
        .child(action_button_named(
            &icons::MORE,
            "More actions",
            p,
            move |s: &mut App| {
                s.context_menu = Some(crate::app::ContextMenu {
                    kind: crate::app::ContextMenuKind::Message(mid),
                    anchor: crate::app::ContextMenuAnchor::Element(format!("msg-actions-{mid}")),
                });
            },
        ));

    let row_el = if grouped {
        row_el.child(content)
    } else {
        let size = if compact { 28.0 } else { 40.0 };
        let mut avatar_el = avatar(author, size, p);
        // A webhook has no profile behind it; everyone else opens their card.
        if msg.webhook.is_none() {
            let profile = msg.author_id;
            avatar_el = avatar_el
                .role("button")
                .label(format!("View {}'s profile", author.display_name))
                .on_press(move |s: &mut App| s.modal = Modal::Profile(profile));
        }
        row_el.child(avatar_el).child(content)
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

fn components_row(msg: &Message, p: Palette) -> El {
    let mut row_el = row()
        .key("components")
        .w_full()
        .gap(6.0)
        .pt(6.0)
        .flex_wrap();
    for component in &msg.components {
        let fill = match component.style.as_str() {
            "primary" => p.accent,
            "success" => p.green,
            "danger" => p.danger,
            "link" => p.input,
            _ => p.input,
        };
        let mut button = el()
            .key(format!("component-{}-{}", msg.id, component.label))
            .h(30.0)
            .px(12.0)
            .radius(4.0)
            .center()
            .role("button")
            .label(component.label.clone())
            .fill(fill)
            .hover(if fill == p.input {
                p.input_hover
            } else {
                fill.alpha(0.85)
            })
            .child(
                text(component.label.clone())
                    .font(theme::SMALL_MED)
                    .color(Color::WHITE),
            );
        if !component.url.is_empty() {
            let url = component.url.clone();
            button = button.on_press_tx(move |_: &mut App, tx: &mut rune::Tx| {
                let _ = tx.open_url(&url);
            });
        } else if !component.custom_id.is_empty() {
            let custom_id = component.custom_id.clone();
            let message = msg.id;
            button =
                button.on_press(move |s: &mut App| s.click_component(message, custom_id.clone()));
        }
        row_el = row_el.child(button);
    }
    row_el
}

fn embed_card(s: &App, embed: &runecord_protocol::Embed, p: Palette) -> El {
    let mut card = col()
        .key("embed")
        .w_full()
        .max_w(440.0)
        .radius(6.0)
        .clip_rounded(6.0)
        .fill(p.card)
        .border_sides([0.0, 0.0, 0.0, 3.0], p.accent)
        .padding(10.0)
        .gap(4.0);
    if !embed.site_name.is_empty() {
        card = card.child(
            text(embed.site_name.clone())
                .font(theme::TINY)
                .color(p.text_muted),
        );
    }
    if !embed.title.is_empty() {
        card = card.child(
            text(embed.title.clone())
                .font(theme::BODY_BOLD)
                .color(p.text_link),
        );
    }
    if !embed.description.is_empty() {
        card = card.child(
            text(embed.description.clone())
                .font(theme::SMALL)
                .color(p.text)
                .max_lines(4),
        );
    }
    let image_key = format!("{}?w=480", embed.image_url);
    if !embed.image_url.is_empty()
        && let Some(pixels) = s.images.get(&image_key)
    {
        card = card.child(
            el().w_full()
                .h(180.0)
                .radius(4.0)
                .clip_rounded(4.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fill),
        );
    }
    let url = embed.url.clone();
    card.role("button")
        .label(format!("Open {}", embed.url))
        .hover(p.card_hover)
        .on_press_tx(move |_: &mut App, tx: &mut rune::Tx| {
            let _ = tx.open_url(&url);
        })
}

fn poll_card(message: Id, poll: &runecord_protocol::Poll, p: Palette) -> El {
    let mut card = col()
        .key(format!("poll-{message}"))
        .w_full()
        .max_w(440.0)
        .radius(8.0)
        .fill(p.input)
        .border(1.0, p.divider)
        .padding(12.0)
        .gap(8.0)
        .child(
            text(poll.question.clone())
                .font(theme::BODY_BOLD)
                .color(p.text_bright),
        );
    for option in &poll.options {
        let percent = if poll.total_votes == 0 {
            0.0
        } else {
            option.votes as f32 / poll.total_votes as f32
        };
        let mut row_el = row()
            .key(format!("poll-option-{message}-{}", option.index))
            .w_full()
            .h(36.0)
            .radius(4.0)
            .clip_rounded(4.0)
            .fill(if option.me { p.mention } else { p.card })
            .border(1.0, if option.me { p.accent } else { p.divider_soft })
            .px(10.0)
            .gap(8.0)
            .center_y();
        if !poll.closed {
            let index = option.index;
            row_el = row_el
                .role("button")
                .label(format!("Vote for {}", option.text))
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.vote_poll(message, index));
        }
        card = card.child(
            row_el
                .child(
                    text(option.text.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                )
                .child(
                    text(format!("{}%", (percent * 100.0).round() as u32))
                        .font(theme::TINY)
                        .color(if option.me {
                            p.text_bright
                        } else {
                            p.text_muted
                        }),
                )
                .child(
                    text(format!("({})", option.votes))
                        .font(theme::TINY)
                        .color(p.text_faint),
                )
                .children(option.me.then(|| icon(&icons::CHECK, 13.0).color(p.accent))),
        );
    }
    let mut meta = format!(
        "{} {}",
        poll.total_votes,
        if poll.total_votes == 1 {
            "vote"
        } else {
            "votes"
        }
    );
    if poll.multiple {
        meta.push_str(" - Multiple choice");
    }
    if poll.anonymous {
        meta.push_str(" - Anonymous");
    }
    if poll.closed {
        meta.push_str(" - Poll closed");
    } else if let Some(at) = poll.expires_at {
        let remaining = at.saturating_sub(now_ms());
        let hours = remaining / (60 * 60 * 1000);
        if hours > 0 {
            meta.push_str(&format!(" - Ends in {hours}h"));
        } else {
            meta.push_str(&format!(" - Ends in {}m", remaining / (60 * 1000)));
        }
    }
    card.child(text(meta).font(theme::TINY).color(p.text_faint))
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

/// A tag chip in the forum filter bar. Chips are checkboxes, so picking more
/// than one narrows the list to posts carrying all of them.
fn forum_tag_chip(tag: &runecord_protocol::ForumTag, selected: bool, p: Palette) -> El {
    let id = tag.id;
    let tint = theme::avatar_color(id.rem_euclid(10) as u32);
    el().key(format!("forum-tag-{id}"))
        .h(26.0)
        .px(10.0)
        .radius(13.0)
        .center()
        .role("checkbox")
        .label(format!("Filter by {}", tag.name))
        .checked(selected)
        .fill(if selected { tint.alpha(0.3) } else { p.input })
        .hover(if selected {
            tint.alpha(0.4)
        } else {
            p.card_hover
        })
        .on_press(move |s: &mut App| {
            if !s.forum_tag_filter.remove(&id) {
                s.forum_tag_filter.insert(id);
            }
        })
        .child(
            text(format!("{}{}", tag.emoji, tag.name))
                .font(theme::TINY)
                .color(if selected {
                    p.text_bright
                } else {
                    p.text_muted
                }),
        )
}

/// Sort and layout controls sit to the right of the tag filters, as they do on
/// a Discord forum channel.
fn forum_toolbar(s: &App, tags: &[runecord_protocol::ForumTag], p: Palette) -> El {
    let all_selected = s.forum_tag_filter.is_empty();
    let mut chips = row().gap(6.0).flex_wrap().grow(1.0).min_w(0.0);
    chips = chips.child(
        el().key("forum-tag-all")
            .h(26.0)
            .px(10.0)
            .radius(13.0)
            .center()
            .role("radio")
            .label("All posts")
            .checked(all_selected)
            .fill(if all_selected { p.accent } else { p.input })
            .hover(if all_selected {
                p.accent_hover
            } else {
                p.card_hover
            })
            .on_press(|s: &mut App| s.forum_tag_filter.clear())
            .child(text("All").font(theme::TINY).color(if all_selected {
                Color::WHITE
            } else {
                p.text_muted
            })),
    );
    for tag in tags {
        chips = chips.child(forum_tag_chip(tag, s.forum_tag_filter.contains(&tag.id), p));
    }
    row()
        .key("forum-toolbar")
        .w_full()
        .px(24.0)
        .pt(10.0)
        .gap(10.0)
        .center_y()
        .child(chips)
        .child(
            el().key("forum-sort")
                .h(26.0)
                .px(8.0)
                .radius(4.0)
                .gap(4.0)
                .center_y()
                .role("button")
                .label("Sort posts")
                .help("Sort posts")
                .fill(p.input)
                .hover(p.input_hover)
                .on_press(|s: &mut App| s.forum_sort_new = !s.forum_sort_new)
                .child(
                    text(if s.forum_sort_new {
                        "Creation Date"
                    } else {
                        "Latest Activity"
                    })
                    .font(theme::TINY)
                    .color(p.text_muted),
                )
                .child(icon(&icons::CHEVRON_DOWN, 12.0).color(p.text_faint)),
        )
        .child(
            el().key("forum-layout")
                .h(26.0)
                .px(8.0)
                .radius(4.0)
                .gap(4.0)
                .center_y()
                .role("button")
                .label("Forum layout")
                .help("Switch between gallery and list view")
                .fill(p.input)
                .hover(p.input_hover)
                .on_press(|s: &mut App| s.forum_list_layout = !s.forum_list_layout)
                .child(
                    text(if s.forum_list_layout {
                        "List"
                    } else {
                        "Gallery"
                    })
                    .font(theme::TINY)
                    .color(p.text_muted),
                )
                .child(icon(&icons::CHEVRON_DOWN, 12.0).color(p.text_faint)),
        )
}

fn forum_more_button(thread: Id, p: Palette) -> El {
    el().key(format!("forum-actions-{thread}"))
        .size(24.0, 24.0)
        .radius(4.0)
        .center()
        .role("button")
        .label("Post actions")
        .help("Post actions")
        .hover(p.rail)
        .group_opacity(0.0, 1.0)
        .on_press(move |s: &mut App| {
            s.context_menu = Some(crate::app::ContextMenu {
                kind: crate::app::ContextMenuKind::Thread(thread),
                anchor: crate::app::ContextMenuAnchor::Element(format!("forum-actions-{thread}")),
            });
        })
        .child(icon(&icons::MORE, 16.0).color(p.text_muted))
}

/// A gallery tile: title, tags, preview and the author/message footer.
fn forum_post_card(
    s: &App,
    thread: &Channel,
    tags: &[runecord_protocol::ForumTag],
    p: Palette,
) -> El {
    let id = thread.id;
    let Some(info) = thread.thread.clone() else {
        return el();
    };
    let archived = info.archived;
    let unread = s.unread.get(&id).copied().unwrap_or(0) > 0;
    let applied: Vec<runecord_protocol::ForumTag> = tags
        .iter()
        .filter(|tag| info.tags.contains(&tag.id))
        .cloned()
        .collect();
    let preview = s
        .messages
        .get(&id)
        .and_then(|messages| messages.first())
        .map(|message| message.preview())
        .filter(|preview| !preview.is_empty())
        .or_else(|| Some(info.preview.clone()).filter(|preview| !preview.is_empty()))
        .unwrap_or_else(|| "No messages yet.".to_string());
    let owner = thread.owner_id.and_then(|owner| s.user(owner)).cloned();
    let mut tag_row = row().gap(4.0).flex_wrap();
    for tag in &applied {
        tag_row = tag_row.child(forum_tag_pill(tag, p));
    }
    let mut card = col()
        .key(format!("forum-{id}"))
        .w(300.0)
        .min_h(168.0)
        .radius(8.0)
        .fill(p.card)
        .border(1.0, p.divider)
        .padding(14.0)
        .gap(8.0)
        .role("button")
        .label(format!("Open post {}", thread.name))
        .hover(p.card_hover)
        .hover_group()
        .on_press(move |s: &mut App| s.open_thread(id))
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
            row()
                .w_full()
                .gap(6.0)
                .center_y()
                .children(unread.then(|| el().size(8.0, 8.0).radius(4.0).fill(p.accent)))
                .child(
                    icon(
                        if archived {
                            &icons::LOCK
                        } else {
                            &icons::THREAD
                        },
                        16.0,
                    )
                    .color(p.text_faint),
                )
                .child(
                    text(thread.name.clone())
                        .font(theme::BODY_BOLD)
                        .color(if archived {
                            p.text_muted
                        } else {
                            p.text_bright
                        })
                        .grow(1.0)
                        .min_w(0.0)
                        .max_lines(2),
                )
                .child(forum_more_button(id, p)),
        );
    if !applied.is_empty() {
        card = card.child(tag_row);
    }
    card = card.child(
        text(preview)
            .font(theme::SMALL)
            .color(p.text_muted)
            .max_lines(3)
            .grow(1.0)
            .min_w(0.0),
    );
    let mut footer = row().w_full().gap(6.0).center_y();
    match owner {
        Some(owner) => {
            footer = footer
                .child(avatar_status(&owner, 20.0, p, p.card, false))
                .child(
                    text(owner.display_name.clone())
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .truncate()
                        .min_w(0.0)
                        .grow(1.0),
                );
        }
        None => footer = footer.child(el().grow(1.0)),
    }
    footer = footer
        .child(icon(&icons::CHAT, 12.0).color(p.text_faint))
        .child(
            text(info.message_count.to_string())
                .font(theme::TINY)
                .color(p.text_faint),
        );
    card.child(footer)
}

/// The list-view row: a wide row with the title, tags, preview and activity on
/// the right, which is easier to scan in text-heavy forums.
fn forum_post_row(
    s: &App,
    thread: &Channel,
    tags: &[runecord_protocol::ForumTag],
    p: Palette,
) -> El {
    let id = thread.id;
    let Some(info) = thread.thread.clone() else {
        return el();
    };
    let archived = info.archived;
    let unread = s.unread.get(&id).copied().unwrap_or(0) > 0;
    let applied: Vec<runecord_protocol::ForumTag> = tags
        .iter()
        .filter(|tag| info.tags.contains(&tag.id))
        .cloned()
        .collect();
    let preview = s
        .messages
        .get(&id)
        .and_then(|messages| messages.first())
        .map(|message| message.preview())
        .filter(|preview| !preview.is_empty())
        .or_else(|| Some(info.preview.clone()).filter(|preview| !preview.is_empty()))
        .unwrap_or_default();
    let owner = thread.owner_id.and_then(|owner| s.user(owner)).cloned();
    let mut tag_row = row().gap(4.0).center_y();
    for tag in &applied {
        tag_row = tag_row.child(forum_tag_pill(tag, p));
    }
    row()
        .key(format!("forum-row-{id}"))
        .w_full()
        .min_h(64.0)
        .px(12.0)
        .py(10.0)
        .radius(8.0)
        .gap(12.0)
        .center_y()
        .hover(p.card_hover)
        .hover_group()
        .role("button")
        .label(format!("Open post {}", thread.name))
        .on_press(move |s: &mut App| s.open_thread(id))
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
            el().size(38.0, 38.0)
                .radius(19.0)
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
                .gap(4.0)
                .child(
                    row()
                        .gap(6.0)
                        .center_y()
                        .children(unread.then(|| el().size(8.0, 8.0).radius(4.0).fill(p.accent)))
                        .child(
                            text(thread.name.clone())
                                .font(theme::BODY_BOLD)
                                .color(if archived {
                                    p.text_muted
                                } else {
                                    p.text_bright
                                })
                                .truncate(),
                        )
                        .children(archived.then(|| {
                            text("CLOSED")
                                .font(theme::TINY)
                                .color(p.yellow)
                                .border(1.0, p.yellow)
                                .radius(4.0)
                                .px(4.0)
                        }))
                        .child(tag_row),
                )
                .child(
                    text(preview)
                        .font(theme::SMALL)
                        .color(p.text_muted)
                        .truncate(),
                ),
        )
        .child(
            col()
                .items_end()
                .gap(3.0)
                .child(match owner {
                    Some(owner) => row()
                        .gap(5.0)
                        .center_y()
                        .child(avatar_status(&owner, 16.0, p, p.chat, false))
                        .child(
                            text(owner.display_name.clone())
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                    None => row(),
                })
                .child(
                    row()
                        .gap(4.0)
                        .center_y()
                        .child(icon(&icons::CHAT, 12.0).color(p.text_faint))
                        .child(
                            text(format!(
                                "{} message{}",
                                info.message_count,
                                if info.message_count == 1 { "" } else { "s" }
                            ))
                            .font(theme::TINY)
                            .color(p.text_faint),
                        ),
                ),
        )
        .child(forum_more_button(id, p))
}

fn forum_post_grid(
    s: &App,
    threads: &[&Channel],
    tags: &[runecord_protocol::ForumTag],
    p: Palette,
) -> El {
    if s.forum_list_layout {
        let mut list = col().w_full().gap(4.0);
        for thread in threads {
            list = list.child(forum_post_row(s, thread, tags, p));
        }
        list
    } else {
        let mut grid = row().w_full().gap(12.0).flex_wrap();
        for thread in threads {
            grid = grid.child(forum_post_card(s, thread, tags, p));
        }
        grid
    }
}

fn forum_section_header(label: &str, count: usize, p: Palette) -> El {
    row()
        .w_full()
        .h(38.0)
        .gap(8.0)
        .center_y()
        .child(text(label).font(theme::SECTION).color(p.text_muted))
        .child(
            text(count.to_string())
                .font(theme::TINY)
                .color(p.text_faint),
        )
}

/// A forum channel: tag filters and sort controls, then active posts and the
/// older (closed) ones, rendered as gallery tiles or list rows.
pub(crate) fn forum_view(s: &App, channel: &Channel, p: Palette, header: El) -> El {
    let parent = channel.id;
    let tags: Vec<runecord_protocol::ForumTag> = s
        .forum_tags
        .iter()
        .filter(|tag| tag.channel_id == parent)
        .cloned()
        .collect();
    let mut threads: Vec<&Channel> =
        s.guild_channels(s.active_guild)
            .iter()
            .filter(|thread| {
                thread.kind == ChannelKind::Thread
                    && thread.thread.as_ref().map(|info| info.parent_id) == Some(parent)
            })
            .filter(|thread| {
                thread.thread.as_ref().is_some_and(|info| {
                    s.forum_tag_filter.iter().all(|tag| info.tags.contains(tag))
                })
            })
            .collect();
    if s.forum_sort_new {
        threads.sort_by_key(|thread| std::cmp::Reverse(thread.id));
    } else {
        threads
            .sort_by_key(|thread| std::cmp::Reverse(thread.last_message_id.unwrap_or(thread.id)));
    }
    let (active, older): (Vec<&Channel>, Vec<&Channel>) = threads
        .into_iter()
        .partition(|thread| !thread.thread.as_ref().is_some_and(|info| info.archived));
    let can_post = s.can_start_thread(channel);
    let mut body = col().w_full().gap(12.0).pb(24.0);

    body = body.child(
        row()
            .w_full()
            .gap(12.0)
            .center_y()
            .child(forum_section_header("POSTS", active.len(), p).grow(1.0))
            .children(can_post.then(|| {
                let parent = channel.id;
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
                        s.thread_tag_ids.clear();
                        s.forum_guidelines_ok = s
                            .channel(parent)
                            .is_none_or(|channel| channel.topic.trim().is_empty());
                        s.modal = Modal::ThreadCreate {
                            parent,
                            message: None,
                        };
                    })
                    .child(icon(&icons::PLUS, 16.0).color(Color::WHITE))
                    .child(text("New Post").font(theme::SMALL_MED).color(Color::WHITE))
            })),
    );

    if active.is_empty() && older.is_empty() {
        body = body.child(
            col()
                .w_full()
                .py(40.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::FORUM, 48.0).color(p.text_faint))
                .child(
                    text(if s.forum_tag_filter.is_empty() {
                        "No posts yet"
                    } else {
                        "No posts match these tags"
                    })
                    .font(theme::BODY)
                    .color(p.text_muted),
                )
                .child(
                    text(if s.forum_tag_filter.is_empty() {
                        "Start the first conversation."
                    } else {
                        "Clear a tag filter to see more posts."
                    })
                    .font(theme::SMALL)
                    .color(p.text_faint),
                ),
        );
    } else if !active.is_empty() {
        body = body.child(forum_post_grid(s, &active, &tags, p));
    }

    if !older.is_empty() {
        body = body
            .child(el().h(1.0).w_full().fill(p.divider_soft).my(6.0))
            .child(forum_section_header("OLDER POSTS", older.len(), p))
            .child(forum_post_grid(s, &older, &tags, p));
    }

    col()
        .key("forum")
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .fill(p.chat)
        .child(header)
        .child(forum_toolbar(s, &tags, p))
        .child(
            col()
                .grow(1.0)
                .min_h(0.0)
                .px(24.0)
                .pt(8.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(body),
        )
}

#[cfg(test)]
mod tests {
    use super::{mass_mention_at, video_tile_size};

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

    #[test]
    fn mass_mentions_match_whole_words() {
        assert_eq!(mass_mention_at("@everyone hi"), Some((0, 9, "@everyone")));
        assert_eq!(mass_mention_at("hi @here"), Some((3, 5, "@here")));
        // Part of an address or a longer word is not a ping.
        assert!(mass_mention_at("email@everyone.com").is_none());
        assert!(mass_mention_at("@everyones").is_none());
        assert!(mass_mention_at("@everyone_").is_none());
    }
}
