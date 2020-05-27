#![feature(min_specialization)]
pub mod batch;
mod camera;
pub mod entity;
pub mod mesh;
pub mod render_graph;
pub mod renderer;
pub mod shader;
pub mod vertex;

mod color;

pub use camera::*;
pub use color::*;
pub use renderable::*;

pub use vertex::Vertex;

pub mod base_render_graph;
pub mod draw_target;
pub mod pass;
pub mod pipeline;
pub mod render_resource;
mod renderable;
pub mod texture;

pub use once_cell;

use self::{
    mesh::Mesh,
    pipeline::{
        PipelineAssignments, PipelineCompiler, PipelineDescriptor, VertexBufferDescriptors,
    },
    render_resource::{
        entity_render_resource_assignments_system, EntityRenderResourceAssignments,
        RenderResourceAssignments,
    },
    shader::Shader,
    texture::Texture,
};

use base_render_graph::{BaseRenderGraphBuilder, BaseRenderGraphConfig};
use bevy_app::{stage, AppBuilder, AppPlugin};
use bevy_asset::AddAsset;
use bevy_component_registry::RegisterComponent;
use legion::prelude::IntoSystem;
use mesh::mesh_resource_provider_system;
use render_graph::RenderGraph;
use render_resource::EntitiesWaitingForAssets;
use texture::{PngTextureLoader, TextureResourceSystemState};
use std::ops::Range;

pub static RENDER_RESOURCE_STAGE: &str = "render_resource";
pub static RENDER_STAGE: &str = "render";

pub struct RenderPlugin {
    /// configures the "base render graph". If this is not `None`, the "base render graph" will be added  
    pub base_render_graph_config: Option<BaseRenderGraphConfig>,
}

impl Default for RenderPlugin {
    fn default() -> Self {
        RenderPlugin {
            base_render_graph_config: Some(BaseRenderGraphConfig::default()),
        }
    }
}

impl AppPlugin for RenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage_after(bevy_asset::stage::ASSET_EVENTS, RENDER_RESOURCE_STAGE)
            .add_stage_after(RENDER_RESOURCE_STAGE, RENDER_STAGE)
            .add_asset::<Mesh>()
            .add_asset::<Texture>()
            .add_asset::<Shader>()
            .add_asset::<PipelineDescriptor>()
            .add_asset_loader::<Texture, PngTextureLoader>()
            .register_component::<Camera>()
            .register_component::<Renderable>()
            .register_component::<ActiveCamera>()
            .register_component::<ActiveCamera2d>()
            .register_property_type::<Color>()
            .register_property_type::<CameraType>()
            .register_property_type::<Range<f32>>()
            .init_resource::<RenderGraph>()
            .init_resource::<PipelineAssignments>()
            .init_resource::<PipelineCompiler>()
            .init_resource::<RenderResourceAssignments>()
            .init_resource::<VertexBufferDescriptors>()
            .init_resource::<EntityRenderResourceAssignments>()
            .init_resource::<EntitiesWaitingForAssets>()
            .init_resource::<TextureResourceSystemState>()
            .add_system(entity_render_resource_assignments_system())
            .init_system_to_stage(stage::POST_UPDATE, camera::camera_update_system)
            .add_system_to_stage(
                stage::PRE_UPDATE,
                EntitiesWaitingForAssets::clear_system.system(),
            )
            .init_system_to_stage(RENDER_RESOURCE_STAGE, mesh_resource_provider_system)
            .add_system_to_stage(
                RENDER_RESOURCE_STAGE,
                Texture::texture_resource_system.system(),
            );

        if let Some(ref config) = self.base_render_graph_config {
            let resources = app.resources();
            let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
            render_graph.add_base_graph(resources, config);
        }
    }
}
