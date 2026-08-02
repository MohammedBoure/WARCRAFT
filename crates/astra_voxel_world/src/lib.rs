//! Deterministic voxel world generation for long-distance Minecraft-like worlds.
//!
//! This crate is intentionally independent from the planet surface renderer.
//! It owns both chunk generation and a small reusable Bevy mesh renderer so the
//! standalone viewer and the in-game planet visit can share the same visual
//! result without inheriting the old planet-view architecture.
//!
//! `VoxelWorldSettings::composition` lets callers tune biome, weather, and
//! resource ratios while keeping generation deterministic for the same seed and
//! settings.

pub mod edit;
pub mod generator;
pub mod model;
mod noise;
pub mod prelude;
pub mod render;
pub mod visual;
