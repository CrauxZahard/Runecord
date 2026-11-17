//! Demo seed and benchmark fixtures.

use super::*;

impl App {
    pub(crate) fn seed(&mut self) {
        let t = now_ms();
        let mk = |id, username: &str, display: &str, color: u32, status| {
            (
                id,
                User {
                    id,
                    username: username.into(),
                    display_name: display.into(),
                    avatar_color: color,
                    avatar_border: 0,
                    banner_color: 0,
                    bio: String::new(),
                    custom_status: String::new(),
                    pronouns: String::new(),
                    avatar_url: String::new(),
                    banner_url: String::new(),
                    badges: Vec::new(),
                    avatar_decoration: String::new(),
                    profile_effect: String::new(),
                    nameplate: String::new(),
                    profile_widget: String::new(),
                    activity: None,
                    connections: Vec::new(),
                    status,
                    is_bot: false,
                    created_at: t,
                    dm_privacy: Default::default(),
                    friend_privacy: Default::default(),
                },
            )
        };
        let people = vec![
            mk(1, "nova", "Nova", 0, Presence::Online),
            mk(2, "maya", "Maya Chen", 4, Presence::Online),
            mk(3, "leo", "Leo Park", 1, Presence::Dnd),
            mk(4, "sam", "Sam Rivera", 2, Presence::Idle),
            mk(5, "ana", "Ana Souza", 3, Presence::Online),
            mk(6, "priya", "Priya Nair", 5, Presence::Online),
            mk(7, "tom", "Tom Vale", 6, Presence::Offline),
            mk(8, "runebot", "Runebot", 9, Presence::Online),
        ];
        for (id, mut user) in people {
            if id == 2 {
                user.custom_status = "building tiny UIs".into();
                user.pronouns = "she/her".into();
                user.badges = vec!["nitro".into(), "booster".into()];
                user.avatar_decoration = "sakura".into();
            }
            if id == 1 {
                user.pronouns = "they/them".into();
                user.badges = vec!["staff".into(), "early_supporter".into(), "nitro".into()];
                user.avatar_decoration = "aurora".into();
            }
            if id == 5 {
                user.custom_status = "shipping pixels".into();
            }
            self.presence.insert(id, user.status);
            self.users.insert(id, user);
        }
        self.me = self.users[&1].clone();

        let role = |id, name: &str, color: u32, permissions: u64, position: i32| Role {
            id,
            guild_id: 100,
            name: name.into(),
            color,
            permissions,
            position,
            hoist: position > 0,
            mentionable: true,
            icon_url: String::new(),
        };
        let roles = vec![
            role(900, "@everyone", 0, perm::EVERYONE, 0),
            role(
                901,
                "Moderator",
                0x3ba55d,
                perm::EVERYONE | perm::MANAGE_MESSAGES | perm::KICK_MEMBERS,
                2,
            ),
            role(902, "Designer", 0xeb459e, perm::EVERYONE, 1),
        ];

        let ch =
            |id, name: &str, kind: ChannelKind, position: i32, parent: Option<Id>, topic: &str| {
                Channel {
                    id,
                    guild_id: Some(100),
                    name: name.into(),
                    kind,
                    position,
                    parent_id: parent,
                    topic: topic.into(),
                    my_permissions: u64::MAX / 2,
                    ..Channel::default()
                }
            };
        let guild_channels = vec![
            Channel {
                id: 200,
                guild_id: Some(100),
                name: "Information".into(),
                kind: ChannelKind::Category,
                position: 0,
                ..Channel::default()
            },
            ch(
                201,
                "welcome",
                ChannelKind::Text,
                1,
                Some(200),
                "Start here",
            ),
            ch(
                202,
                "announcements",
                ChannelKind::Announcement,
                2,
                Some(200),
                "News from the team",
            ),
            Channel {
                id: 210,
                guild_id: Some(100),
                name: "Text Channels".into(),
                kind: ChannelKind::Category,
                position: 3,
                ..Channel::default()
            },
            ch(
                211,
                "general",
                ChannelKind::Text,
                4,
                Some(210),
                "Talk about anything and everything",
            ),
            ch(
                212,
                "design",
                ChannelKind::Text,
                5,
                Some(210),
                "Screens, type, motion",
            ),
            ch(
                213,
                "dev",
                ChannelKind::Text,
                6,
                Some(210),
                "Build logs and bugs",
            ),
            ch(
                214,
                "off-topic",
                ChannelKind::Text,
                7,
                Some(210),
                "Anything goes",
            ),
            Channel {
                id: 220,
                guild_id: Some(100),
                name: "Voice Channels".into(),
                kind: ChannelKind::Category,
                position: 8,
                ..Channel::default()
            },
            ch(221, "General", ChannelKind::Voice, 9, Some(220), ""),
            ch(222, "Focus Room", ChannelKind::Voice, 10, Some(220), ""),
            ch(
                223,
                "town-hall",
                ChannelKind::Stage,
                11,
                Some(220),
                "Roadmap AMA with the team",
            ),
            Channel {
                id: 230,
                guild_id: Some(100),
                name: "Community".into(),
                kind: ChannelKind::Category,
                position: 11,
                ..Channel::default()
            },
            ch(
                215,
                "forum",
                ChannelKind::Forum,
                12,
                Some(230),
                "Long-form posts with tags",
            ),
            Channel {
                id: 216,
                guild_id: Some(100),
                name: "Keyboard shortcuts cheat sheet".into(),
                kind: ChannelKind::Thread,
                position: 13,
                parent_id: Some(215),
                owner_id: Some(3),
                last_message_id: Some(3001),
                my_permissions: u64::MAX / 2,
                thread: Some(runecord_protocol::ThreadInfo {
                    parent_id: 215,
                    name: "Keyboard shortcuts cheat sheet".into(),
                    message_count: 4,
                    archived: false,
                    preview: "Every shortcut I use daily, in one place.".into(),
                    tags: vec![2301],
                }),
                ..Channel::default()
            },
            Channel {
                id: 217,
                guild_id: Some(100),
                name: "Sidebar spacing feels off on 1440p".into(),
                kind: ChannelKind::Thread,
                position: 14,
                parent_id: Some(215),
                owner_id: Some(2),
                last_message_id: Some(3101),
                my_permissions: u64::MAX / 2,
                thread: Some(runecord_protocol::ThreadInfo {
                    parent_id: 215,
                    name: "Sidebar spacing feels off on 1440p".into(),
                    message_count: 2,
                    archived: false,
                    preview: "The 12px gap looks bigger than the 8px one at that scale.".into(),
                    tags: vec![2302, 2303],
                }),
                ..Channel::default()
            },
        ];
        let members = vec![
            Member {
                guild_id: 100,
                user: self.users[&1].clone(),
                nickname: String::new(),
                roles: vec![901],
                joined_at: t,
                timeout_until: 0,
            },
            Member {
                guild_id: 100,
                user: self.users[&2].clone(),
                nickname: String::new(),
                roles: vec![901],
                joined_at: t,
                timeout_until: 0,
            },
            Member {
                guild_id: 100,
                user: self.users[&5].clone(),
                nickname: String::new(),
                roles: vec![902],
                joined_at: t,
                timeout_until: 0,
            },
            Member {
                guild_id: 100,
                user: self.users[&3].clone(),
                nickname: String::new(),
                roles: vec![],
                joined_at: t,
                timeout_until: 0,
            },
            Member {
                guild_id: 100,
                user: self.users[&4].clone(),
                nickname: String::new(),
                roles: vec![],
                joined_at: t,
                timeout_until: 0,
            },
            Member {
                guild_id: 100,
                user: self.users[&6].clone(),
                nickname: String::new(),
                roles: vec![],
                joined_at: t,
                timeout_until: 0,
            },
            Member {
                guild_id: 100,
                user: self.users[&7].clone(),
                nickname: String::new(),
                roles: vec![],
                joined_at: t,
                timeout_until: 0,
            },
            Member {
                guild_id: 100,
                user: self.users[&8].clone(),
                nickname: String::new(),
                roles: vec![],
                joined_at: t,
                timeout_until: 0,
            },
        ];
        let summary = GuildSummary {
            id: 100,
            name: "Rune Lounge".into(),
            icon_color: 0x5865f2,
            owner_id: 1,
            description: "A home for interface obsessives.".into(),
            member_count: members.len() as u32,
            tag: "RUNE".into(),
            permissions: u64::MAX / 2,
            unread_count: 0,
            mention_count: 0,
            created_at: t,
            discoverable: true,
            icon_url: String::new(),
            banner_url: String::new(),
            boosts: 0,
            boost_level: 0,
            boosted_by_me: false,
            reputation_cooldown_until: 0,
            upload_limit_bytes: 128 * 1024 * 1024,
            voice_kbps: 384,
        };
        self.guilds.push(summary.clone());
        self.guild_details.insert(
            100,
            GuildDetail {
                guild: summary,
                channels: guild_channels.clone(),
                roles: roles.clone(),
                overwrites: Vec::new(),
                permissions: u64::MAX / 2,
                onboarding: runecord_protocol::GuildOnboarding::default(),
                forum_tags: vec![
                    runecord_protocol::ForumTag {
                        id: 2301,
                        channel_id: 215,
                        name: "Guide".into(),
                        emoji: String::new(),
                        position: 0,
                    },
                    runecord_protocol::ForumTag {
                        id: 2302,
                        channel_id: 215,
                        name: "Bug".into(),
                        emoji: String::new(),
                        position: 1,
                    },
                    runecord_protocol::ForumTag {
                        id: 2303,
                        channel_id: 215,
                        name: "Design".into(),
                        emoji: String::new(),
                        position: 2,
                    },
                ],
            },
        );
        self.forum_tags = self
            .guild_details
            .get(&100)
            .map(|detail| detail.forum_tags.clone())
            .unwrap_or_default();
        self.channels.insert(100, guild_channels);
        self.members.insert(100, members.clone());
        self.member_state.insert(
            100,
            MemberState {
                total: members.len() as u32,
                next_offset: None,
                loading: false,
                complete: true,
                query: String::new(),
                channel: 0,
            },
        );

        for (id, name, color, channels, count) in [
            (
                300,
                "Design Guild",
                0xeb459e,
                vec![(301, "general"), (302, "critique")],
                128,
            ),
            (
                400,
                "Rustaceans",
                0xf0b232,
                vec![(401, "general"), (402, "show-and-tell")],
                512,
            ),
            (500, "Game Night", 0x3ba55d, vec![(501, "general")], 42),
        ] {
            let list: Vec<Channel> = channels
                .into_iter()
                .enumerate()
                .map(|(i, (cid, cname))| Channel {
                    id: cid,
                    guild_id: Some(id),
                    name: cname.into(),
                    kind: ChannelKind::Text,
                    position: i as i32,
                    topic: "Say hello".into(),
                    ..Channel::default()
                })
                .collect();
            self.channels.insert(id, list.clone());
            self.members.insert(id, Vec::new());
            self.member_state.insert(
                id,
                MemberState {
                    total: count,
                    next_offset: None,
                    loading: false,
                    complete: true,
                    query: String::new(),
                    channel: 0,
                },
            );
            let summary = GuildSummary {
                id,
                name: name.into(),
                icon_color: color,
                owner_id: 1,
                description: String::new(),
                member_count: count,
                tag: String::new(),
                permissions: u64::MAX / 2,
                unread_count: 0,
                mention_count: 0,
                created_at: t,
                discoverable: true,
                icon_url: String::new(),
                banner_url: String::new(),
                boosts: 0,
                boost_level: 0,
                boosted_by_me: false,
                reputation_cooldown_until: 0,
                upload_limit_bytes: 128 * 1024 * 1024,
                voice_kbps: 384,
            };
            self.guilds.push(summary.clone());
            self.guild_details.insert(
                id,
                GuildDetail {
                    guild: summary,
                    channels: list,
                    roles: vec![roles[0].clone()],
                    overwrites: Vec::new(),
                    permissions: u64::MAX / 2,
                    onboarding: runecord_protocol::GuildOnboarding::default(),
                    forum_tags: Vec::new(),
                },
            );
        }

        let msg = |id: Id, author: Id, minutes: i64, content: &str| Message {
            id,
            channel_id: 211,
            author_id: author,
            content: content.into(),
            timestamp: t - minutes * 60_000,
            ..Message::default()
        };
        let mut general = vec![
            msg(1000, 2, 310, "morning! did the new sidebar land?"),
            msg(1001, 1, 308, "yeah — it's built in Rune now. 120 fps and the whole client is ~9 MB of RAM"),
            msg(1002, 2, 306, "nine. megabytes."),
            msg(1003, 2, 305, "electron could never"),
            Message {
                reply_to: Some(MessageReply {
                    message_id: 1001,
                    author_id: 1,
                    author_name: "Nova".into(),
                    content: "it's built in Rune now. 120 fps and the whole client is ~9 MB of RAM".into(),
                }),
                kind: MessageKind::Reply,
                ..msg(1004, 4, 300, "wait, with the virtualized message list? what does a 10k message channel cost?")
            },
            msg(1005, 1, 298, "same. rows outside the viewport never get laid out or shaped"),
            Message {
                reactions: vec![
                    Reaction { emoji: "🔥".into(), count: 4, me: true, user_ids: vec![1, 2, 4, 5] },
                    Reaction { emoji: "🚀".into(), count: 2, me: false, user_ids: vec![2, 5] },
                ],
                ..msg(1006, 5, 240, "the hover state on grouped messages is so clean")
            },
            Message {
                content: "here's the row builder, if anyone wants to poke at it:\n```rust\nlet row = row().key(msg.id).pl(16.0).pr(48.0)\n    .hover(theme.chat_hover)\n    .child(avatar(&author, 40.0))\n    .child(body(msg, &author));\n```".into(),
                reactions: vec![Reaction { emoji: "👀".into(), count: 3, me: false, user_ids: vec![1, 2, 5] }],
                ..msg(1007, 2, 210, "")
            },
            Message {
                attachments: vec![Attachment {
                    id: 1,
                    filename: "sidebar-dark.png".into(),
                    size: 184_320,
                    content_type: "image/png".into(),
                    url: String::new(),
                    width: 640,
                    height: 400,
                }],
                ..msg(1008, 2, 180, "")
            },
            msg(1009, 6, 90, "just joined — is there a channel for keyboard shortcuts?"),
            msg(1010, 3, 88, "ctrl+k is the quick switcher. ctrl+shift+m toggles mute. esc closes whatever you opened"),
            msg(1011, 1, 60, "pinned it in #welcome so it doesn't get lost"),
            msg(
                1012,
                5,
                50,
                "the roadmap leak is ||Phase 6 is only a rumour|| and ||it stays hidden until you click||",
            ),
        ];
        general.sort_by_key(|m| m.timestamp);
        self.messages.insert(211, general);
        self.messages.insert(
            216,
            vec![
                Message {
                    id: 3000,
                    channel_id: 216,
                    author_id: 3,
                    content: "Every shortcut I use daily, in one place.".into(),
                    timestamp: t - 7_200_000,
                    ..Message::default()
                },
                Message {
                    id: 3001,
                    channel_id: 216,
                    author_id: 1,
                    content: "ctrl+shift+m for mute is the one I keep forgetting.".into(),
                    timestamp: t - 3_600_000,
                    ..Message::default()
                },
            ],
        );
        self.messages.insert(
            217,
            vec![Message {
                id: 3100,
                channel_id: 217,
                author_id: 2,
                content: "The 12px gap looks bigger than the 8px one at that scale.".into(),
                timestamp: t - 5_400_000,
                ..Message::default()
            }],
        );
        self.messages.insert(
            201,
            vec![Message {
                id: 1100,
                channel_id: 201,
                author_id: 8,
                content: "Welcome to **Rune Lounge** — read the rules, pick a role, and say hi in #general.".into(),
                timestamp: t - 86_400_000,
                ..Message::default()
            }],
        );
        self.messages.insert(
            202,
            vec![Message {
                id: 1200,
                channel_id: 202,
                author_id: 1,
                content: "Rune build 0.1 is live on the `dev` channel. Voice is next on the list."
                    .into(),
                timestamp: t - 3_600_000,
                reactions: vec![Reaction {
                    emoji: "🎉".into(),
                    count: 7,
                    me: true,
                    user_ids: vec![1, 2, 3, 4, 5, 6, 7],
                }],
                ..Message::default()
            }],
        );
        for id in [212, 213, 214, 301, 401, 501] {
            self.messages.entry(id).or_default();
        }
        self.unread.insert(212, 3);
        self.unread.insert(213, 1);
        self.unread.insert(300, 2);

        let dm = |id: Id, user: Id, name: &str| DmChannel {
            channel: Channel {
                id,
                guild_id: None,
                name: name.into(),
                kind: ChannelKind::Dm,
                ..Channel::default()
            },
            recipients: vec![self.users[&user].clone()],
        };
        self.dms = vec![dm(900, 2, "Maya Chen"), dm(901, 3, "Leo Park")];
        self.messages.insert(
            900,
            vec![
                Message {
                    id: 2000,
                    channel_id: 900,
                    author_id: 2,
                    content: "the app icon looks great at 16px".into(),
                    timestamp: t - 7_200_000,
                    ..Message::default()
                },
                Message {
                    id: 2001,
                    channel_id: 900,
                    author_id: 1,
                    content: "I redrew it as a vector path so it stays crisp".into(),
                    timestamp: t - 7_100_000,
                    ..Message::default()
                },
            ],
        );
        self.messages.insert(
            901,
            vec![Message {
                id: 2100,
                channel_id: 901,
                author_id: 3,
                content: "send me the crash log when you get a sec".into(),
                timestamp: t - 86_400_000,
                ..Message::default()
            }],
        );

        self.relationships = vec![
            Relationship {
                kind: RelationshipKind::Friend,
                user: self.users[&2].clone(),
                since: t,
            },
            Relationship {
                kind: RelationshipKind::Friend,
                user: self.users[&3].clone(),
                since: t,
            },
            Relationship {
                kind: RelationshipKind::Friend,
                user: self.users[&5].clone(),
                since: t,
            },
            Relationship {
                kind: RelationshipKind::Incoming,
                user: self.users[&4].clone(),
                since: t,
            },
            Relationship {
                kind: RelationshipKind::Outgoing,
                user: self.users[&6].clone(),
                since: t,
            },
        ];

        self.active_guild = 100;
        self.active_channel = 211;
        self.active_dm = 900;
        self.last_guild_channel.insert(100, 211);

        // Hidden benchmark hook: populate one channel with N rows so Rune can
        // report frame cost without a live server.
        if let Ok(raw) = std::env::var("RUNECORD_BENCH_MESSAGES")
            && let Ok(count) = raw.parse::<usize>()
        {
            self.messages.insert(211, bench_messages(count, t));
        }
        if let Ok(raw) = std::env::var("RUNECORD_BENCH_MEMBERS")
            && let Ok(count) = raw.parse::<usize>()
        {
            let mut list = Vec::with_capacity(count);
            for i in 0..count {
                let id = 20_000 + i as Id;
                let user = User {
                    id,
                    username: format!("bench{id}"),
                    display_name: format!("Bench {id}"),
                    avatar_color: (i % 10) as u32,
                    status: if i % 5 == 0 {
                        Presence::Offline
                    } else {
                        Presence::Online
                    },
                    created_at: t,
                    ..User::default()
                };
                self.presence.insert(id, user.status);
                self.users.insert(id, user.clone());
                list.push(Member {
                    guild_id: 100,
                    user,
                    nickname: String::new(),
                    roles: Vec::new(),
                    joined_at: t + i as i64,
                    timeout_until: 0,
                });
            }
            self.members.insert(100, list);
            if let Some(state) = self.member_state.get_mut(&100) {
                state.total = count as u32;
                state.next_offset = None;
            }
        }
    }
}
