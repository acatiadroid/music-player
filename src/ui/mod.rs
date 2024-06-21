use std::path::Path;
use std::sync::mpsc;

use crate::core::json;
use crate::core::youtube;
use components::sidebar;
use components::theme;
use components::toast;

use iced::advanced::graphics::futures::event;
use iced::event::Event as IcedEvent;
use iced::keyboard;
use iced::keyboard::key;
use iced::widget;
use iced::widget::{column, row};
use iced::{Subscription, Task, Theme};

use self::components::theme::get_theme_from_settings;
use crate::core::playback::{start_receiver, AudioEvent};

mod components;
mod download;
mod ffmpeg;
mod playlist;
mod results;
mod settings;
mod track_list;

pub struct Pages {
    pub current_page: Page,

    nav: components::nav::State,
    sidebar: components::sidebar::State,
    controls: components::control_bar::State,

    track_list: track_list::State,
    settings: settings::State,
    download: download::State,
    results: results::State,
    ffmpeg: ffmpeg::State,
    playlist: playlist::State,

    playback_sender: mpsc::Sender<AudioEvent>,

    toasts: Vec<toast::Toast>,
    theme: Theme,
    track_list_loaded: bool,
}

#[derive(Default)]
pub enum Page {
    #[default]
    TrackList,
    Settings,
    Download,
    Results,
    FFmpeg,
    Playlist,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    NavAction(components::nav::Event),
    SidebarAction(components::sidebar::Event),
    ControlsAction(components::control_bar::Event),

    TrackListAction(track_list::Event),
    SettingsAction(settings::Event),
    DownloadAction(download::Event),
    ResultsAction(results::Event),
    FFmpegAction(ffmpeg::Event),
    PlaylistAction(playlist::Event),

    CloseToast(usize),
    KeyboardEvent(IcedEvent),
}

impl Pages {
    pub fn new() -> Self {
        let (sender, reciever) = mpsc::channel();

        start_receiver(reciever);

        let theme_value = json::get_theme().expect("Dark");
        let matched = get_theme_from_settings(theme_value);

        let current_page: Page;

        let ffmpeg_path = json::get_ffmpeg_path();

        if let Ok(path) = &ffmpeg_path {
            if path.is_empty() || !Path::new(path).exists() {
                if cfg!(unix) {
                    current_page = Page::TrackList;
                } else {
                    current_page = Page::FFmpeg;
                }
            } else {
                current_page = Page::TrackList;
            }
        } else {
            current_page = Page::FFmpeg;
        }

        Self {
            current_page,

            nav: Default::default(),
            sidebar: Default::default(),
            controls: Default::default(),

            track_list: Default::default(),
            download: Default::default(),
            settings: Default::default(),
            results: Default::default(),
            ffmpeg: Default::default(),
            playlist: Default::default(),

            playback_sender: sender,
            toasts: vec![],
            theme: matched,
            track_list_loaded: false,
        }
    }
    
    pub fn update(&mut self, message: UiEvent) -> Task<UiEvent> {
        match message {
            UiEvent::KeyboardEvent(event) => match event {
                IcedEvent::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Tab),
                    modifiers,
                    ..
                }) => {
                    if modifiers.shift() {
                        widget::focus_previous()
                    } else {
                        widget::focus_next()
                    }
                }
                IcedEvent::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Space),
                    ..
                }) => {
                    self.playback_sender
                        .send(AudioEvent::PauseToggle)
                        .expect("Failed to send pause command");

                    self.controls
                        .update(components::control_bar::Event::PauseToggleAction)
                        .map(UiEvent::ControlsAction)
                }
                _ => Task::none(),
            },
            UiEvent::NavAction(event) => {
                match event {
                    components::nav::Event::CollapseSidebar => {
                        return Task::batch(vec![
                            self.sidebar
                                .update(sidebar::Event::CollapseToggle)
                                .map(UiEvent::SidebarAction),
                            self.nav.update(event.clone()).map(UiEvent::NavAction),
                        ]);
                    }
                    _ => (),
                }

                self.nav.update(event.clone()).map(UiEvent::NavAction)
            }

            UiEvent::PlaylistAction(event) => {
                let playlist_command = self
                    .playlist
                    .update(event.clone())
                    .map(UiEvent::PlaylistAction);

                match event {
                    playlist::Event::CreatePlaylist => Task::batch(vec![
                        playlist_command,
                        self.sidebar
                            .update(sidebar::Event::UpdatePlaylists)
                            .map(UiEvent::SidebarAction),
                    ]),
                    playlist::Event::PlayTrack(video_id, display_name, duration, handle) => {
                        self.playback_sender
                            .send(AudioEvent::Queue(video_id.clone().to_string(), true))
                            .expect("Failed to send play command");

                        Task::batch(vec![
                            self.controls
                                .update(components::control_bar::Event::InitiatePlay(
                                    display_name.to_string(),
                                    duration,
                                    handle.clone(),
                                ))
                                .map(UiEvent::ControlsAction),
                            playlist_command,
                        ])
                    }
                    _ => playlist_command,
                }
            }
            UiEvent::CloseToast(index) => {
                self.toasts.remove(index);

                Task::none()
            }

            UiEvent::FFmpegAction(event) => {
                match event {
                    ffmpeg::Event::Continue => self.current_page = Page::TrackList,
                    _ => (),
                };

                self.ffmpeg.update(event).map(UiEvent::FFmpegAction)
            }

            UiEvent::DownloadAction(event) => {
                let download_command = self
                    .download
                    .update(event.clone())
                    .map(UiEvent::DownloadAction);
                match event {
                    download::Event::DownloadQueryReceived(data) => {
                        self.current_page = Page::Results;

                        let data = match data {
                            Ok(data) => data,
                            Err(error) => {
                                match error {
                                    youtube::StatusError::NetworkError => {
                                        self.toasts.push(toast::Toast {
                                            title: "Network Error".into(),
                                            body: "Failed to fetch search results".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::UnknownError => {
                                        self.toasts.push(toast::Toast {
                                            title: "Unknown Error".into(),
                                            body: "An unknown error occurred".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::VideoNotFound => {
                                        self.toasts.push(toast::Toast {
                                            title: "Video Not Found".into(),
                                            body: "The video you are looking for was not found"
                                                .into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::FFmpegConversionError => {
                                        self.toasts.push(toast::Toast {
                                            title: "FFmpeg Error".into(),
                                            body: "Failed to convert video".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::CodecError => {
                                        self.toasts.push(toast::Toast {
                                            title: "Codec Error".into(),
                                            body: "Failed to decode video".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::VideoOptionError => {
                                        self.toasts.push(toast::Toast {
                                            title: "Video Option Error".into(),
                                            body: "Failed to set video options".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::DownloadError => {
                                        self.toasts.push(toast::Toast {
                                            title: "Download Error".into(),
                                            body: "Failed to download video".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::ThumbnailError => {
                                        self.toasts.push(toast::Toast {
                                            title: "Thumbnail Error".into(),
                                            body: "Failed to fetch thumbnail".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::VideoInfoError => {
                                        self.toasts.push(toast::Toast {
                                            title: "Video Info Error".into(),
                                            body: "Failed to fetch video info".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                    youtube::StatusError::WriteError => {
                                        self.toasts.push(toast::Toast {
                                            title: "Write Error".into(),
                                            body: "Failed to write video".into(),
                                            status: toast::Status::Danger,
                                        })
                                    }
                                }
                                return download_command;
                            }
                        };

                        Task::batch(vec![
                            self.results
                                .update(results::Event::PopulateResults(data))
                                .map(UiEvent::ResultsAction),
                            download_command,
                        ])
                    }
                    _ => download_command,
                }
            }
            UiEvent::SettingsAction(event) => {
                match event {
                    settings::Event::ThemeSelected(theme) => {
                        self.theme = theme::match_theme(Some(theme));
                    }
                }
                self.settings.update(event).map(UiEvent::SettingsAction)
            }
            UiEvent::TrackListAction(ref event) => {
                let track_list_command: Task<UiEvent>;

                if !self.track_list_loaded {
                    track_list_command = Task::batch(vec![
                        self.track_list
                            .update(track_list::Event::GetThumbnailHandles)
                            .map(UiEvent::TrackListAction),
                        self.track_list
                            .update(event.clone())
                            .map(UiEvent::TrackListAction),
                    ]);
                    self.track_list_loaded = true;
                } else {
                    track_list_command = self
                        .track_list
                        .update(event.clone())
                        .map(UiEvent::TrackListAction);
                }
                match event {
                    track_list::Event::PlayTrack(video_id, display_name, duration, handle) => {
                        self.playback_sender
                            .send(AudioEvent::Queue(video_id.clone().to_string(), true))
                            .expect("Failed to send play command");

                        Task::batch(vec![
                            self.controls
                                .update(components::control_bar::Event::InitiatePlay(
                                    display_name.to_string(),
                                    *duration,
                                    handle.clone(),
                                ))
                                .map(UiEvent::ControlsAction),
                            track_list_command,
                        ])
                    }
                    _ => track_list_command,
                }
            }

            UiEvent::SidebarAction(event) => {
                let sidebar_command = self
                    .sidebar
                    .update(event.clone())
                    .map(UiEvent::SidebarAction);

                match event {
                    components::sidebar::Event::OpenDownload => self.current_page = Page::Download,
                    components::sidebar::Event::OpenPlaylists => {
                        return {
                            self.current_page = Page::Playlist;
                            self.playlist
                                .update(playlist::Event::OpenInListMode)
                                .map(UiEvent::PlaylistAction)
                        }
                    }
                    components::sidebar::Event::OpenSettings => self.current_page = Page::Settings,
                    components::sidebar::Event::OpenTrackList => {
                        self.current_page = Page::TrackList
                    }
                    components::sidebar::Event::CreatePlaylist => {
                        return {
                            self.current_page = Page::Playlist;
                            self.playlist
                                .update(playlist::Event::OpenInCreateMode)
                                .map(UiEvent::PlaylistAction)
                        }
                    }
                    components::sidebar::Event::OpenPlaylist(index) => {
                        return {
                            self.current_page = Page::Playlist;
                            self.playlist
                                .update(playlist::Event::OpenPlaylist(index))
                                .map(UiEvent::PlaylistAction)
                        }
                    }
                    _ => (),
                }

                sidebar_command
            }

            UiEvent::ControlsAction(event) => {
                match event {
                    components::control_bar::Event::ProgressChanged(value) => {
                        self.playback_sender
                            .send(AudioEvent::SeekTo(value as u64))
                            .expect("Failed to send seek command");
                    }
                    components::control_bar::Event::PauseToggleAction => {
                        self.playback_sender
                            .send(AudioEvent::PauseToggle)
                            .expect("Failed to send pause command");
                    }
                    components::control_bar::Event::VolumeChanged(value) => {
                        self.playback_sender
                            .send(AudioEvent::SetVolume(value))
                            .expect("Failed to send volume command");
                    }
                    components::control_bar::Event::Mute => {
                        self.playback_sender
                            .send(AudioEvent::Mute)
                            .expect("Failed to send mute command");
                    }
                    components::control_bar::Event::Unmute => {
                        self.playback_sender
                            .send(AudioEvent::Unmute)
                            .expect("Failed to send unmute command");
                    }
                    _ => (),
                }
                self.controls.update(event).map(UiEvent::ControlsAction)
            }
            UiEvent::ResultsAction(event) => {
                match event.clone() {
                    results::Event::DownloadPressed(_url) => {
                        self.toasts.push(toast::Toast {
                            title: "Download".into(),
                            body: "Your download has started.".into(),
                            status: toast::Status::Secondary,
                        });
                    }
                    results::Event::DownloadComplete(status) => match status {
                        Ok(_) => {
                            self.toasts.push(toast::Toast {
                                title: "Download".into(),
                                body: "Your download has completed.".into(),
                                status: toast::Status::Success,
                            });
                        }
                        Err(_) => {
                            self.toasts.push(toast::Toast {
                                title: "Download".into(),
                                body: "Your download has failed.".into(),
                                status: toast::Status::Danger,
                            });
                        }
                    },
                    _ => (),
                }
                self.results.update(event).map(UiEvent::ResultsAction)
            }
        }
    }

    pub fn view(&self) -> iced::Element<UiEvent> {
        match &self.current_page {
            Page::Playlist => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.playlist.view().map(UiEvent::PlaylistAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }

            Page::FFmpeg => {
                let content = self.ffmpeg.view().map(UiEvent::FFmpegAction);

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }
            Page::Results => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.results.view().map(UiEvent::ResultsAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }

            Page::TrackList => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.track_list.view().map(UiEvent::TrackListAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }

            Page::Download => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.download.view().map(UiEvent::DownloadAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }

            Page::Settings => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.settings.view().map(UiEvent::SettingsAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<UiEvent> {
        Subscription::batch(vec![
            event::listen().map(UiEvent::KeyboardEvent),
            self.track_list.subscription().map(UiEvent::TrackListAction),
            self.controls.subscription().map(UiEvent::ControlsAction),
            self.ffmpeg.subscription().map(UiEvent::FFmpegAction),
        ])
    }

    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}

impl Default for Pages {
    fn default() -> Self {
        Self::new()
    }
}
