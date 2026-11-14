//! Microbenchmarks for the Runecord client cache.
//!
//! Run with:
//!   cargo run --release --manifest-path tools/cache-bench/Cargo.toml
//!
//! The snapshot is synthetic: 4 guilds, 200 members, 40 channels and 2,880
//! retained messages (12 channels x 240). The real 13 KB cache is measured
//! separately when a profile cache exists.

use runecord_client::cache;
use runecord_protocol::{
    Attachment, Bootstrap, Channel, ChannelKind, GuildDetail, GuildSummary, Member, Message,
    Presence, User,
};
use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const GUILDS: usize = 4;
const MEMBERS: usize = 50;
const CHANNELS: usize = 10;
const MESSAGES: usize = 240;
const IMAGE_CHANNELS: usize = 50;

fn user(id: i64) -> User {
    User {
        id,
        username: format!("user{id}"),
        display_name: format!("User {id}"),
        avatar_color: (id % 10) as u32,
        bio: "benchmark account".into(),
        status: Presence::Online,
        created_at: 1,
        ..User::default()
    }
}

fn channel(id: i64, guild: i64) -> Channel {
    Channel {
        id,
        guild_id: Some(guild),
        name: format!("channel-{id}"),
        kind: ChannelKind::Text,
        position: 0,
        topic: "benchmark channel".into(),
        my_permissions: u64::MAX / 2,
        ..Channel::default()
    }
}

fn message(id: i64, channel_id: i64, image: bool) -> Message {
    Message {
        id,
        channel_id,
        author_id: (id % 50) + 1,
        content: format!(
            "message {id} with a typical amount of text for layout and JSON serialization"
        ),
        timestamp: id,
        attachments: if image {
            vec![Attachment {
                id,
                filename: format!("image-{id}.png"),
                size: 120_000,
                content_type: "image/png".into(),
                url: format!("/cdn/{id}.png"),
                width: 640,
                height: 400,
            }]
        } else {
            Vec::new()
        },
        ..Message::default()
    }
}

fn snapshot() -> cache::Snapshot {
    let mut guilds = Vec::new();
    let mut details = Vec::new();
    let mut messages = Vec::new();
    let mut next_user = 1i64;
    for g in 0..GUILDS {
        let guild_id = 100 + g as i64;
        for _ in 0..MEMBERS {
            next_user += 1;
        }
        let mut channels = Vec::new();
        for c in 0..CHANNELS {
            let channel_id = (guild_id * 1000) + c as i64;
            channels.push(channel(channel_id, guild_id));
            if messages.len() < 12 {
                messages.push(cache::CachedChannel {
                    channel_id,
                    messages: (0..MESSAGES)
                        .map(|m| message((channel_id * 10_000) + m as i64, channel_id, false))
                        .collect(),
                });
            }
        }
        let summary = GuildSummary {
            id: guild_id,
            name: format!("Guild {guild_id}"),
            icon_color: 0x5865f2,
            owner_id: 1,
            description: "benchmark guild".into(),
            member_count: MEMBERS as u32,
            tag: "BENCH".into(),
            permissions: u64::MAX / 2,
            unread_count: 0,
            mention_count: 0,
            created_at: 1,
            ..GuildSummary::default()
        };
        if details.len() < 6 {
            details.push(GuildDetail {
                guild: summary.clone(),
                channels: channels.clone(),
                roles: Vec::new(),
                overwrites: Vec::new(),
                permissions: u64::MAX / 2,
            });
        }
        guilds.push(summary);
    }
    cache::Snapshot {
        version: cache::CACHE_VERSION,
        saved_at: runecord_protocol::now_ms(),
        bootstrap: Bootstrap {
            user: user(1),
            guilds,
            dms: Vec::new(),
            relationships: Vec::new(),
            reads: Vec::new(),
            users: (1..=next_user).map(user).collect(),
            presences: (1..=next_user).map(|id| (id, Presence::Online)).collect(),
            unread: Default::default(),
            emojis: Vec::new(),
            voice_states: Vec::new(),
            ..Bootstrap::default()
        },
        details,
        messages,
        active_guild: 100,
        active_channel: 100_000,
        active_dm: 0,
        ..Default::default()
    }
}

fn mean_ms(iterations: usize, mut run: impl FnMut()) -> f64 {
    let started = Instant::now();
    for _ in 0..iterations {
        run();
    }
    started.elapsed().as_secs_f64() * 1000.0 / iterations as f64
}

fn scan_all(messages: &HashMap<i64, Vec<Message>>) -> usize {
    let mut found = 0;
    for message in messages.values().flatten() {
        for attachment in &message.attachments {
            if attachment.content_type.starts_with("image/") && !attachment.url.is_empty() {
                found += 1;
            }
        }
    }
    found
}

fn scan_channel(messages: &HashMap<i64, Vec<Message>>, channel: i64) -> usize {
    let mut found = 0;
    if let Some(list) = messages.get(&channel) {
        for message in list {
            for attachment in &message.attachments {
                if attachment.content_type.starts_with("image/") && !attachment.url.is_empty() {
                    found += 1;
                }
            }
        }
    }
    found
}

fn main() {
    let snapshot = snapshot();
    let encoded = serde_json::to_vec(&snapshot).expect("serialize snapshot");
    println!("snapshot_bytes={}", encoded.len());
    println!(
        "serialize_ms={:.3}",
        mean_ms(200, || {
            black_box(serde_json::to_vec(black_box(&snapshot)).unwrap());
        })
    );
    println!(
        "deserialize_ms={:.3}",
        mean_ms(200, || {
            black_box(serde_json::from_slice::<cache::Snapshot>(black_box(&encoded)).unwrap());
        })
    );

    let profile = "bench";
    cache::clear(profile);
    cache::save(profile, snapshot.clone());
    std::thread::sleep(std::time::Duration::from_millis(1_500));
    println!(
        "cache_load_ms={:.3}",
        mean_ms(100, || {
            black_box(cache::load(profile).unwrap());
        })
    );

    // Old client: every cached message across every channel, on every event.
    let mut all_messages: HashMap<i64, Vec<Message>> = HashMap::new();
    for c in 0..IMAGE_CHANNELS {
        let channel_id = 500_000 + c as i64;
        all_messages.insert(
            channel_id,
            (0..MESSAGES)
                .map(|m| message((channel_id * 10_000) + m as i64, channel_id, true))
                .collect(),
        );
    }
    let total_messages: usize = all_messages.values().map(Vec::len).sum();
    println!("scan_messages_total={total_messages}");
    println!(
        "scan_all_us={:.2}",
        mean_ms(1_000, || {
            black_box(scan_all(black_box(&all_messages)));
        }) * 1_000.0
    );
    println!(
        "scan_one_channel_us={:.2}",
        mean_ms(1_000, || {
            black_box(scan_channel(black_box(&all_messages), 500_000));
        }) * 1_000.0
    );

    // Cost of enforcing the 240-row retention window per incoming message.
    let mut list: Vec<Message> = (0..MESSAGES).map(|m| message(m as i64, 1, false)).collect();
    let mut next = 1_000_000i64;
    println!(
        "trim_push_us={:.3}",
        mean_ms(100_000, || {
            next += 1;
            list.push(message(next, 1, false));
            if list.len() > MESSAGES {
                let drop = list.len() - MESSAGES;
                list.drain(..drop);
            }
        }) * 1_000.0
    );

    // Old message_list() cloned the entire channel every frame.
    for n in [50usize, 240, 1_000, 5_000] {
        let rows: Vec<Message> = (0..n).map(|m| message(m as i64, 1, false)).collect();
        println!(
            "clone_messages_{n}_ms={:.3}",
            mean_ms(2_000, || {
                black_box(black_box(&rows).clone());
            })
        );
    }

    // Old member_list() cloned the entire guild member list every frame.
    let members: Vec<Member> = (1..=500)
        .map(|id| Member {
            guild_id: 100,
            user: user(id),
            nickname: String::new(),
            roles: vec![900],
            joined_at: 1,
            ..Member::default()
        })
        .collect();
    println!(
        "clone_members_500_ms={:.3}",
        mean_ms(2_000, || {
            black_box(black_box(&members).clone());
        })
    );

    cache::clear(profile);

    let real_path = std::env::var("APPDATA")
        .map(std::path::PathBuf::from)
        .map(|base| base.join("Runecord").join("cache-1.json"));
    if let Ok(path) = real_path
        && let Ok(real) = std::fs::read(path)
    {
        println!("real_cache_bytes={}", real.len());
        println!(
            "real_cache_load_us={:.1}",
            mean_ms(1_000, || {
                black_box(cache::load("1"));
            }) * 1_000.0
        );
    }
}
