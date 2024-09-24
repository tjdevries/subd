use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SunoResponse {
    pub id: String,
    pub video_url: String,
    pub audio_url: String,
    pub image_url: Option<String>,
    pub lyric: Option<String>,
    pub image_large_url: Option<String>,
    pub is_video_pending: Option<bool>,
    #[serde(default)]
    pub major_model_version: String,
    pub model_name: String,
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub handle: String,
    #[serde(default)]
    pub is_handle_updated: bool,
    #[serde(default)]
    pub avatar_image_url: String,
    #[serde(default)]
    pub is_following_creator: bool,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub play_count: i32,
    #[serde(default)]
    pub upvote_count: i32,
    #[serde(default)]
    pub is_public: bool,
}

impl SunoResponse {
    pub fn builder() -> SunoResponseBuilder {
        SunoResponseBuilder::default()
    }
}

#[derive(Default)]
pub struct SunoResponseBuilder {
    id: Option<String>,
    video_url: Option<String>,
    audio_url: Option<String>,
    image_url: Option<String>,
    lyric: Option<String>,
    image_large_url: Option<String>,
    is_video_pending: Option<bool>,
    major_model_version: Option<String>,
    model_name: Option<String>,
    metadata: Option<Metadata>,
    display_name: Option<String>,
    handle: Option<String>,
    is_handle_updated: Option<bool>,
    avatar_image_url: Option<String>,
    is_following_creator: Option<bool>,
    user_id: Option<String>,
    created_at: Option<String>,
    status: Option<String>,
    title: Option<String>,
    play_count: Option<i32>,
    upvote_count: Option<i32>,
    is_public: Option<bool>,
}

impl SunoResponseBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn video_url(mut self, url: impl Into<String>) -> Self {
        self.video_url = Some(url.into());
        self
    }

    pub fn audio_url(mut self, url: impl Into<String>) -> Self {
        self.audio_url = Some(url.into());
        self
    }

    pub fn image_url(mut self, url: impl Into<String>) -> Self {
        self.image_url = Some(url.into());
        self
    }

    pub fn lyric(mut self, lyric: impl Into<String>) -> Self {
        self.lyric = Some(lyric.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn build(self) -> SunoResponse {
        SunoResponse {
            id: self.id.unwrap_or_default(),
            video_url: self.video_url.unwrap_or_default(),
            audio_url: self.audio_url.unwrap_or_default(),
            image_url: self.image_url,
            lyric: self.lyric,
            image_large_url: self.image_large_url,
            is_video_pending: self.is_video_pending,
            major_model_version: self.major_model_version.unwrap_or_default(),
            model_name: self.model_name.unwrap_or_default(),
            metadata: self.metadata.unwrap_or_default(),
            display_name: self.display_name.unwrap_or_default(),
            handle: self.handle.unwrap_or_default(),
            is_handle_updated: self.is_handle_updated.unwrap_or_default(),
            avatar_image_url: self.avatar_image_url.unwrap_or_default(),
            is_following_creator: self.is_following_creator.unwrap_or_default(),
            user_id: self.user_id.unwrap_or_default(),
            created_at: self.created_at.unwrap_or_default(),
            status: self.status.unwrap_or_default(),
            title: self.title.unwrap_or_default(),
            play_count: self.play_count.unwrap_or_default(),
            upvote_count: self.upvote_count.unwrap_or_default(),
            is_public: self.is_public.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub tags: String,
    pub prompt: String,
    pub gpt_description_prompt: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub duration: f64,
    pub refund_credits: bool,
    pub stream: bool,
}

impl Metadata {
    /// Creates a new builder for `Metadata`.
    pub fn builder() -> MetadataBuilder {
        MetadataBuilder::default()
    }

    // Why do we need this??
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }
}

#[derive(Default)]
pub struct MetadataBuilder {
    tags: Option<String>,
    prompt: Option<String>,
    gpt_description_prompt: Option<String>,
    type_field: Option<String>,
    duration: Option<f64>,
    refund_credits: Option<bool>,
    stream: Option<bool>,
}

impl MetadataBuilder {
    pub fn tags(mut self, tags: impl Into<String>) -> Self {
        self.tags = Some(tags.into());
        self
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn gpt_description_prompt(mut self, desc: impl Into<String>) -> Self {
        self.gpt_description_prompt = Some(desc.into());
        self
    }

    pub fn type_field(mut self, type_field: impl Into<String>) -> Self {
        self.type_field = Some(type_field.into());
        self
    }

    pub fn duration(mut self, duration: f64) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn refund_credits(mut self, refund: bool) -> Self {
        self.refund_credits = Some(refund);
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn build(self) -> Metadata {
        Metadata {
            tags: self.tags.unwrap_or_default(),
            prompt: self.prompt.unwrap_or_default(),
            gpt_description_prompt: self
                .gpt_description_prompt
                .unwrap_or_default(),
            type_field: self.type_field.unwrap_or_default(),
            duration: self.duration.unwrap_or_default(),
            refund_credits: self.refund_credits.unwrap_or_default(),
            stream: self.stream.unwrap_or_default(),
        }
    }
}
