use std::{
    fs::OpenOptions,
    io::Write,
    ops::{Deref, DerefMut},
};

use crate::*;
use oggvorbismeta::{CommentHeader, VorbisComments};

use crate::TagType;

pub struct VorbisInnerTag {
    inner: CommentHeader,
}

impl VorbisInnerTag {
    pub fn new() -> Self {
        Self {
            inner: CommentHeader::new(),
        }
    }

    pub fn read_from_path(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let inner = oggvorbismeta::read_comment_header(&mut file);

        Ok(Self { inner })
    }
}

impl Default for VorbisInnerTag {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for VorbisInnerTag {
    type Target = CommentHeader;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VorbisInnerTag {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Default)]
pub struct VorbisTag {
    inner: VorbisInnerTag,
    config: Config,

    // we need the fields here already
    // because we need to implement AudioTagEdit
    title: Option<String>,
    artist: Option<String>,
    album_title: Option<String>,
    genre: Option<String>,
    comment: Option<String>,
}

impl VorbisTag {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        let inner = VorbisInnerTag::read_from_path(path)?;

        Ok(Self {
            config: Config::default(),
            title: inner.get_tag_single("TITLE"),
            artist: inner.get_tag_single("ARTIST"),
            album_title: inner.get_tag_single("ALBUM"),
            genre: inner.get_tag_single("GENRE"),
            comment: inner.get_tag_single("DESCRIPTION"),
            inner,
        })
    }
}

impl_audiotag_config!(VorbisTag);

use std::any::Any;

impl ToAnyTag for VorbisTag {
    fn to_anytag(&self) -> AnyTag<'_> {
        self.into()
    }
}

impl ToAny for VorbisTag {
    fn to_any(&self) -> &dyn Any {
        self
    }
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl AudioTag for VorbisTag {}

// From wrapper to inner (same type)
impl From<VorbisTag> for VorbisInnerTag {
    fn from(inp: VorbisTag) -> Self {
        inp.inner
    }
}

// From inner to wrapper (same type)
impl From<VorbisInnerTag> for VorbisTag {
    fn from(inner: VorbisInnerTag) -> Self {
        Self {
            config: Config::default(),
            title: inner.get_tag_single("TITLE"),
            artist: inner.get_tag_single("ARTIST"),
            album_title: inner.get_tag_single("ALBUM"),
            genre: inner.get_tag_single("GENRE"),
            comment: inner.get_tag_single("DESCRIPTION"),
            inner,
        }
    }
}

// From dyn AudioTag to wrapper (any type)
impl From<Box<dyn AudioTag + Send + Sync>> for VorbisTag {
    fn from(inp: Box<dyn AudioTag + Send + Sync>) -> Self {
        let mut inp = inp;
        if let Some(t_refmut) = inp.to_any_mut().downcast_mut::<VorbisTag>() {
            // TODO: can we avoid creating the dummy tag?
            std::mem::replace(t_refmut, VorbisTag::new())
        } else {
            let mut t = inp.to_dyn_tag(TagType::Vorbis);
            let t_refmut = t.to_any_mut().downcast_mut::<VorbisTag>().unwrap();

            std::mem::replace(t_refmut, VorbisTag::new())
        }
    }
}
// From dyn AudioTag to inner (any type)
impl std::convert::From<Box<dyn AudioTag + Send + Sync>> for VorbisInnerTag {
    fn from(inp: Box<dyn AudioTag + Send + Sync>) -> Self {
        let t: VorbisTag = inp.into();
        t.into()
    }
}

impl<'a> From<&'a VorbisTag> for AnyTag<'a> {
    fn from(inp: &'a VorbisTag) -> Self {
        Self {
            config: inp.config,
            title: inp.title(),
            artists: inp.artists(),
            year: inp.year(),
            duration: None,
            album_title: inp.album_title(),
            album_artists: inp.album_artists(),
            album_cover: inp.album_cover(),
            track_number: inp.track_number(),
            total_tracks: inp.total_tracks(),
            disc_number: inp.disc_number(),
            total_discs: inp.total_discs(),
            genre: inp.genre(),
            composer: inp.composer(),
            comment: inp.comment(),
        }
    }
}

impl<'a> From<AnyTag<'a>> for VorbisTag {
    fn from(inp: AnyTag<'a>) -> Self {
        Self {
            config: inp.config,
            inner: {
                let mut t = VorbisInnerTag::new();
                if let Some(v) = inp.title() {
                    t.add_tag_single("TITLE", v);
                }
                if let Some(v) = inp.artists_as_string() {
                    t.add_tag_single("ARTIST", &v);
                }
                if let Some(v) = inp.year {
                    t.add_tag_single("DATE", &v.to_string());
                }
                if let Some(v) = inp.album_title() {
                    t.add_tag_single("ALBUM", v);
                }
                if let Some(v) = inp.track_number() {
                    t.add_tag_single("TRACKNUMBER", &v.to_string());
                }
                if let Some(v) = inp.genre() {
                    t.add_tag_single("GENRE", v);
                }
                if let Some(v) = inp.comment() {
                    t.add_tag_single("DESCRIPTION", v);
                }
                t
            },
            title: inp.title().map(|x| x.to_string()),
            artist: inp.artists_as_string(),
            album_title: inp.album_title().map(|x| x.to_string()),
            genre: inp.genre().map(|x| x.to_string()),
            comment: inp.comment().map(|x| x.to_string()),
        }
    }
}

impl AudioTagEdit for VorbisTag {
    fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    fn set_title(&mut self, title: &str) {
        self.inner.add_tag_single("TITLE", title);
    }

    fn remove_title(&mut self) {
        self.inner.clear_tag("TITLE");
    }

    fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    fn set_artist(&mut self, artist: &str) {
        self.inner.add_tag_single("ARTIST", artist);
    }

    fn remove_artist(&mut self) {
        self.inner.clear_tag("ARTIST");
    }

    fn year(&self) -> Option<i32> {
        self.inner
            .get_tag_single("DATE")
            .map(|x| x.as_str().parse::<i32>().unwrap())
    }

    fn set_year(&mut self, year: i32) {
        self.inner.add_tag_single("DATE", &year.to_string());
    }

    fn remove_year(&mut self) {
        self.inner.clear_tag("DATE");
    }

    fn album_title(&self) -> Option<&str> {
        self.album_title.as_deref()
    }

    fn set_album_title(&mut self, album_title: &str) {
        self.inner.add_tag_single("ALBUM", album_title);
    }

    fn remove_album_title(&mut self) {
        self.inner.clear_tag("ALBUM");
    }

    fn album_artist(&self) -> Option<&str> {
        None
    }

    fn set_album_artist(&mut self, _album_artist: &str) {
        // there is no album artist tag in vorbis
    }

    fn remove_album_artist(&mut self) {
        // there is no album artist tag in vorbis
    }

    fn album_cover(&self) -> Option<Picture> {
        None
    }

    fn set_album_cover(&mut self, _cover: Picture) {
        // there is no album cover tag in vorbis
    }

    fn remove_album_cover(&mut self) {
        // there is no album cover tag in vorbis
    }

    fn composer(&self) -> Option<&str> {
        None
    }

    fn set_composer(&mut self, _composer: String) {
        // there is no composer tag in vorbis
    }

    fn remove_composer(&mut self) {
        // there is no composer tag in vorbis
    }

    fn track_number(&self) -> Option<u16> {
        self.inner
            .get_tag_single("TRACKNUMBER")
            .map(|x| x.as_str().parse::<u16>().unwrap())
    }

    fn set_track_number(&mut self, track_number: u16) {
        self.inner
            .add_tag_single("TRACKNUMBER", &track_number.to_string());
    }

    fn remove_track_number(&mut self) {
        self.inner.clear_tag("TRACKNUMBER");
    }

    fn total_tracks(&self) -> Option<u16> {
        None
    }

    fn set_total_tracks(&mut self, _total_tracks: u16) {
        // there is no total tracks tag in vorbis
    }

    fn remove_total_tracks(&mut self) {
        // there is no total tracks tag in vorbis
    }

    fn disc_number(&self) -> Option<u16> {
        None
    }

    fn set_disc_number(&mut self, _disc_number: u16) {
        // there is no disc number tag in vorbis
    }

    fn remove_disc_number(&mut self) {
        // there is no disc number tag in vorbis
    }

    fn total_discs(&self) -> Option<u16> {
        None
    }

    fn set_total_discs(&mut self, _total_discs: u16) {
        // there is no total discs tag in vorbis
    }

    fn remove_total_discs(&mut self) {
        // there is no total discs tag in vorbis
    }

    fn genre(&self) -> Option<&str> {
        self.genre.as_deref()
    }

    fn set_genre(&mut self, genre: &str) {
        self.inner.add_tag_single("GENRE", genre);
    }

    fn remove_genre(&mut self) {
        self.inner.clear_tag("GENRE");
    }

    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    fn set_comment(&mut self, comment: String) {
        self.inner.add_tag_single("DESCRIPTION", &comment);
    }

    fn remove_comment(&mut self) {
        self.inner.clear_tag("DESCRIPTION");
    }

    fn duration(&self) -> Option<f64> {
        None
    }
}

impl AudioTagWrite for VorbisTag {
    fn write_to(&mut self, file: &mut std::fs::File) -> crate::Result<()> {
        let mut cursor = oggvorbismeta::replace_comment_header(file, self.inner.clone());
        cursor.flush()?;

        Ok(())
    }

    fn write_to_path(&mut self, path: &str) -> crate::Result<()> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;

        self.write_to(&mut file)
    }
}
