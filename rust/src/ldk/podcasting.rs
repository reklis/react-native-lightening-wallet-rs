/// Podcasting 2.0 Value-for-Value support
///
/// This module provides utilities for creating keysend payments with
/// Podcasting 2.0 metadata according to the specification.

use std::collections::HashMap;

/// Standard Podcasting 2.0 TLV type numbers
pub mod tlv_types {
    /// Podcast name
    pub const PODCAST: u64 = 7629169;

    /// Episode GUID
    pub const EPISODE: u64 = 7629171;

    /// Action type (stream, boost, etc.)
    pub const ACTION: u64 = 7629173;

    /// Timestamp in the episode
    pub const TIMESTAMP: u64 = 7629175;

    /// App name
    pub const APP_NAME: u64 = 7629177;

    /// Value msat amount (millisatoshis)
    pub const VALUE_MSAT: u64 = 7629179;

    /// Remote feed GUID
    pub const REMOTE_FEED_GUID: u64 = 7629181;

    /// Remote item GUID
    pub const REMOTE_ITEM_GUID: u64 = 7629183;
}

/// Helper to create Podcasting 2.0 custom records from value recipient data
///
/// # Example
/// ```rust
/// use std::collections::HashMap;
///
/// let mut builder = Podcasting20Builder::new();
/// builder
///     .podcast("My Podcast")
///     .episode("episode-guid-123")
///     .action("stream")
///     .custom(696969, "8");  // customKey="696969" customValue="8"
///
/// let custom_records = builder.build();
/// ```
pub struct Podcasting20Builder {
    records: HashMap<u64, Vec<u8>>,
}

impl Podcasting20Builder {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Set the podcast name
    pub fn podcast(mut self, name: &str) -> Self {
        self.records.insert(tlv_types::PODCAST, name.as_bytes().to_vec());
        self
    }

    /// Set the episode GUID
    pub fn episode(mut self, guid: &str) -> Self {
        self.records.insert(tlv_types::EPISODE, guid.as_bytes().to_vec());
        self
    }

    /// Set the action (stream, boost, etc.)
    pub fn action(mut self, action: &str) -> Self {
        self.records.insert(tlv_types::ACTION, action.as_bytes().to_vec());
        self
    }

    /// Set the timestamp in the episode
    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.records.insert(tlv_types::TIMESTAMP, timestamp.to_string().as_bytes().to_vec());
        self
    }

    /// Set the app name
    pub fn app_name(mut self, name: &str) -> Self {
        self.records.insert(tlv_types::APP_NAME, name.as_bytes().to_vec());
        self
    }

    /// Add a custom TLV record (customKey/customValue from value recipient)
    ///
    /// # Arguments
    /// * `key` - The customKey from the value recipient (TLV type number)
    /// * `value` - The customValue as a string (will be converted to bytes)
    pub fn custom(mut self, key: u64, value: &str) -> Self {
        self.records.insert(key, value.as_bytes().to_vec());
        self
    }

    /// Add a custom TLV record with raw bytes
    pub fn custom_bytes(mut self, key: u64, value: Vec<u8>) -> Self {
        self.records.insert(key, value);
        self
    }

    /// Build and return the custom records HashMap
    pub fn build(self) -> HashMap<u64, Vec<u8>> {
        self.records
    }
}

impl Default for Podcasting20Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_podcasting_builder() {
        let records = Podcasting20Builder::new()
            .podcast("My Awesome Podcast")
            .episode("ep-123")
            .action("stream")
            .custom(696969, "8")
            .build();

        assert_eq!(records.len(), 4);
        assert_eq!(records.get(&tlv_types::PODCAST), Some(&"My Awesome Podcast".as_bytes().to_vec()));
        assert_eq!(records.get(&696969), Some(&"8".as_bytes().to_vec()));
    }

    #[test]
    fn test_custom_value_numeric() {
        let records = Podcasting20Builder::new()
            .custom(696969, "8")
            .build();

        // customValue="8" becomes byte array [56] (ASCII for '8')
        assert_eq!(records.get(&696969), Some(&vec![56]));
    }
}
