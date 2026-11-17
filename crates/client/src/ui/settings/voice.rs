use super::*;

pub(crate) fn volume_track(value: f32, set: fn(&mut App, f32), p: Palette) -> El {
    let fill = value.clamp(0.0, 1.0);
    el().w_full()
        .h(20.0)
        .role("slider")
        .label(format!("{:.0}%", fill * 100.0))
        .on_drag(move |s: &mut App, drag, _| {
            if drag.phase != rune::Phase::Up && drag.w > 0.0 {
                set(s, (drag.x / drag.w).clamp(0.0, 1.0));
            }
        })
        .child(
            row()
                .w_full()
                .h(8.0)
                .radius(4.0)
                .clip_rounded(4.0)
                .fill(p.rail)
                .child(el().h(8.0).grow(fill.max(0.001)).fill(p.accent))
                .child(el().h(8.0).grow((1.0 - fill).max(0.001))),
        )
}

pub(crate) fn voice_device_row(
    label: &'static str,
    value: &str,
    kind: &'static str,
    p: Palette,
) -> El {
    row()
        .w_full()
        .gap(10.0)
        .center_y()
        .child(text(label).font(theme::TINY).color(p.text_muted).w(140.0))
        .child(
            el().key(format!("voice-{kind}-device"))
                .h(38.0)
                .grow(1.0)
                .px(10.0)
                .radius(4.0)
                .fill(p.input)
                .center_y()
                .role("button")
                .label(format!("{label}: {value}"))
                .hover(p.card_hover)
                .on_press(move |s: &mut App| {
                    s.refresh_voice_devices();
                    s.modal = Modal::VoiceDevice {
                        kind: kind.to_string(),
                    };
                })
                .child(
                    text(if value.is_empty() { "Default" } else { value })
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                )
                .child(icon(&icons::CHEVRON_DOWN, 14.0).color(p.text_muted)),
        )
}

pub(crate) fn voice_tab(s: &App, p: Palette) -> El {
    let level = crate::voice::mic_level();
    let mic_running = crate::voice::mic_testing();
    let camera_running = crate::voice::camera_testing();
    let camera_frame = crate::voice::camera_test_frame();
    let auto = s.voice_auto_sensitivity;

    let section = |label: &'static str, p: Palette| {
        text(label).font(theme::SECTION).color(p.text_muted).pt(6.0)
    };

    col()
        .key("voice-settings")
        .w_full()
        .gap(14.0)
        .child(settings_title(
            "Voice & Video",
            "Choose devices, test your microphone and preview your camera.",
            p,
        ))
        .child(voice_device_row(
            "Input Device",
            &s.voice_input_device,
            "input",
            p,
        ))
        .child(voice_device_row(
            "Output Device",
            &s.voice_output_device,
            "output",
            p,
        ))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            row()
                .w_full()
                .gap(16.0)
                .child(
                    col()
                        .grow(1.0)
                        .gap(6.0)
                        .child(text("INPUT VOLUME").font(theme::TINY).color(p.text_muted))
                        .child(volume_track(
                            s.voice_input_volume,
                            |s: &mut App, value| s.set_voice_volume(true, value),
                            p,
                        )),
                )
                .child(
                    col()
                        .grow(1.0)
                        .gap(6.0)
                        .child(text("OUTPUT VOLUME").font(theme::TINY).color(p.text_muted))
                        .child(volume_track(
                            s.voice_output_volume,
                            |s: &mut App, value| s.set_voice_volume(false, value),
                            p,
                        )),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(section("MIC TEST", p))
        .child(
            row()
                .w_full()
                .gap(12.0)
                .center_y()
                .child(
                    el().key("mic-test-toggle")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(if mic_running {
                            "Stop mic test"
                        } else {
                            "Let's Check"
                        })
                        .fill(if mic_running { p.rail } else { p.accent })
                        .hover(if mic_running {
                            p.rail_hover
                        } else {
                            p.accent_hover
                        })
                        .on_press(|s: &mut App| s.toggle_mic_test())
                        .child(
                            text(if mic_running {
                                "Stop Test"
                            } else {
                                "Let's Check"
                            })
                            .font(theme::SMALL_MED)
                            .color(Color::WHITE),
                        ),
                )
                .child(
                    el().grow(1.0)
                        .h(10.0)
                        .radius(5.0)
                        .clip_rounded(5.0)
                        .fill(p.rail)
                        .child(
                            row()
                                .h(10.0)
                                .w_full()
                                .child(el().h(10.0).grow(level.max(0.001)).fill(if level > 0.7 {
                                    p.yellow
                                } else {
                                    p.green
                                }))
                                .child(el().h(10.0).grow((1.0 - level).max(0.001))),
                        ),
                )
                .child(
                    text(if level > 0.02 {
                        "Hearing you"
                    } else {
                        "Say something"
                    })
                    .font(theme::TINY)
                    .color(p.text_faint),
                ),
        )
        .child(section("INPUT MODE", p))
        .child({
            let mode = |key: &'static str,
                        label: &'static str,
                        detail: &'static str,
                        value: &'static str,
                        p: Palette| {
                let selected = s.voice_mode == value;
                el().key(key)
                    .w_full()
                    .min_h(56.0)
                    .px(14.0)
                    .py(9.0)
                    .radius(6.0)
                    .gap(12.0)
                    .center_y()
                    .role("radio")
                    .label(label)
                    .checked(selected)
                    .fill(if selected { p.selected } else { p.card })
                    .border(1.0, if selected { p.accent } else { p.divider })
                    .hover(p.card_hover)
                    .motion(motion::SNAPPY)
                    .on_press(move |s: &mut App| s.set_voice_mode(value))
                    .child(
                        el().size(18.0, 18.0)
                            .radius(9.0)
                            .border(2.0, if selected { p.accent } else { p.text_faint })
                            .center()
                            .children(selected.then(|| {
                                el().size(8.0, 8.0).radius(4.0).fill(p.accent)
                            })),
                    )
                    .child(
                        col()
                            .grow(1.0)
                            .min_w(0.0)
                            .gap(3.0)
                            .child(
                                text(label)
                                    .font(theme::SMALL_MED)
                                    .color(p.text_bright),
                            )
                            .child(text(detail).wrap().font(theme::TINY).color(p.text_muted)),
                    )
            };
            col()
                .w_full()
                .gap(8.0)
                .child(mode(
                    "voice-mode-voice",
                    "Voice Activity",
                    "Transmit automatically when your microphone picks up speech.",
                    "voice",
                    p,
                ))
                .child(mode(
                    "voice-mode-ptt",
                    "Push to Talk",
                    "Transmit only while you hold your shortcut key.",
                    "ptt",
                    p,
                ))
                .children((s.voice_mode == "ptt").then(|| {
                    let label = crate::voice::push_to_talk_key_label();
                    col()
                        .w_full()
                        .gap(4.0)
                        .child(
                            row()
                                .w_full()
                                .h(30.0)
                                .px(12.0)
                                .gap(8.0)
                                .center_y()
                                .child(el().size(8.0, 8.0).radius(4.0).fill(
                                    if crate::voice::push_to_talk_held() {
                                        p.green
                                    } else {
                                        p.text_faint
                                    },
                                ))
                                .child(
                                    text(if crate::voice::push_to_talk_held() {
                                        format!("Transmitting - {label} held")
                                    } else {
                                        format!("Hold {label} to talk")
                                    })
                                    .font(theme::TINY)
                                    .color(p.text_muted),
                                ),
                        )
                        .child(
                            row()
                                .w_full()
                                .gap(8.0)
                                .center_y()
                                .child(
                                    el().key("voice-ptt-key")
                                        .h(30.0)
                                        .px(12.0)
                                        .radius(4.0)
                                        .center()
                                        .role("button")
                                        .label("Change Push-to-Talk key")
                                        .fill(if s.ptt_capture { p.accent } else { p.input })
                                        .hover(p.input_hover)
                                        .on_press(|s: &mut App| s.ptt_capture = !s.ptt_capture)
                                        .child(
                                            text(if s.ptt_capture {
                                                "Press a key...".to_string()
                                            } else {
                                                format!("Shortcut: {label}")
                                            })
                                            .font(theme::TINY)
                                            .color(if s.ptt_capture {
                                                Color::WHITE
                                            } else {
                                                p.text_bright
                                            }),
                                        ),
                                )
                                .child(
                                    el().key("voice-ptt-reset")
                                        .h(30.0)
                                        .px(10.0)
                                        .radius(4.0)
                                        .center()
                                        .role("button")
                                        .label("Reset Push-to-Talk key to F8")
                                        .hover(p.card_hover)
                                        .on_press(|s: &mut App| s.set_push_to_talk_key(0x77))
                                        .child(
                                            text("Reset to F8")
                                                .font(theme::TINY)
                                                .color(p.text_muted),
                                        ),
                                ),
                        )
                }))
                .child(
                    row()
                        .key("voice-noise-suppression")
                        .w_full()
                        .min_h(56.0)
                        .px(14.0)
                        .py(9.0)
                        .radius(6.0)
                        .gap(12.0)
                        .center_y()
                        .role("checkbox")
                        .label("Noise Suppression")
                        .checked(s.voice_noise_suppression)
                        .fill(p.card)
                        .border(1.0, p.divider)
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| {
                            let next = !s.voice_noise_suppression;
                            s.set_noise_suppression(next)
                        })
                        .child(
                            el().size(20.0, 20.0)
                                .radius(5.0)
                                .border(2.0, if s.voice_noise_suppression {
                                    p.accent
                                } else {
                                    p.text_faint
                                })
                                .fill(if s.voice_noise_suppression {
                                    p.accent
                                } else {
                                    p.input
                                })
                                .center()
                                .children(s.voice_noise_suppression.then(|| {
                                    icon(&icons::CHECK, 13.0).color(Color::WHITE)
                                })),
                        )
                        .child(
                            col()
                                .grow(1.0)
                                .min_w(0.0)
                                .gap(3.0)
                                .child(
                                    text("Noise Suppression")
                                        .font(theme::SMALL_MED)
                                        .color(p.text_bright),
                                )
                                .child(
                                    text("Tracks the room's noise floor and pulls steady hum and fan noise out of your mic.")
                                        .wrap()
                                        .font(theme::TINY)
                                        .color(p.text_muted),
                                ),
                        ),
                )
        })
        .child(section("INPUT SENSITIVITY", p))
        .child(
            row()
                .key("voice-auto-sensitivity")
                .w_full()
                .min_h(40.0)
                .px(10.0)
                .radius(4.0)
                .center_y()
                .role("switch")
                .label("Automatically determine input sensitivity")
                .checked(auto)
                .hover(p.card_hover)
                .on_press(|s: &mut App| {
                    let next = !s.voice_auto_sensitivity;
                    s.set_voice_auto_sensitivity(next);
                })
                .child(
                    text("Automatically determine input sensitivity")
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(
                    el().size(40.0, 22.0)
                        .radius(11.0)
                        .fill(if auto { p.green } else { p.input })
                        .child(
                            el().size(18.0, 18.0)
                                .radius(9.0)
                                .fill(Color::WHITE)
                                .ml(if auto { 19.0 } else { 2.0 })
                                .mt(2.0),
                        ),
                ),
        )
        .child(if auto {
            el().w_full().h(20.0).child(
                row()
                    .w_full()
                    .h(8.0)
                    .radius(4.0)
                    .clip_rounded(4.0)
                    .fill(p.rail)
                    .child(el().h(8.0).grow(0.35).fill(p.yellow))
                    .child(el().h(8.0).grow(0.65).fill(p.green)),
            )
        } else {
            volume_track(
                (s.voice_sensitivity / 0.5).clamp(0.0, 1.0),
                |s: &mut App, value| s.set_voice_sensitivity(value * 0.5),
                p,
            )
        })
        .child(
            text(if auto {
                "Automatic sensitivity follows the noise floor."
            } else {
                "Drag the bar to set the cutoff for transmitting your microphone."
            })
            .font(theme::TINY)
            .color(p.text_faint),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(section("CAMERA", p))
        .child(voice_device_row(
            "Camera",
            &s.voice_camera_device,
            "camera",
            p,
        ))
        .child(
            row()
                .w_full()
                .gap(14.0)
                .child(
                    el().w(360.0)
                        .h(203.0)
                        .radius(8.0)
                        .clip_rounded(8.0)
                        .fill(p.rail)
                        .center()
                        .child(if let Some(pixels) = camera_frame {
                            el().w_full()
                                .h(203.0)
                                .pixels_fit(&pixels, rune::SurfaceFit::Fill)
                        } else {
                            col()
                                .center()
                                .gap(6.0)
                                .child(icon(&icons::VIDEO, 36.0).color(p.text_faint))
                                .child(
                                    text("Camera preview")
                                        .font(theme::SMALL)
                                        .color(p.text_muted),
                                )
                        }),
                )
                .child(
                    col()
                        .gap(8.0)
                        .child(
                            el().key("camera-test-toggle")
                                .h(34.0)
                                .px(14.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label(if camera_running {
                                    "Stop video test"
                                } else {
                                    "Test Video"
                                })
                                .fill(if camera_running { p.rail } else { p.accent })
                                .hover(if camera_running {
                                    p.rail_hover
                                } else {
                                    p.accent_hover
                                })
                                .on_press(|s: &mut App| s.toggle_camera_test())
                                .child(
                                    text(if camera_running {
                                        "Stop Video"
                                    } else {
                                        "Test Video"
                                    })
                                    .font(theme::SMALL_MED)
                                    .color(Color::WHITE),
                                ),
                        )
                        .child(
                            text("Live DirectShow preview at 640 px wide.")
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                ),
        )
}

pub(crate) fn voice_device_sheet(s: &App, p: Palette, kind: &str) -> El {
    let (devices, title) = match kind {
        "input" => (&s.audio_devices, "Input Device"),
        "output" => (&s.output_devices, "Output Device"),
        _ => (&s.video_devices, "Camera"),
    };
    let current = match kind {
        "input" => &s.voice_input_device,
        "output" => &s.voice_output_device,
        _ => &s.voice_camera_device,
    };
    let kind_owned = kind.to_string();
    let mut list = col().w_full().gap(2.0);
    let mut options = vec![String::new()];
    options.extend(devices.iter().cloned());
    for device in options {
        let selected = current == &device;
        let label = if device.is_empty() {
            "Default".to_string()
        } else {
            device.clone()
        };
        let kind = kind_owned.clone();
        list = list.child(
            el().key(format!(
                "voice-device-{}",
                if device.is_empty() {
                    "default"
                } else {
                    &device
                }
            ))
            .w_full()
            .min_h(38.0)
            .px(10.0)
            .radius(4.0)
            .center_y()
            .role("radio")
            .label(label.clone())
            .checked(selected)
            .fill(if selected {
                p.selected
            } else {
                Color::TRANSPARENT
            })
            .hover(p.card_hover)
            .on_press(move |s: &mut App| s.select_voice_device(&kind, device.clone()))
            .child(
                text(label)
                    .font(theme::SMALL)
                    .color(if selected {
                        p.text_bright
                    } else {
                        p.text_muted
                    })
                    .truncate(),
            ),
        );
    }
    col()
        .key("voice-devices")
        .w(460.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| {
            s.modal = Modal::Settings(SettingsTab::Voice)
        })
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text(title)
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            col()
                .w_full()
                .max_h(360.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
}
