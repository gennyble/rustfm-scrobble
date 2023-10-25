pub mod responses {

    use std::fmt;

    use serde::Deserialize;
    use serde_json as json;

    #[derive(Deserialize, Debug)]
    pub struct AuthResponse {
        pub session: SessionResponse,
    }

    /// Response to an Authentication request.
    ///
    /// Contains a Session Key and the username of the authenticated Last.fm user and a subscriber ID.
    /// Only the Session Key is used internally by the crate; the other values are exposed as they may have some value
    /// for clients.
    ///
    /// [Authentication API Requests Documentation](https://www.last.fm/api/authspec)
    #[derive(Deserialize, Debug, Clone)]
    pub struct SessionResponse {
        pub key: String,
        pub subscriber: i64,
        pub name: String,
    }

    #[derive(Deserialize)]
    pub struct NowPlayingResponseWrapper {
        pub nowplaying: NowPlayingResponse,
    }

    /// Response to a Now Playing request.
    ///
    /// Represents a response to a Now Playing API request. This type can often be ignored by clients. All of the
    /// fields are [`CorrectableString`] types, which can be used to see if Last.fm applied any metadata correction
    /// to your artist, song or album.
    ///
    /// [Now Playing Request API Documentation](https://www.last.fm/api/show/track.updateNowPlaying)
    #[derive(Deserialize, Debug)]
    pub struct NowPlayingResponse {
        pub artist: CorrectableString,
        pub album: CorrectableString,
        #[serde(rename = "albumArtist")]
        pub album_artist: CorrectableString,
        pub track: CorrectableString,
    }

    #[derive(Deserialize)]
    pub struct ScrobbleResponseWrapper {
        pub scrobbles: SingleScrobble,
    }

    #[derive(Deserialize)]
    pub struct SingleScrobble {
        pub scrobble: ScrobbleResponse,
    }

    /// Response to a Scrobble request
    ///
    /// Represents a response to a Scrobble API request. Contains the results of the Scrobble call, including any
    /// metadata corrections the Last.fm API made to the arist/track/album submitted.
    ///
    /// [Scrobble Request API Documentation](https://www.last.fm/api/show/track.scrobble)
    #[derive(Deserialize, Debug, WrappedVec)]
    #[CollectionName = "ScrobbleList"]
    #[CollectionDerives = "Debug, Deserialize"]
    pub struct ScrobbleResponse {
        pub artist: CorrectableString,
        pub album: CorrectableString,
        #[serde(rename = "albumArtist")]
        pub album_artist: CorrectableString,
        pub track: CorrectableString,
        pub timestamp: String,
    }

    /// Response to a Batch Scrobble request
    ///
    /// Represents a response to a batched Scrobble request. Contains the results of the Scrobble call, including
    /// any metadata corrections the Last.fm API made to the arist/track/album submitted.
    ///
    /// [Scrobble Request API Documentation](https://www.last.fm/api/show/track.scrobble)
    #[derive(Debug)]
    pub struct BatchScrobbleResponse {
        pub scrobbles: ScrobbleList,
    }

    #[derive(Deserialize, Debug)]
    pub struct BatchScrobbleResponseWrapper {
        pub scrobbles: BatchScrobbles,
    }

    #[derive(Deserialize, Debug)]
    pub struct BatchScrobbles {
        #[serde(deserialize_with = "BatchScrobbles::deserialize_response_scrobbles")]
        #[serde(rename = "scrobble")]
        pub scrobbles: ScrobbleList,
    }

    impl BatchScrobbles {
        fn deserialize_response_scrobbles<'de, D>(de: D) -> Result<ScrobbleList, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let deser_result: json::Value = serde::Deserialize::deserialize(de)?;
            let scrobbles = match deser_result {
                obj @ json::Value::Object(_) => {
                    let scrobble: ScrobbleResponse =
                        serde_json::from_value(obj).expect("Parsing scrobble failed");
                    ScrobbleList::from(vec![scrobble])
                }
                arr @ json::Value::Array(_) => {
                    let scrobbles: ScrobbleList =
                        serde_json::from_value(arr).expect("Parsing scrobble list failed");
                    scrobbles
                }
                _ => ScrobbleList::from(vec![]),
            };
            Ok(scrobbles)
        }
    }

    /// Represents a string that can be marked as 'corrected' by the Last.fm API.
    ///
    /// All Scrobble/NowPlaying responses have their fields as `CorrectableString`'s. The API will sometimes change
    /// the artist/song name/album name data that you have submitted. For example - it is common for Bjork to be turned
    /// into Björk by the API; the modified artist field would be marked `corrected = true`, `text = "Björk".
    ///
    /// Most clients can ignore these corrections, but the information is exposed for clients that require it.
    ///
    /// [Meta-Data Correction Documentation](https://www.last.fm/api/scrobbling#meta-data-corrections)
    #[derive(Deserialize, Debug)]
    pub struct CorrectableString {
        #[serde(deserialize_with = "CorrectableString::deserialize_corrected_field")]
        pub corrected: bool,
        #[serde(rename = "#text", default)]
        pub text: String,
    }

    impl CorrectableString {
        fn deserialize_corrected_field<'de, D>(de: D) -> Result<bool, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let deser_result: json::Value = serde::Deserialize::deserialize(de)?;
            match deser_result {
                json::Value::String(ref s) if s == "1" => Ok(true),
                json::Value::String(ref s) if s == "0" => Ok(false),
                _ => Err(serde::de::Error::custom("Unexpected value")),
            }
        }
    }

    impl fmt::Display for CorrectableString {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.text)
        }
    }
}

pub mod metadata {

    use std::collections::HashMap;

    /// Repesents a single music track played at a point in time. In the Last.fm universe, this is known as a
    /// "scrobble".
    ///
    /// Takes an artist, track and album name. Can hold a timestamp indicating when the track was listened to.
    /// `Scrobble` objects are submitted via [`Scrobbler::now_playing`], [`Scrobbler::scrobble`] and batches of
    /// Scrobbles are sent via [`Scrobbler::scrobble_batch`].
    ///
    /// [`Scrobbler::now_playing`]: struct.Scrobbler.html#method.now_playing
    /// [`Scrobbler::scrobble`]: struct.Scrobbler.html#method.scrobble
    /// [`Scrobbler::scrobble_batch`]: struct.Scrobbler.html#method.scrobble_batch
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, WrappedVec)]
    #[CollectionName = "ScrobbleBatch"]
    #[CollectionDoc = "A batch of Scrobbles to be submitted to Last.fm together."]
    #[CollectionDerives = "Clone, Debug"]
    pub struct Scrobble {
        artist: String,
        track: String,
        album: String,

        timestamp: Option<u64>,
    }

    impl Scrobble {
        /// Constructs a new Scrobble instance, representing a single playthrough of a music track. `Scrobble`s are
        /// submitted to Last.fm via an instance of [`Scrobbler`]. A new `Scrobble` requires an artist name, song/track
        /// name, and an album name.
        ///
        /// # Example
        /// ```ignore
        /// let scrobble = Scrobble::new("Example Artist", "Example Track", "Example Album")
        /// ```
        ///
        /// [`Scrobbler`]: struct.Scrobbler.html
        pub fn new(artist: &str, track: &str, album: &str) -> Self {
            Self {
                artist: artist.to_owned(),
                track: track.to_owned(),
                album: album.to_owned(),
                timestamp: None,
            }
        }

        /// Sets the timestamp (date/time of play) of a Scrobble. Used in a builder-style pattern, typically after
        /// [`Scrobble::new`].
        ///
        /// # Example
        /// ```ignore
        /// let mut scrobble = Scrobble::new(...).with_timestamp(12345);
        /// ```
        ///
        /// # Note on Timestamps
        /// Scrobbles without timestamps are automatically assigned a timestamp of the current time when
        /// submitted via [`Scrobbler::scrobble`] or [`Scrobbler::scrobble_batch`]. Timestamps only need to be
        /// explicitly set when you are submitting a Scrobble at a point in the past, or in the future.
        ///
        /// [`Scrobble::new`]: struct.Scrobble.html#method.new
        /// [`Scrobbler::scrobble`]: struct.Scrobbler.html#method.scrobble
        /// [`Scrobbler::scrobble_batch`]: struct.Scrobbler.html#method.scrobble_batch
        pub fn with_timestamp(&mut self, timestamp: u64) -> &mut Self {
            self.timestamp = Some(timestamp);
            self
        }

        /// Converts the Scrobble metadata (track name, artist & album name) into a `HashMap`. Map keys are
        /// `"track"`, `"artist"` and `"album"`. If a timestamp is set, it will be present in the map under key
        /// `"timestamp"`.
        ///
        /// # Example
        /// ```ignore
        /// let scrobble = Scrobble::new("Example Artist", ...);
        /// let scrobble_map = scrobble.as_map();
        /// assert_eq!(scrobble_map.get("artist"), "Example Artist");
        /// ```
        pub fn as_map(&self) -> HashMap<String, String> {
            let mut params = HashMap::new();
            params.insert("track".to_string(), self.track.clone());
            params.insert("artist".to_string(), self.artist.clone());
            params.insert("album".to_string(), self.album.clone());

            if let Some(timestamp) = self.timestamp {
                params.insert("timestamp".to_string(), timestamp.to_string());
            }

            params
        }

        /// Returns the `Scrobble`'s artist name
        pub fn artist(&self) -> &str {
            &self.artist
        }

        /// Returns the `Scrobble`'s track name
        pub fn track(&self) -> &str {
            &self.track
        }

        /// Returns the `Scrobble`'s album name
        pub fn album(&self) -> &str {
            &self.album
        }
    }

    /// Converts from tuple of `&str`s in the form `(artist, track, album)`
    ///
    /// Designed to make it easier to cooperate with other track info types.
    impl From<&(&str, &str, &str)> for Scrobble {
        fn from((artist, track, album): &(&str, &str, &str)) -> Self {
            Scrobble::new(artist, track, album)
        }
    }

    /// Converts from tuple of `String`s in the form `(artist, track, album)`
    ///
    /// Designed to make it easier to cooperate with other track info types.
    impl From<&(String, String, String)> for Scrobble {
        fn from((artist, track, album): &(String, String, String)) -> Self {
            Scrobble::new(artist, track, album)
        }
    }

    /// Converts from vector of `&str` tuples, in the form `(artist, track, album)`.
    ///
    /// Designed to make it easier to cooperate with other track info types.
    impl From<Vec<(&str, &str, &str)>> for ScrobbleBatch {
        fn from(collection: Vec<(&str, &str, &str)>) -> Self {
            let scrobbles: Vec<Scrobble> = collection.iter().map(Scrobble::from).collect();

            ScrobbleBatch::from(scrobbles)
        }
    }

    /// Converts from vector of `String` tuples, in the form `(artist, track, album)`.
    ///
    /// Designed to make it easier to cooperate with other track info types.
    impl From<Vec<(String, String, String)>> for ScrobbleBatch {
        fn from(collection: Vec<(String, String, String)>) -> Self {
            let scrobbles: Vec<Scrobble> = collection.iter().map(Scrobble::from).collect();

            ScrobbleBatch::from(scrobbles)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn make_scrobble() {
            let mut scrobble = Scrobble::new(
                "foo floyd and the fruit flies",
                "old bananas",
                "old bananas",
            );
            scrobble.with_timestamp(1337);
            assert_eq!(scrobble.artist(), "foo floyd and the fruit flies");
            assert_eq!(scrobble.track(), "old bananas");
            assert_eq!(scrobble.album(), "old bananas");
            assert_eq!(scrobble.timestamp, Some(1337));
        }

        #[test]
        fn make_scrobble_check_map() {
            let scrobble = Scrobble::new(
                "foo floyd and the fruit flies",
                "old bananas",
                "old bananas",
            );

            let params = scrobble.as_map();
            assert_eq!(params["artist"], "foo floyd and the fruit flies");
            assert_eq!(params["track"], "old bananas");
            assert_eq!(params["album"], "old bananas");
        }
    }
}
