# Gita Migration Plan: PostgreSQL to Datomic

This document outlines the tasks required to migrate the Gita application's backend from a PostgreSQL database to Datomic.

## Part 1: Environment Setup & Project Configuration

- [ ] **Install JDK:** Install Java Development Kit (JDK) 17 on your local Windows machine and configure `JAVA_HOME` and `PATH` environment variables.
- [ ] **Download Datomic:** Download Datomic Pro On-Prem and unzip it to a local directory (e.g., `C:\datomic`).
- [ ] **Run Peer Server:** Practice starting the Datomic Peer Server from the command line and ensure it can create a local database.
- [ ] **Update `Cargo.toml`:**
    - [ ] Remove `sqlx`, `sqlx-postgres`, and related dependencies.
    - [ ] Add `reqwest` (with `json` feature) for making HTTP requests.
    - [ ] Add `serde` and `serde_json` for handling JSON data.
    - [ ] Add `tokio` with `full` features if not already present for async operations.
- [ ] **Update Tauri State:** In `main.rs`, remove the `PgPool` from the application state and replace it with a `reqwest::Client`.

## Part 2: Schema and Database Connectivity

- [ ] **Create Datomic Schema:**
    - [ ] Create a new file `src-tauri/src/datomic_schema.rs`.
    - [ ] Define the Datomic schema as a Rust function that returns a JSON string. This schema will define attributes like `:block/content`, `:block/parent` (`:db/ref`), `:timestamp/recording_path`, etc.
- [ ] **Implement Initial Schema Transaction:**
    - [ ] In `database.rs`, create a new function `ensure_schema_is_present`.
    - [ ] This function will query the database to see if the schema attributes already exist.
    - [ ] If they don't, it will transact the schema definition to the Datomic Peer Server via an HTTP POST request to its `/api/transact` endpoint.
    - [ ] Call this function once on application startup.

## Part 3: Rewriting Database Operations

- [ ] **Refactor `database.rs`:**
    - [ ] Remove all existing `sqlx` based functions.
    - [ ] Create a new `DatomicClient` struct or a set of functions that will handle communication with the Peer Server.
- [ ] **Rewrite Core Functions:**
    - [ ] **`create_block`**:
        - [ ] Rewrite to accept block data.
        - [ ] Construct a Datomic transaction using `serde_json::json!`.
        - [ ] `POST` the transaction to the `/api/transact` endpoint.
        - [ ] Parse the response to get the entity ID of the newly created block.
    - [ ] **`update_block`**:
        - [ ] Rewrite to accept a block's entity ID and new content.
        - [ ] Construct a transaction to add the new value for the `:block/content` attribute.
        - [ ] `POST` the transaction.
    - [ ] **`get_block` / `get_block_with_children`**:
        - [ ] Rewrite to accept a block's entity ID or page title.
        - [ ] Construct a Datalog query to find the block and its children (using the `:block/parent` reference).
        - [ ] `POST` the query to the `/api/query` endpoint.
        - [ ] Deserialize the JSON response into the `Block` structs from `models.rs`.
    - [ ] **`get_or_create_daily_note_block`**:
        - [ ] Rewrite the logic to first query for a daily note page by its title (e.g., "July 05, 2025").
        - [ ] If the query returns a result, return that block's data.
        - [ ] If not, run a transaction to create the new page block.
    - [ ] **`create_audio_timestamp`**:
        - [ ] Rewrite to create a new `:timestamp` entity.
        - [ ] The transaction must include a reference to the block it belongs to (e.g., `[":db/add", -1, ":timestamp/block", <block_entity_id>]`).
    - [ ] **`get_audio_timestamp_for_block`**:
        - [ ] Rewrite to query for a `:timestamp` entity that has a `:timestamp/block` reference pointing to the given block entity ID.

## Part 4: Integration and Verification

- [ ] **Update `main.rs` Tauri Commands:**
    - [ ] Go through each `#[tauri::command]` function.
    - [ ] Replace the calls to the old `database.rs` functions with calls to the new Datomic-based functions.
    - [ ] Adjust data handling as needed to match the new return types and structures.
- [ ] **Data Migration (Manual Task):**
    - [ ] Create a separate, one-off Rust binary or script.
    - [ ] This script will:
        1. Connect to the old PostgreSQL database.
        2. `SELECT` all data from the `blocks` and `audio_timestamps` tables.
        3. For each row, construct a corresponding Datomic transaction.
        4. `POST` these transactions to the new Datomic Peer Server to populate it with existing data.
- [ ] **Thorough Testing:**
    - [ ] Launch the application and test every feature: creating notes, editing, linking, creating daily notes, and audio timestamping/playback.
    - [ ] Check the logs from the Datomic Peer Server for any transaction or query errors.
