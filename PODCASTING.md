# Podcasting 2.0 Support

This wallet has full support for Podcasting 2.0 value-for-value payments using Lightning keysend with custom TLV records.

## Overview

Podcasting 2.0 value recipients (from the RSS feed) look like this:

```xml
<podcast:valueRecipient
    name="John"
    type="node"
    address="035c42ea716c7ffa3c380a73f1de7ba82a61f0c8af710b175d6ecbdb6b64edda6a"
    split="99"
    customKey="696969"
    customValue="8"/>
```

## Sending Payments

### Method 1: Direct JSON (React Native)

```javascript
const params = {
  destination_pubkey: "035c42ea716c7ffa3c380a73f1de7ba82a61f0c8af710b175d6ecbdb6b64edda6a",
  amount_sats: 1000,
  custom_records: {
    // Standard Podcasting 2.0 TLV types
    "7629169": Array.from(Buffer.from("My Podcast", "utf8")),        // podcast name
    "7629171": Array.from(Buffer.from("episode-guid-123", "utf8")), // episode GUID
    "7629173": Array.from(Buffer.from("stream", "utf8")),           // action
    "7629175": Array.from(Buffer.from("3600", "utf8")),             // timestamp (1 hour)

    // Custom key/value from value recipient
    "696969": Array.from(Buffer.from("8", "utf8"))                  // customValue="8"
  }
};

const result = await sendKeysend(userId, params);
```

### Method 2: Using Rust Helper (Backend)

```rust
use crate::ldk::{KeysendParams, Podcasting20Builder};

// Build custom records using the helper
let custom_records = Podcasting20Builder::new()
    .podcast("My Awesome Podcast")
    .episode("episode-guid-123")
    .action("stream")
    .timestamp(3600)  // 1 hour into episode
    .custom(696969, "8")  // customKey="696969" customValue="8"
    .build();

// Create keysend params
let params = KeysendParams {
    destination_pubkey: "035c42ea...".to_string(),
    amount_sats: 1000,
    custom_records,
};

// Send the payment
lightning_node.send_keysend(params)?;
```

## Standard TLV Types

The following TLV types are defined in the Podcasting 2.0 specification:

| TLV Type | Name | Description |
|----------|------|-------------|
| 7629169 | `podcast` | Podcast name |
| 7629171 | `episode` | Episode GUID |
| 7629173 | `action` | Action type (stream, boost, etc.) |
| 7629175 | `timestamp` | Position in episode (seconds) |
| 7629177 | `app_name` | App sending the payment |
| 7629179 | `value_msat` | Value in millisatoshis |
| 7629181 | `remote_feed_guid` | Remote feed GUID |
| 7629183 | `remote_item_guid` | Remote item GUID |

## Custom Keys

Any custom key can be used (like `696969` in the example). The `customKey` becomes the TLV type number, and `customValue` is encoded as bytes.

### Important: Data Encoding

All values must be provided as **byte arrays**:

- **JavaScript/TypeScript**: Use `Array.from(Buffer.from(value, "utf8"))`
- **Rust**: Use `value.as_bytes().to_vec()` or the `Podcasting20Builder`

## Complete Example: Processing RSS Value Recipients

```javascript
// Parse value recipient from RSS feed
const recipient = {
  name: "John",
  address: "035c42ea716c7ffa3c380a73f1de7ba82a61f0c8af710b175d6ecbdb6b64edda6a",
  split: 99,
  customKey: "696969",
  customValue: "8"
};

// Build payment params
const params = {
  destination_pubkey: recipient.address,
  amount_sats: calculateSplit(totalAmount, recipient.split), // 99% of total
  custom_records: {
    "7629169": toBytes("My Podcast"),
    "7629171": toBytes("ep-guid-123"),
    "7629173": toBytes("stream"),
    "7629175": toBytes(currentTimestamp.toString()),
    [recipient.customKey]: toBytes(recipient.customValue)
  }
};

// Helper function
function toBytes(str) {
  return Array.from(Buffer.from(str, "utf8"));
}

// Send payment
await sendKeysend(userId, params);
```

## Streaming Payments

For continuous "streaming" payments (every minute while listening):

```javascript
// Every 60 seconds while playing
setInterval(async () => {
  const satsPerMinute = 100;
  const timestamp = Math.floor(audioPlayer.currentTime);

  const params = {
    destination_pubkey: recipient.address,
    amount_sats: satsPerMinute,
    custom_records: {
      "7629169": toBytes(podcastName),
      "7629171": toBytes(episodeGuid),
      "7629173": toBytes("stream"),
      "7629175": toBytes(timestamp.toString()),
      "7629177": toBytes("MyPodcastApp"),
      [recipient.customKey]: toBytes(recipient.customValue)
    }
  };

  await sendKeysend(userId, params);
}, 60000);
```

## Boost Payments

For one-time "boost" payments (when user sends a tip):

```javascript
async function sendBoost(amount) {
  const params = {
    destination_pubkey: recipient.address,
    amount_sats: amount,
    custom_records: {
      "7629169": toBytes(podcastName),
      "7629171": toBytes(episodeGuid),
      "7629173": toBytes("boost"),  // action = boost
      "7629175": toBytes(Math.floor(audioPlayer.currentTime).toString()),
      "7629177": toBytes("MyPodcastApp"),
      [recipient.customKey]: toBytes(recipient.customValue)
    }
  };

  return await sendKeysend(userId, params);
}
```

## Testing

You can test with a Podcasting 2.0 compatible wallet like:
- Fountain FM
- Breez
- Podverse

Or use the `podcast:value` tag parser from any Podcasting 2.0 compliant app.

## References

- [Podcasting 2.0 Value Spec](https://github.com/Podcastindex-org/podcast-namespace/blob/main/value/value.md)
- [TLV Record Spec](https://github.com/lightning/blips/blob/master/blip-0010.md)
- [Lightning Keysend](https://github.com/lightning/blips/blob/master/blip-0003.md)
