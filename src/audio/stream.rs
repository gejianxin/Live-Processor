use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamProtocol {
    HLS,
    RTMP,
    MpegDash,
}

impl StreamProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamProtocol::HLS => "HLS",
            StreamProtocol::RTMP => "RTMP",
            StreamProtocol::MpegDash => "MPEG-DASH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationStatus {
    Standby,
    Connecting,
    Streaming,
    Error,
}

#[derive(Debug, Clone)]
pub struct StreamDestination {
    pub id: String,
    pub name: String,
    pub protocol: StreamProtocol,
    pub enabled: bool,
    pub url: String,
    pub stream_key: String,
    pub bitrate_kbps: u32,
    pub status: DestinationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerRegion {
    EuropeParis,
    EuropeFrankfurt,
    NorthAmericaEast,
    NorthAmericaWest,
    AsiaTokyo,
    AsiaSingapore,
    AsiaHongKong,
}

impl ServerRegion {
    pub fn all() -> &'static [ServerRegion] {
        &[
            ServerRegion::EuropeParis,
            ServerRegion::EuropeFrankfurt,
            ServerRegion::NorthAmericaEast,
            ServerRegion::NorthAmericaWest,
            ServerRegion::AsiaTokyo,
            ServerRegion::AsiaSingapore,
            ServerRegion::AsiaHongKong,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ServerRegion::EuropeParis => "Europe (Paris)",
            ServerRegion::EuropeFrankfurt => "Europe (Frankfurt)",
            ServerRegion::NorthAmericaEast => "North America (East)",
            ServerRegion::NorthAmericaWest => "North America (West)",
            ServerRegion::AsiaTokyo => "Asia (Tokyo)",
            ServerRegion::AsiaSingapore => "Asia (Singapore)",
            ServerRegion::AsiaHongKong => "Asia (Hong Kong)",
        }
    }

    pub fn latency_ms(&self) -> u32 {
        match self {
            ServerRegion::EuropeParis => 18,
            ServerRegion::EuropeFrankfurt => 22,
            ServerRegion::NorthAmericaEast => 45,
            ServerRegion::NorthAmericaWest => 68,
            ServerRegion::AsiaTokyo => 32,
            ServerRegion::AsiaSingapore => 28,
            ServerRegion::AsiaHongKong => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerConnectionState {
    Offline,
    Connecting,
    Online,
    Streaming,
}

pub struct LiveStreamHub {
    pub selected_device: String,
    pub server_region: ServerRegion,
    pub server_state: ServerConnectionState,
    pub destinations: Vec<StreamDestination>,

    // Real-time broadcast telemetry
    pub is_broadcasting: bool,
    pub is_recording: bool,
    pub broadcast_start_time: Option<Instant>,
    pub elapsed_seconds: u32,
    pub mic_muted: bool,

    // Audio Tri-Stage Knobs & Levels
    pub input_pan: f32,       // -1.0 .. 1.0 (0.00 center)
    pub input_gain_db: f32,   // -60.0 .. +12.0 dB
    pub monitor_pan: f32,     // -1.0 .. 1.0 (0.00 center)
    pub monitor_gain_db: f32, // -60.0 .. +12.0 dB
    pub stream_gain_db: f32,  // -60.0 .. +12.0 dB

    // Meters (Peak linear 0.0 .. 1.2)
    pub input_meter_l: f32,
    pub input_meter_r: f32,
    pub monitor_meter_l: f32,
    pub monitor_meter_r: f32,
    pub stream_meter_l: f32,
    pub stream_meter_r: f32,

    // Network / Spectrum oscilloscope bars (32 bars)
    pub spectrum_bars: [f32; 32],
}

impl Default for LiveStreamHub {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveStreamHub {
    pub fn new() -> Self {
        let destinations = vec![
            StreamDestination {
                id: "bilibili".into(),
                name: "哔哩哔哩直播 (Bilibili)".into(),
                protocol: StreamProtocol::RTMP,
                enabled: true,
                url: "rtmp://live-push.bilivideo.com/live-bvc/".into(),
                stream_key: "live_bili_token_8892".into(),
                bitrate_kbps: 192,
                status: DestinationStatus::Standby,
            },
            StreamDestination {
                id: "douyin".into(),
                name: "抖音直播 (Douyin Live)".into(),
                protocol: StreamProtocol::RTMP,
                enabled: true,
                url: "rtmp://live-push.douyin.com/live/".into(),
                stream_key: "live_stream_douyin_6671".into(),
                bitrate_kbps: 192,
                status: DestinationStatus::Standby,
            },
            StreamDestination {
                id: "wechat".into(),
                name: "微信视频号 (WeChat Live)".into(),
                protocol: StreamProtocol::RTMP,
                enabled: false,
                url: "rtmp://channels.weixin.qq.com/live/".into(),
                stream_key: "wx_live_token_sec".into(),
                bitrate_kbps: 192,
                status: DestinationStatus::Standby,
            },
            StreamDestination {
                id: "kuaishou".into(),
                name: "快手直播 (Kuaishou)".into(),
                protocol: StreamProtocol::RTMP,
                enabled: false,
                url: "rtmp://live-push.kuaishou.com/live/".into(),
                stream_key: "ks_live_key_9921".into(),
                bitrate_kbps: 128,
                status: DestinationStatus::Standby,
            },
            StreamDestination {
                id: "huya".into(),
                name: "虎牙直播 (Huya Live)".into(),
                protocol: StreamProtocol::RTMP,
                enabled: false,
                url: "rtmp://ws.live.huya.com/live/".into(),
                stream_key: "huya_stream_token".into(),
                bitrate_kbps: 128,
                status: DestinationStatus::Standby,
            },
            StreamDestination {
                id: "netease".into(),
                name: "网易云播客 (NetEase Live)".into(),
                protocol: StreamProtocol::RTMP,
                enabled: false,
                url: "rtmp://live.music.163.com/live/".into(),
                stream_key: "cloudmusic_live_token".into(),
                bitrate_kbps: 320,
                status: DestinationStatus::Standby,
            },
            StreamDestination {
                id: "hls".into(),
                name: "HLS CDN Stream".into(),
                protocol: StreamProtocol::HLS,
                enabled: true,
                url: "https://live-cdn.dialogaudio.net/hls/live.m3u8".into(),
                stream_key: "hls_live_sec_token_99".into(),
                bitrate_kbps: 320,
                status: DestinationStatus::Standby,
            },
            StreamDestination {
                id: "youtube".into(),
                name: "YouTube Live".into(),
                protocol: StreamProtocol::RTMP,
                enabled: false,
                url: "rtmp://a.rtmp.youtube.com/live2".into(),
                stream_key: "xxxx-xxxx-xxxx-xxxx".into(),
                bitrate_kbps: 128,
                status: DestinationStatus::Standby,
            },
            StreamDestination {
                id: "custom".into(),
                name: "自定义 RTMP (Custom)".into(),
                protocol: StreamProtocol::RTMP,
                enabled: false,
                url: "rtmp://broadcast.live.custom.net/live/".into(),
                stream_key: "".into(),
                bitrate_kbps: 128,
                status: DestinationStatus::Standby,
            },
        ];

        Self {
            selected_device: "Signature 200 (USB Audio / ASIO)".into(),
            server_region: ServerRegion::EuropeParis,
            server_state: ServerConnectionState::Online,
            destinations,

            is_broadcasting: true,
            is_recording: false,
            broadcast_start_time: Some(Instant::now()),
            elapsed_seconds: 309, // 00:05:09 from genuine reference screenshot
            mic_muted: false,

            input_pan: 0.0,
            input_gain_db: 0.0,
            monitor_pan: 0.0,
            monitor_gain_db: -11.2, // Matches authentic reference knob (-11.2 dB)
            stream_gain_db: 0.0,

            input_meter_l: 0.72,
            input_meter_r: 0.68,
            monitor_meter_l: 0.45,
            monitor_meter_r: 0.42,
            stream_meter_l: 0.88,
            stream_meter_r: 0.85,

            spectrum_bars: [0.3; 32],
        }
    }

    pub fn toggle_server_connection(&mut self) {
        match self.server_state {
            ServerConnectionState::Offline => {
                self.server_state = ServerConnectionState::Online;
            }
            ServerConnectionState::Connecting => {
                self.server_state = ServerConnectionState::Offline;
            }
            ServerConnectionState::Online | ServerConnectionState::Streaming => {
                self.server_state = ServerConnectionState::Offline;
                self.is_broadcasting = false;
                for dest in &mut self.destinations {
                    dest.status = DestinationStatus::Standby;
                }
            }
        }
    }

    pub fn toggle_broadcast(&mut self) {
        if self.is_broadcasting {
            self.is_broadcasting = false;
            for dest in &mut self.destinations {
                if dest.enabled {
                    dest.status = DestinationStatus::Standby;
                }
            }
        } else {
            if self.server_state == ServerConnectionState::Offline {
                self.server_state = ServerConnectionState::Online;
            }
            self.is_broadcasting = true;
            if self.broadcast_start_time.is_none() {
                self.broadcast_start_time = Some(Instant::now());
            }
            for dest in &mut self.destinations {
                if dest.enabled {
                    dest.status = DestinationStatus::Streaming;
                }
            }
        }
    }

    pub fn toggle_record(&mut self) {
        self.is_recording = !self.is_recording;
    }

    pub fn toggle_mic_mute(&mut self) {
        self.mic_muted = !self.mic_muted;
    }

    pub fn update_frame(&mut self, time: f64) {
        if self.is_broadcasting {
            self.elapsed_seconds = (309.0 + (time * 1.0) % 36000.0) as u32;

            // Animate realistic dynamic broadcast VU levels
            let base_in = (time * 4.2).sin() * 0.18 + 0.65;
            let jiggle_l = ((time * 11.5).cos() * 0.07) as f32;
            let jiggle_r = ((time * 9.8).sin() * 0.08) as f32;

            if self.mic_muted {
                self.input_meter_l = 0.0;
                self.input_meter_r = 0.0;
            } else {
                let gain_factor = 10.0_f32.powf(self.input_gain_db / 20.0);
                self.input_meter_l = ((base_in as f32 + jiggle_l) * gain_factor).clamp(0.0, 1.15);
                self.input_meter_r = ((base_in as f32 + jiggle_r) * gain_factor).clamp(0.0, 1.15);
            }

            let mon_factor = 10.0_f32.powf(self.monitor_gain_db / 20.0);
            self.monitor_meter_l = (self.input_meter_l * mon_factor * 1.5).clamp(0.0, 1.1);
            self.monitor_meter_r = (self.input_meter_r * mon_factor * 1.5).clamp(0.0, 1.1);

            let stream_factor = 10.0_f32.powf(self.stream_gain_db / 20.0);
            self.stream_meter_l = ((self.input_meter_l * 1.15 + 0.1) * stream_factor).clamp(0.0, 1.25);
            self.stream_meter_r = ((self.input_meter_r * 1.12 + 0.12) * stream_factor).clamp(0.0, 1.25);

            // Animate 32 spectrum oscilloscope waveform bars
            for i in 0..32 {
                let phase = time * 8.0 + (i as f64) * 0.35;
                let height = (phase.sin() * 0.45 + 0.55) as f32;
                // Add spikes at specific frequencies (e.g. index 8 and 22 as in genuine reference)
                let spike = if i == 7 || i == 21 { 0.4 } else { 0.0 };
                self.spectrum_bars[i] = (height + spike).clamp(0.15, 1.0);
            }
        } else {
            self.input_meter_l *= 0.85;
            self.input_meter_r *= 0.85;
            self.monitor_meter_l *= 0.85;
            self.monitor_meter_r *= 0.85;
            self.stream_meter_l *= 0.85;
            self.stream_meter_r *= 0.85;
            for bar in &mut self.spectrum_bars {
                *bar = (*bar * 0.9).max(0.08);
            }
        }
    }

    pub fn formatted_timer(&self) -> String {
        let hrs = self.elapsed_seconds / 3600;
        let mins = (self.elapsed_seconds % 3600) / 60;
        let secs = self.elapsed_seconds % 60;
        format!("{:02}:{:02}:{:02}", hrs, mins, secs)
    }

    pub fn active_destination_count(&self) -> usize {
        self.destinations.iter().filter(|d| d.enabled).count()
    }
}
