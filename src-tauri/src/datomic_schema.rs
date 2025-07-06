use serde_json::json;

pub fn gita_schema() -> String {
    json!([
        // Block Attributes
        {
            ":db/ident": ":block/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of a block."
        },
        {
            ":db/ident": ":block/content",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The textual content of a block."
        },
        {
            ":db/ident": ":block/is_page",
            ":db/valueType": ":db.type/boolean",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "Whether this block represents a page."
        },
        {
            ":db/ident": ":block/page_title",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The title of the page, if this block is a page."
        },
        {
            ":db/ident": ":block/parent",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the parent block."
        },
        {
            ":db/ident": ":block/order",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The order of the block within its parent."
        },
        {
            ":db/ident": ":block/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the block."
        },
        {
            ":db/ident": ":block/updated_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The last update timestamp of the block."
        },

        // Audio Recording Attributes
        {
            ":db/ident": ":audio/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of an audio recording."
        },
        {
            ":db/ident": ":audio/page",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the page this audio recording belongs to."
        },
        {
            ":db/ident": ":audio/path",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The path to the audio recording file."
        },
        {
            ":db/ident": ":audio/duration",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The duration of the audio recording in seconds."
        },
        {
            ":db/ident": ":audio/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the audio recording."
        },

        // Timestamp Attributes
        {
            ":db/ident": ":timestamp/block",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the block associated with this timestamp."
        },
        {
            ":db/ident": ":timestamp/recording_id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The ID of the recording this timestamp belongs to."
        },
        {
            ":db/ident": ":timestamp/seconds",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The timestamp in seconds from the start of the recording."
        }
    ]).to_string()
}

/// Return the schema in EDN format for the Peer API
pub fn gita_schema_edn() -> serde_json::Value {
    json!([
        // Block Attributes
        {
            ":db/ident": ":block/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of a block."
        },
        {
            ":db/ident": ":block/content",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The textual content of a block."
        },
        {
            ":db/ident": ":block/is_page",
            ":db/valueType": ":db.type/boolean",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "Whether this block represents a page."
        },
        {
            ":db/ident": ":block/page_title",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The title of the page, if this block is a page."
        },
        {
            ":db/ident": ":block/parent",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the parent block."
        },
        {
            ":db/ident": ":block/order",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The order of the block within its parent."
        },
        {
            ":db/ident": ":block/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the block."
        },
        {
            ":db/ident": ":block/updated_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The last update timestamp of the block."
        },

        // Audio Recording Attributes
        {
            ":db/ident": ":audio/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of an audio recording."
        },
        {
            ":db/ident": ":audio/page",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the page this audio recording belongs to."
        },
        {
            ":db/ident": ":audio/path",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The path to the audio recording file."
        },
        {
            ":db/ident": ":audio/duration",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The duration of the audio recording in seconds."
        },
        {
            ":db/ident": ":audio/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the audio recording."
        },

        // Timestamp Attributes
        {
            ":db/ident": ":timestamp/block",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the block associated with this timestamp."
        },
        {
            ":db/ident": ":timestamp/recording_id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The ID of the recording this timestamp belongs to."
        },
        {
            ":db/ident": ":timestamp/seconds",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The timestamp in seconds from the start of the recording."
        }
    ])
}
    json!([
        // Block Attributes
        {
            ":db/ident": ":block/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of a block."
        },
        {
            ":db/ident": ":block/content",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The textual content of a block."
        },
        {
            ":db/ident": ":block/is_page",
            ":db/valueType": ":db.type/boolean",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "Whether this block represents a page."
        },
        {
            ":db/ident": ":block/page_title",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The title of the page, if this block is a page."
        },
        {
            ":db/ident": ":block/parent",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the parent block."
        },
        {
            ":db/ident": ":block/order",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The order of the block within its parent."
        },
        {
            ":db/ident": ":block/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the block."
        },
        {
            ":db/ident": ":block/updated_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The last update timestamp of the block."
        },

        // Audio Recording Attributes
        {
            ":db/ident": ":audio/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of an audio recording."
        },
        {
            ":db/ident": ":audio/page",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the page this audio recording belongs to."
        },
        {
            ":db/ident": ":audio/path",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The path to the audio recording file."
        },
        {
            ":db/ident": ":audio/duration",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The duration of the audio recording in seconds."
        },
        {
            ":db/ident": ":audio/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the audio recording."
        },

        // Timestamp Attributes
        {
            ":db/ident": ":timestamp/block",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the block associated with this timestamp."
        },
        {
            ":db/ident": ":timestamp/recording_id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The ID of the recording this timestamp belongs to."
        },
        {
            ":db/ident": ":timestamp/seconds",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The timestamp in seconds from the start of the recording."
        }
    ]).to_string()
}

/// Return the schema in EDN format for the Peer API
pub fn gita_schema_edn() -> serde_json::Value {
    json!([
        // Block Attributes
        {
            ":db/ident": ":block/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of a block."
        },
        {
            ":db/ident": ":block/content",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The textual content of a block."
        },
        {
            ":db/ident": ":block/is_page",
            ":db/valueType": ":db.type/boolean",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "Whether this block represents a page."
        },
        {
            ":db/ident": ":block/page_title",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The title of the page, if this block is a page."
        },
        {
            ":db/ident": ":block/parent",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the parent block."
        },
        {
            ":db/ident": ":block/order",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The order of the block within its parent."
        },
        {
            ":db/ident": ":block/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the block."
        },
        {
            ":db/ident": ":block/updated_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The last update timestamp of the block."
        },

        // Audio Recording Attributes
        {
            ":db/ident": ":audio/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of an audio recording."
        },
        {
            ":db/ident": ":audio/page",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the page this audio recording belongs to."
        },
        {
            ":db/ident": ":audio/path",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The path to the audio recording file."
        },
        {
            ":db/ident": ":audio/duration",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The duration of the audio recording in seconds."
        },
        {
            ":db/ident": ":audio/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the audio recording."
        },

        // Timestamp Attributes
        {
            ":db/ident": ":timestamp/block",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the block associated with this timestamp."
        },
        {
            ":db/ident": ":timestamp/recording_id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The ID of the recording this timestamp belongs to."
        },
        {
            ":db/ident": ":timestamp/seconds",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The timestamp in seconds from the start of the recording."
        }
    ])
}
    json!([
        // Block Attributes
        {
            ":db/ident": ":block/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of a block."
        },
        {
            ":db/ident": ":block/content",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The textual content of a block."
        },
        {
            ":db/ident": ":block/is_page",
            ":db/valueType": ":db.type/boolean",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "Whether this block represents a page."
        },
        {
            ":db/ident": ":block/page_title",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The title of the page, if this block is a page."
        },
        {
            ":db/ident": ":block/parent",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the parent block."
        },
        {
            ":db/ident": ":block/order",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The order of the block within its parent."
        },
        {
            ":db/ident": ":block/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the block."
        },
        {
            ":db/ident": ":block/updated_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The last update timestamp of the block."
        },

        // Audio Recording Attributes
        {
            ":db/ident": ":audio/id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/unique": ":db.unique/identity",
            ":db/doc": "The unique ID of an audio recording."
        },
        {
            ":db/ident": ":audio/page",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the page this audio recording belongs to."
        },
        {
            ":db/ident": ":audio/path",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The path to the audio recording file."
        },
        {
            ":db/ident": ":audio/duration",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The duration of the audio recording in seconds."
        },
        {
            ":db/ident": ":audio/created_at",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The creation timestamp of the audio recording."
        },

        // Timestamp Attributes
        {
            ":db/ident": ":timestamp/block",
            ":db/valueType": ":db.type/ref",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "A reference to the block associated with this timestamp."
        },
        {
            ":db/ident": ":timestamp/recording_id",
            ":db/valueType": ":db.type/string",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The ID of the audio recording."
        },
        {
            ":db/ident": ":timestamp/timestamp_ms",
            ":db/valueType": ":db.type/long",
            ":db/cardinality": ":db.cardinality/one",
            ":db/doc": "The timestamp in milliseconds within the audio recording."
        }
    ]).to_string()
}
