//! Layer system for Creative Engine
//!
//! Provides layer tree management, compositing, and layer operations.

use creative_domain::{
    BlendMode, Color, LayerId, Layer, ObjectId, Rect, Transform2D,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Layer errors
#[derive(Debug, Error)]
pub enum LayerError {
    #[error("Layer not found: {0}")]
    NotFound(LayerId),
    #[error("Layer is locked: {0}")]
    Locked(LayerId),
    #[error("Invalid layer hierarchy: {0}")]
    InvalidHierarchy(String),
    #[error("Cannot move layer into its own descendant: {0}")]
    CircularReference(LayerId),
    #[error("Layer name already exists: {0}")]
    NameConflict(String),
    #[error("Cannot delete root layer")]
    CannotDeleteRoot,
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Layer tree structure with compositing support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerTree {
    /// All layers indexed by ID
    layers: HashMap<LayerId, LayerNode>,
    /// Root layer ID (always exists)
    root_id: LayerId,
    /// Layer order (top to bottom) for rendering
    render_order: Vec<LayerId>,
    /// Layer groups for organization
    groups: HashMap<LayerId, LayerGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LayerNode {
    layer: Layer,
    /// Computed world transform from parent hierarchy
    world_transform: Transform2D,
    /// Whether this layer or any ancestor is hidden
    effectively_visible: bool,
    /// Whether this layer or any ancestor is locked
    effectively_locked: bool,
    /// Effective opacity (multiplied through hierarchy)
    effective_opacity: f32,
    /// Effective blend mode (topmost non-normal in hierarchy)
    effective_blend_mode: BlendMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerGroup {
    pub id: LayerId,
    pub name: String,
    pub expanded: bool,
    pub color: Option<Color>, // Group color tag
}

impl LayerTree {
    /// Create a new layer tree with a root layer
    pub fn new() -> Self {
        let root_id = LayerId::new();
        let mut layers = HashMap::new();
        let mut render_order = Vec::new();

        let root_layer = Layer {
            id: root_id,
            name: "Root".to_string(),
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            object_ids: Vec::new(),
            parent: None,
            children: Vec::new(),
        };

        let root_node = LayerNode {
            layer: root_layer.clone(),
            world_transform: Transform2D::IDENTITY,
            effectively_visible: true,
            effectively_locked: false,
            effective_opacity: 1.0,
            effective_blend_mode: BlendMode::Normal,
        };

        layers.insert(root_id, root_node);
        render_order.push(root_id);

        Self {
            layers,
            root_id,
            render_order,
            groups: HashMap::new(),
        }
    }

    /// Create a layer tree from an existing document's layers
    pub fn from_layers(layers: Vec<Layer>) -> Result<Self, LayerError> {
        let mut tree = Self::new();
        
        // Find or create root
        let _root_id = layers
            .iter()
            .find(|l| l.parent.is_none())
            .map(|l| l.id)
            .unwrap_or_else(|| tree.root_id);

        // Build hierarchy
        for layer in layers {
            tree.add_existing_layer(layer)?;
        }

        tree.rebuild_render_order();
        tree.update_effective_properties();
        Ok(tree)
    }

    /// Get the root layer ID
    pub fn root_id(&self) -> LayerId {
        self.root_id
    }

    /// Get a layer by ID
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.get(&id).map(|n| &n.layer)
    }

    /// Get mutable layer by ID
    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.get_mut(&id).map(|n| &mut n.layer)
    }

    /// Get layer node with computed properties
    #[allow(dead_code)]
    pub(crate) fn get_node(&self, id: LayerId) -> Option<&LayerNode> {
        self.layers.get(&id)
    }

    /// Get all layers (including root)
    pub fn all_layers(&self) -> Vec<&Layer> {
        self.layers.values().map(|n| &n.layer).collect()
    }

    /// Get all layer IDs in render order (bottom to top)
    pub fn render_order(&self) -> &[LayerId] {
        &self.render_order
    }

    /// Get layers in render order with their effective properties
    pub fn render_layers(&self) -> Vec<RenderLayer> {
        self.render_order
            .iter()
            .filter_map(|id| self.layers.get(id))
            .map(|node| RenderLayer {
                id: node.layer.id,
                name: node.layer.name.clone(),
                visible: node.effectively_visible,
                opacity: node.effective_opacity,
                blend_mode: node.effective_blend_mode,
                world_transform: node.world_transform,
                object_ids: node.layer.object_ids.clone(),
                locked: node.effectively_locked,
            })
            .collect()
    }

    /// Get children of a layer
    pub fn children(&self, parent: LayerId) -> Vec<LayerId> {
        self.layers
            .get(&parent)
            .map(|n| n.layer.children.clone())
            .unwrap_or_default()
    }

    /// Get parent of a layer
    pub fn parent(&self, id: LayerId) -> Option<LayerId> {
        self.layers.get(&id).and_then(|n| n.layer.parent)
    }

    /// Get ancestors of a layer (from parent to root)
    pub fn ancestors(&self, id: LayerId) -> Vec<LayerId> {
        let mut result = Vec::new();
        let mut current = self.parent(id);
        while let Some(parent_id) = current {
            result.push(parent_id);
            current = self.parent(parent_id);
        }
        result
    }

    /// Get descendants of a layer (all children recursively)
    pub fn descendants(&self, id: LayerId) -> Vec<LayerId> {
        let mut result = Vec::new();
        self.collect_descendants(id, &mut result);
        result
    }

    fn collect_descendants(&self, id: LayerId, result: &mut Vec<LayerId>) {
        if let Some(node) = self.layers.get(&id) {
            for child_id in &node.layer.children {
                result.push(*child_id);
                self.collect_descendants(*child_id, result);
            }
        }
    }

    /// Check if a layer is a descendant of another
    pub fn is_descendant(&self, potential_descendant: LayerId, potential_ancestor: LayerId) -> bool {
        let mut current = self.parent(potential_descendant);
        while let Some(parent_id) = current {
            if parent_id == potential_ancestor {
                return true;
            }
            current = self.parent(parent_id);
        }
        false
    }

    /// Add a new layer as child of parent
    pub fn add_layer(&mut self, parent: LayerId, name: impl Into<String>) -> Result<LayerId, LayerError> {
        let name_str = name.into();
        
        // Check parent exists and is not locked
        let parent_node = self.layers.get(&parent).ok_or(LayerError::NotFound(parent))?;
        if parent_node.layer.locked || parent_node.effectively_locked {
            return Err(LayerError::Locked(parent));
        }

        // Check name uniqueness among siblings
        for child_id in &parent_node.layer.children {
            if let Some(child) = self.layers.get(child_id) {
                if child.layer.name == name_str {
                    return Err(LayerError::NameConflict(name_str));
                }
            }
        }

        let new_id = LayerId::new();
        let new_layer = Layer {
            id: new_id,
            name: name_str,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            object_ids: Vec::new(),
            parent: Some(parent),
            children: Vec::new(),
        };

        let new_node = LayerNode {
            layer: new_layer.clone(),
            world_transform: Transform2D::IDENTITY,
            effectively_visible: true,
            effectively_locked: false,
            effective_opacity: 1.0,
            effective_blend_mode: BlendMode::Normal,
        };

        // Add to parent's children
        if let Some(parent_node) = self.layers.get_mut(&parent) {
            parent_node.layer.children.push(new_id);
        }

        // Insert at top of render order (after parent's subtree)
        let insert_idx = self.find_render_insert_index(parent);
        self.render_order.insert(insert_idx, new_id);

        self.layers.insert(new_id, new_node);
        self.update_effective_properties();

        Ok(new_id)
    }

    /// Add an existing layer (for loading documents)
    fn add_existing_layer(&mut self, layer: Layer) -> Result<(), LayerError> {
        let id = layer.id;
        
        // Check for circular reference
        if let Some(parent_id) = layer.parent {
            if self.is_descendant(parent_id, id) {
                return Err(LayerError::CircularReference(id));
            }
        }

        let node = LayerNode {
            layer,
            world_transform: Transform2D::IDENTITY,
            effectively_visible: true,
            effectively_locked: false,
            effective_opacity: 1.0,
            effective_blend_mode: BlendMode::Normal,
        };

        self.layers.insert(id, node);
        
        // Add to render order if not root
        if id != self.root_id && !self.render_order.contains(&id) {
            self.render_order.push(id);
        }

        Ok(())
    }

    /// Find index to insert new layer in render order (after parent's subtree)
    fn find_render_insert_index(&self, parent: LayerId) -> usize {
        // Find the last descendant of parent in render order
        let descendants = self.descendants(parent);
        let mut last_idx = self.render_order.iter().position(|&id| id == parent).unwrap_or(0);
        
        for desc in descendants {
            if let Some(idx) = self.render_order.iter().position(|&id| id == desc) {
                last_idx = last_idx.max(idx);
            }
        }
        
        last_idx + 1
    }

    /// Delete a layer and all its descendants
    pub fn delete_layer(&mut self, id: LayerId) -> Result<Vec<LayerId>, LayerError> {
        if id == self.root_id {
            return Err(LayerError::CannotDeleteRoot);
        }

        // Collect all descendants to delete first (before borrowing mutably)
        let descendants = self.descendants(id);
        let mut to_delete = vec![id];
        to_delete.extend(descendants);

        // Check if locked (need to borrow)
        if let Some(node) = self.layers.get(&id) {
            if node.layer.locked || node.effectively_locked {
                return Err(LayerError::Locked(id));
            }
            
            // Remove from parent's children
            if let Some(parent_id) = node.layer.parent {
                if let Some(parent_node) = self.layers.get_mut(&parent_id) {
                    parent_node.layer.children.retain(|&c| c != id);
                }
            }
        } else {
            return Err(LayerError::NotFound(id));
        }

        // Remove from render order and layers
        for del_id in &to_delete {
            self.render_order.retain(|&r| r != *del_id);
            self.layers.remove(del_id);
        }

        self.update_effective_properties();
        Ok(to_delete)
    }

    /// Move layer to new parent
    pub fn move_layer(&mut self, id: LayerId, new_parent: LayerId) -> Result<(), LayerError> {
        if id == self.root_id {
            return Err(LayerError::InvalidOperation("Cannot move root layer".to_string()));
        }
        if id == new_parent {
            return Err(LayerError::InvalidOperation("Cannot move layer into itself".to_string()));
        }
        if self.is_descendant(new_parent, id) {
            return Err(LayerError::CircularReference(id));
        }

        // Check locks first (immutable borrow)
        let (locked, effectively_locked, old_parent_id) = {
            let node = self.layers.get(&id).ok_or(LayerError::NotFound(id))?;
            (node.layer.locked, node.effectively_locked, node.layer.parent)
        };
        
        if locked || effectively_locked {
            return Err(LayerError::Locked(id));
        }

        let new_parent_locked = {
            let new_parent_node = self.layers.get(&new_parent).ok_or(LayerError::NotFound(new_parent))?;
            new_parent_node.layer.locked || new_parent_node.effectively_locked
        };
        
        if new_parent_locked {
            return Err(LayerError::Locked(new_parent));
        }

        // Remove from old parent
        if let Some(old_parent_id) = old_parent_id {
            if let Some(old_parent) = self.layers.get_mut(&old_parent_id) {
                old_parent.layer.children.retain(|&c| c != id);
            }
        }

        // Add to new parent
        if let Some(new_parent_mut) = self.layers.get_mut(&new_parent) {
            new_parent_mut.layer.children.push(id);
        }

        // Update layer's parent
        if let Some(layer_mut) = self.layers.get_mut(&id) {
            layer_mut.layer.parent = Some(new_parent);
        }

        // Reorder in render order
        self.render_order.retain(|&r| r != id);
        let insert_idx = self.find_render_insert_index(new_parent);
        self.render_order.insert(insert_idx, id);

        // Also move all descendants in render order
        let descendants = self.descendants(id);
        for desc in &descendants {
            self.render_order.retain(|&r| r != *desc);
        }
        let mut insert_idx = insert_idx;
        for desc in descendants {
            self.render_order.insert(insert_idx, desc);
            insert_idx += 1;
        }

        self.update_effective_properties();
        Ok(())
    }

    /// Reorder layer within its parent (change z-index)
    pub fn reorder_layer(&mut self, id: LayerId, new_index: usize) -> Result<(), LayerError> {
        // Check lock first
        let (locked, effectively_locked, parent_id) = {
            let node = self.layers.get(&id).ok_or(LayerError::NotFound(id))?;
            (node.layer.locked, node.effectively_locked, node.layer.parent)
        };
        
        if locked || effectively_locked {
            return Err(LayerError::Locked(id));
        }

        let parent_id = parent_id.ok_or(LayerError::InvalidOperation("Root layer cannot be reordered".to_string()))?;
        
        // Update parent's children list
        if let Some(parent_node) = self.layers.get_mut(&parent_id) {
            let children = &mut parent_node.layer.children;
            if let Some(pos) = children.iter().position(|&c| c == id) {
                children.remove(pos);
                let clamped_index = new_index.clamp(0, children.len());
                children.insert(clamped_index, id);
            }
        }
        
        let siblings = self.children(parent_id);
        let clamped_index = new_index.clamp(0, siblings.len().saturating_sub(1));
        
        // Remove from render order
        self.render_order.retain(|&r| r != id);
        
        // Find insert position based on sibling order
        let sibling_ids: Vec<LayerId> = siblings.into_iter()
            .filter(|&s| s != id)
            .collect();
        
        let insert_idx = if clamped_index < sibling_ids.len() {
            self.render_order.iter().position(|&r| r == sibling_ids[clamped_index]).unwrap_or(self.render_order.len())
        } else {
            // Insert after last sibling
            let last_sibling = sibling_ids.last().copied().unwrap_or(parent_id);
            self.find_render_insert_index(last_sibling)
        };
        
        self.render_order.insert(insert_idx, id);
        
        // Also move descendants
        let descendants = self.descendants(id);
        for desc in &descendants {
            self.render_order.retain(|&r| r != *desc);
        }
        let mut curr_idx = insert_idx + 1;
        for desc in descendants {
            self.render_order.insert(curr_idx, desc);
            curr_idx += 1;
        }

        Ok(())
    }

    /// Set layer visibility
    pub fn set_visibility(&mut self, id: LayerId, visible: bool) -> Result<(), LayerError> {
        let node = self.layers.get_mut(&id).ok_or(LayerError::NotFound(id))?;
        if node.layer.locked || node.effectively_locked {
            return Err(LayerError::Locked(id));
        }
        node.layer.visible = visible;
        self.update_effective_properties();
        Ok(())
    }

    /// Set layer lock state
    pub fn set_locked(&mut self, id: LayerId, locked: bool) -> Result<(), LayerError> {
        let node = self.layers.get_mut(&id).ok_or(LayerError::NotFound(id))?;
        node.layer.locked = locked;
        self.update_effective_properties();
        Ok(())
    }

    /// Set layer opacity
    pub fn set_opacity(&mut self, id: LayerId, opacity: f32) -> Result<(), LayerError> {
        let node = self.layers.get_mut(&id).ok_or(LayerError::NotFound(id))?;
        if node.layer.locked || node.effectively_locked {
            return Err(LayerError::Locked(id));
        }
        node.layer.opacity = opacity.clamp(0.0, 1.0);
        self.update_effective_properties();
        Ok(())
    }

    /// Set layer blend mode
    pub fn set_blend_mode(&mut self, id: LayerId, mode: BlendMode) -> Result<(), LayerError> {
        let node = self.layers.get_mut(&id).ok_or(LayerError::NotFound(id))?;
        if node.layer.locked || node.effectively_locked {
            return Err(LayerError::Locked(id));
        }
        node.layer.blend_mode = mode;
        self.update_effective_properties();
        Ok(())
    }

    /// Rename layer
    pub fn rename_layer(&mut self, id: LayerId, new_name: impl Into<String>) -> Result<(), LayerError> {
        let new_name_str = new_name.into();
        
        // Check lock and get parent first
        let (locked, effectively_locked, parent_id) = {
            let node = self.layers.get(&id).ok_or(LayerError::NotFound(id))?;
            (node.layer.locked, node.effectively_locked, node.layer.parent)
        };
        
        if locked || effectively_locked {
            return Err(LayerError::Locked(id));
        }

        // Check name uniqueness among siblings
        if let Some(parent_id) = parent_id {
            let siblings = self.children(parent_id);
            for child_id in siblings {
                if child_id != id {
                    if let Some(sibling) = self.layers.get(&child_id) {
                        if sibling.layer.name == new_name_str {
                            return Err(LayerError::NameConflict(new_name_str));
                        }
                    }
                }
            }
        }

        // Now do the rename
        if let Some(node) = self.layers.get_mut(&id) {
            node.layer.name = new_name_str;
        }
        Ok(())
    }

    /// Add object to layer
    pub fn add_object(&mut self, layer_id: LayerId, object_id: ObjectId) -> Result<(), LayerError> {
        let node = self.layers.get_mut(&layer_id).ok_or(LayerError::NotFound(layer_id))?;
        if node.layer.locked || node.effectively_locked {
            return Err(LayerError::Locked(layer_id));
        }
        if !node.layer.object_ids.contains(&object_id) {
            node.layer.object_ids.push(object_id);
        }
        Ok(())
    }

    /// Remove object from layer
    pub fn remove_object(&mut self, layer_id: LayerId, object_id: ObjectId) -> Result<(), LayerError> {
        let node = self.layers.get_mut(&layer_id).ok_or(LayerError::NotFound(layer_id))?;
        if node.layer.locked || node.effectively_locked {
            return Err(LayerError::Locked(layer_id));
        }
        node.layer.object_ids.retain(|&o| o != object_id);
        Ok(())
    }

    /// Move object between layers
    pub fn move_object(&mut self, object_id: ObjectId, from_layer: LayerId, to_layer: LayerId) -> Result<(), LayerError> {
        self.remove_object(from_layer, object_id)?;
        self.add_object(to_layer, object_id)?;
        Ok(())
    }

    /// Create a layer group
    pub fn create_group(&mut self, parent: LayerId, name: impl Into<String>, color: Option<Color>) -> Result<LayerId, LayerError> {
        let group_id = self.add_layer(parent, name)?;
        
        let group = LayerGroup {
            id: group_id,
            name: self.layers[&group_id].layer.name.clone(),
            expanded: true,
            color,
        };
        
        self.groups.insert(group_id, group);
        Ok(group_id)
    }

    /// Get group info
    pub fn get_group(&self, id: LayerId) -> Option<&LayerGroup> {
        self.groups.get(&id)
    }

    /// Set group expanded state
    pub fn set_group_expanded(&mut self, id: LayerId, expanded: bool) -> Result<(), LayerError> {
        let group = self.groups.get_mut(&id).ok_or(LayerError::NotFound(id))?;
        group.expanded = expanded;
        Ok(())
    }

    /// Set group color tag
    pub fn set_group_color(&mut self, id: LayerId, color: Option<Color>) -> Result<(), LayerError> {
        let group = self.groups.get_mut(&id).ok_or(LayerError::NotFound(id))?;
        group.color = color;
        Ok(())
    }

    /// Duplicate a layer (with all descendants)
    pub fn duplicate_layer(&mut self, id: LayerId) -> Result<LayerId, LayerError> {
        // Get properties we need before mutable borrow
        let (name, parent_id, opacity, blend_mode, visible, locked, children) = {
            let node = self.layers.get(&id).ok_or(LayerError::NotFound(id))?;
            (
                node.layer.name.clone(),
                node.layer.parent.ok_or(LayerError::InvalidOperation("Cannot duplicate root layer".to_string()))?,
                node.layer.opacity,
                node.layer.blend_mode,
                node.layer.visible,
                node.layer.locked,
                node.layer.children.clone(),
            )
        };

        let new_name = format!("{} copy", name);
        let new_id = self.add_layer(parent_id, &new_name)?;
        
        // Copy properties
        if let Some(new_node) = self.layers.get_mut(&new_id) {
            new_node.layer.opacity = opacity;
            new_node.layer.blend_mode = blend_mode;
            new_node.layer.visible = visible;
            new_node.layer.locked = locked;
        }

        // Duplicate descendants recursively
        for child_id in children {
            self.duplicate_subtree(child_id, new_id)?;
        }

        Ok(new_id)
    }

    fn duplicate_subtree(&mut self, id: LayerId, new_parent: LayerId) -> Result<LayerId, LayerError> {
        let (name, opacity, blend_mode, visible, locked, children) = {
            let node = self.layers.get(&id).ok_or(LayerError::NotFound(id))?;
            (
                node.layer.name.clone(),
                node.layer.opacity,
                node.layer.blend_mode,
                node.layer.visible,
                node.layer.locked,
                node.layer.children.clone(),
            )
        };

        let new_id = self.add_layer(new_parent, &name)?;
        
        if let Some(new_node) = self.layers.get_mut(&new_id) {
            new_node.layer.opacity = opacity;
            new_node.layer.blend_mode = blend_mode;
            new_node.layer.visible = visible;
            new_node.layer.locked = locked;
        }

        for child_id in children {
            self.duplicate_subtree(child_id, new_id)?;
        }

        Ok(new_id)
    }

    /// Merge layer down into the layer below it
    pub fn merge_down(&mut self, id: LayerId) -> Result<LayerId, LayerError> {
        let node = self.layers.get(&id).ok_or(LayerError::NotFound(id))?;
        let parent_id = node.layer.parent.ok_or(LayerError::InvalidOperation("Cannot merge root layer".to_string()))?;
        let siblings = self.children(parent_id);
        
        // Find index of this layer among siblings
        let idx = siblings.iter().position(|&s| s == id).ok_or(LayerError::InvalidOperation("Layer not in parent".to_string()))?;
        
        if idx == 0 {
            return Err(LayerError::InvalidOperation("No layer below to merge into".to_string()));
        }
        
        let target_id = siblings[idx - 1];
        
        // Move objects from this layer to target
        let objects: Vec<ObjectId> = {
            let node = self.layers.get(&id).unwrap();
            node.layer.object_ids.clone()
        };
        
        for obj in objects {
            self.move_object(obj, id, target_id)?;
        }
        
        // Delete this layer
        self.delete_layer(id)?;
        
        Ok(target_id)
    }

    /// Flatten all visible layers into a new layer
    pub fn flatten_visible(&mut self, name: impl Into<String>) -> Result<LayerId, LayerError> {
        let visible_layers: Vec<LayerId> = self.render_layers()
            .into_iter()
            .filter(|rl| rl.visible && rl.id != self.root_id)
            .map(|rl| rl.id)
            .collect();
        
        if visible_layers.is_empty() {
            return Err(LayerError::InvalidOperation("No visible layers to flatten".to_string()));
        }
        
        let new_id = self.add_layer(self.root_id, name)?;
        
        // Collect all objects from visible layers
        for layer_id in &visible_layers {
            let objects: Vec<ObjectId> = {
                let node = self.layers.get(layer_id).unwrap();
                node.layer.object_ids.clone()
            };
            
            for obj in objects {
                self.move_object(obj, *layer_id, new_id)?;
            }
        }
        
        // Hide original visible layers
        for layer_id in &visible_layers {
            self.set_visibility(*layer_id, false)?;
        }
        
        Ok(new_id)
    }

    /// Rebuild render order from hierarchy
    pub fn rebuild_render_order(&mut self) {
        self.render_order.clear();
        self.render_order.push(self.root_id);
        self.build_render_order_recursive(self.root_id);
    }

    fn build_render_order_recursive(&mut self, parent: LayerId) {
        let children = self.children(parent);
        for child in children {
            self.render_order.push(child);
            self.build_render_order_recursive(child);
        }
    }

    /// Update all effective properties (visibility, lock, opacity, blend mode, transform)
    pub fn update_effective_properties(&mut self) {
        // Start from root
        if let Some(root) = self.layers.get_mut(&self.root_id) {
            root.effectively_visible = root.layer.visible;
            root.effectively_locked = root.layer.locked;
            root.effective_opacity = root.layer.opacity;
            root.effective_blend_mode = root.layer.blend_mode;
            root.world_transform = Transform2D::IDENTITY;
        }
        
        self.update_effective_recursive(self.root_id);
    }

    fn update_effective_recursive(&mut self, parent: LayerId) {
        let (parent_visible, parent_locked, parent_opacity, parent_blend, parent_transform) = {
            let parent_node = self.layers.get(&parent).unwrap();
            (
                parent_node.effectively_visible,
                parent_node.effectively_locked,
                parent_node.effective_opacity,
                parent_node.effective_blend_mode,
                parent_node.world_transform,
            )
        };
        
        let children = self.children(parent);
        for child_id in children {
            if let Some(child) = self.layers.get_mut(&child_id) {
                child.effectively_visible = parent_visible && child.layer.visible;
                child.effectively_locked = parent_locked || child.layer.locked;
                child.effective_opacity = parent_opacity * child.layer.opacity;
                child.effective_blend_mode = if child.layer.blend_mode != BlendMode::Normal {
                    child.layer.blend_mode
                } else {
                    parent_blend
                };
                child.world_transform = parent_transform; // No local transform for layers yet
            }
            self.update_effective_recursive(child_id);
        }
    }

    /// Get layer bounds (union of all object bounds - placeholder)
    pub fn layer_bounds(&self, _id: LayerId) -> Option<Rect> {
        // This would need access to object geometry
        // For now return None
        None
    }

    /// Get layer count (excluding root)
    pub fn layer_count(&self) -> usize {
        self.layers.len().saturating_sub(1)
    }

    /// Check if layer is empty (no objects, no children)
    pub fn is_empty(&self, id: LayerId) -> bool {
        self.layers.get(&id).map(|n| n.layer.object_ids.is_empty() && n.layer.children.is_empty()).unwrap_or(true)
    }

    /// Export layer tree to flat list for serialization
    pub fn to_layers(&self) -> Vec<Layer> {
        self.layers.values().map(|n| n.layer.clone()).collect()
    }
}

/// Layer info for rendering
#[derive(Debug, Clone)]
pub struct RenderLayer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub world_transform: Transform2D,
    pub object_ids: Vec<ObjectId>,
    pub locked: bool,
}

/// Layer operations for undo/redo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerOperation {
    AddLayer { parent: LayerId, layer: Layer },
    DeleteLayer { layer: Layer, parent: LayerId, index: usize },
    MoveLayer { id: LayerId, old_parent: LayerId, new_parent: LayerId, old_index: usize },
    ReorderLayer { id: LayerId, old_index: usize, new_index: usize },
    SetVisibility { id: LayerId, old: bool, new: bool },
    SetLocked { id: LayerId, old: bool, new: bool },
    SetOpacity { id: LayerId, old: f32, new: f32 },
    SetBlendMode { id: LayerId, old: BlendMode, new: BlendMode },
    RenameLayer { id: LayerId, old: String, new: String },
    AddObject { layer_id: LayerId, object_id: ObjectId },
    RemoveObject { layer_id: LayerId, object_id: ObjectId },
    MoveObject { object_id: ObjectId, from_layer: LayerId, to_layer: LayerId },
    CreateGroup { group: LayerGroup },
    SetGroupExpanded { id: LayerId, expanded: bool },
    SetGroupColor { id: LayerId, color: Option<Color> },
    DuplicateLayer { original: LayerId, duplicate: Layer },
    MergeDown { upper: LayerId, lower: LayerId, moved_objects: Vec<ObjectId> },
    FlattenVisible { new_layer: Layer, hidden_layers: Vec<LayerId> },
}

/// Apply a layer operation
pub fn apply_layer_operation(tree: &mut LayerTree, op: &LayerOperation) -> Result<(), LayerError> {
    match op {
        LayerOperation::AddLayer { parent, layer } => {
            tree.add_existing_layer(layer.clone())?;
            if let Some(parent_node) = tree.layers.get_mut(parent) {
                parent_node.layer.children.push(layer.id);
            }
        }
        LayerOperation::DeleteLayer { layer, parent: _, index: _ } => {
            tree.delete_layer(layer.id)?;
        }
        LayerOperation::MoveLayer { id, old_parent: _, new_parent, old_index: _ } => {
            tree.move_layer(*id, *new_parent)?;
        }
        LayerOperation::ReorderLayer { id, old_index: _, new_index } => {
            tree.reorder_layer(*id, *new_index)?;
        }
        LayerOperation::SetVisibility { id, old: _, new } => {
            tree.set_visibility(*id, *new)?;
        }
        LayerOperation::SetLocked { id, old: _, new } => {
            tree.set_locked(*id, *new)?;
        }
        LayerOperation::SetOpacity { id, old: _, new } => {
            tree.set_opacity(*id, *new)?;
        }
        LayerOperation::SetBlendMode { id, old: _, new } => {
            tree.set_blend_mode(*id, *new)?;
        }
        LayerOperation::RenameLayer { id, old: _, new } => {
            tree.rename_layer(*id, new.clone())?;
        }
        LayerOperation::AddObject { layer_id, object_id } => {
            tree.add_object(*layer_id, *object_id)?;
        }
        LayerOperation::RemoveObject { layer_id, object_id } => {
            tree.remove_object(*layer_id, *object_id)?;
        }
        LayerOperation::MoveObject { object_id, from_layer, to_layer } => {
            tree.move_object(*object_id, *from_layer, *to_layer)?;
        }
        LayerOperation::CreateGroup { group } => {
            tree.groups.insert(group.id, group.clone());
        }
        LayerOperation::SetGroupExpanded { id, expanded } => {
            tree.set_group_expanded(*id, *expanded)?;
        }
        LayerOperation::SetGroupColor { id, color } => {
            tree.set_group_color(*id, *color)?;
        }
        LayerOperation::DuplicateLayer { original: _, duplicate } => {
            tree.add_existing_layer(duplicate.clone())?;
        }
        LayerOperation::MergeDown { upper, lower, moved_objects } => {
            for obj in moved_objects {
                tree.move_object(*obj, *upper, *lower)?;
            }
            tree.delete_layer(*upper)?;
        }
        LayerOperation::FlattenVisible { new_layer, hidden_layers } => {
            tree.add_existing_layer(new_layer.clone())?;
            for id in hidden_layers {
                tree.set_visibility(*id, false)?;
            }
        }
    }
    Ok(())
}

/// Reverse a layer operation for undo
pub fn reverse_layer_operation(op: &LayerOperation) -> LayerOperation {
    match op {
        LayerOperation::AddLayer { parent, layer } => {
            LayerOperation::DeleteLayer {
                layer: layer.clone(),
                parent: *parent,
                index: 0, // Would need actual index
            }
        }
        LayerOperation::DeleteLayer { layer, parent, index: _ } => {
            LayerOperation::AddLayer {
                parent: *parent,
                layer: layer.clone(),
            }
        }
        LayerOperation::MoveLayer { id, old_parent, new_parent, old_index } => {
            LayerOperation::MoveLayer {
                id: *id,
                old_parent: *new_parent,
                new_parent: *old_parent,
                old_index: *old_index,
            }
        }
        LayerOperation::ReorderLayer { id, old_index, new_index } => {
            LayerOperation::ReorderLayer {
                id: *id,
                old_index: *new_index,
                new_index: *old_index,
            }
        }
        LayerOperation::SetVisibility { id, old, new } => {
            LayerOperation::SetVisibility {
                id: *id,
                old: *new,
                new: *old,
            }
        }
        LayerOperation::SetLocked { id, old, new } => {
            LayerOperation::SetLocked {
                id: *id,
                old: *new,
                new: *old,
            }
        }
        LayerOperation::SetOpacity { id, old, new } => {
            LayerOperation::SetOpacity {
                id: *id,
                old: *new,
                new: *old,
            }
        }
        LayerOperation::SetBlendMode { id, old, new } => {
            LayerOperation::SetBlendMode {
                id: *id,
                old: *new,
                new: *old,
            }
        }
        LayerOperation::RenameLayer { id, old, new } => {
            LayerOperation::RenameLayer {
                id: *id,
                old: new.clone(),
                new: old.clone(),
            }
        }
        LayerOperation::AddObject { layer_id, object_id } => {
            LayerOperation::RemoveObject {
                layer_id: *layer_id,
                object_id: *object_id,
            }
        }
        LayerOperation::RemoveObject { layer_id, object_id } => {
            LayerOperation::AddObject {
                layer_id: *layer_id,
                object_id: *object_id,
            }
        }
        LayerOperation::MoveObject { object_id, from_layer, to_layer } => {
            LayerOperation::MoveObject {
                object_id: *object_id,
                from_layer: *to_layer,
                to_layer: *from_layer,
            }
        }
        LayerOperation::CreateGroup { group } => {
            // Delete group would be complex, skip for now
            LayerOperation::SetGroupExpanded {
                id: group.id,
                expanded: !group.expanded,
            }
        }
        LayerOperation::SetGroupExpanded { id, expanded } => {
            LayerOperation::SetGroupExpanded {
                id: *id,
                expanded: !expanded,
            }
        }
        LayerOperation::SetGroupColor { id, color } => {
            LayerOperation::SetGroupColor {
                id: *id,
                color: *color,
            }
        }
        LayerOperation::DuplicateLayer { original, duplicate: _ } => {
            LayerOperation::DeleteLayer {
                layer: Layer::new(""), // Would need actual layer
                parent: *original,
                index: 0,
            }
        }
        LayerOperation::MergeDown { upper, lower, moved_objects: _ } => {
            LayerOperation::MoveLayer {
                id: *upper,
                old_parent: *lower,
                new_parent: *lower,
                old_index: 0,
            }
        }
        LayerOperation::FlattenVisible { new_layer, hidden_layers: _ } => {
            // Complex to reverse, simplified
            LayerOperation::DeleteLayer {
                layer: new_layer.clone(),
                parent: new_layer.parent.unwrap_or_else(LayerId::new),
                index: 0,
            }
        }
    }
}

impl Default for LayerTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::{Color, LayerId, ObjectId};

    #[test]
    fn test_layer_tree_creation() {
        let tree = LayerTree::new();
        assert_eq!(tree.layer_count(), 0);
        assert!(tree.get_layer(tree.root_id()).is_some());
    }

    #[test]
    fn test_add_layer() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer1 = tree.add_layer(root, "Layer 1").unwrap();
        assert_eq!(tree.layer_count(), 1);
        assert_eq!(tree.get_layer(layer1).unwrap().name, "Layer 1");
        assert_eq!(tree.get_layer(layer1).unwrap().parent, Some(root));
    }

    #[test]
    fn test_nested_layers() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let group = tree.add_layer(root, "Group").unwrap();
        let child = tree.add_layer(group, "Child").unwrap();
        
        assert_eq!(tree.get_layer(child).unwrap().parent, Some(group));
        assert!(tree.is_descendant(child, group));
        assert!(tree.is_descendant(child, root));
        assert!(!tree.is_descendant(group, child));
    }

    #[test]
    fn test_delete_layer() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer1 = tree.add_layer(root, "Layer 1").unwrap();
        let layer2 = tree.add_layer(root, "Layer 2").unwrap();
        
        tree.delete_layer(layer1).unwrap();
        assert_eq!(tree.layer_count(), 1);
        assert!(tree.get_layer(layer1).is_none());
        assert!(tree.get_layer(layer2).is_some());
    }

    #[test]
    fn test_delete_layer_with_children() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let group = tree.add_layer(root, "Group").unwrap();
        let child = tree.add_layer(group, "Child").unwrap();
        
        tree.delete_layer(group).unwrap();
        assert_eq!(tree.layer_count(), 0);
        assert!(tree.get_layer(group).is_none());
        assert!(tree.get_layer(child).is_none());
    }

    #[test]
    fn test_move_layer() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let group1 = tree.add_layer(root, "Group 1").unwrap();
        let group2 = tree.add_layer(root, "Group 2").unwrap();
        let layer = tree.add_layer(group1, "Layer").unwrap();
        
        tree.move_layer(layer, group2).unwrap();
        assert_eq!(tree.get_layer(layer).unwrap().parent, Some(group2));
        assert!(tree.is_descendant(layer, group2));
    }

    #[test]
    fn test_move_layer_circular_ref() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let group = tree.add_layer(root, "Group").unwrap();
        let child = tree.add_layer(group, "Child").unwrap();
        
        // Try to move group into its child - should fail
        let result = tree.move_layer(group, child);
        assert!(result.is_err());
    }

    #[test]
    fn test_reorder_layer() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer1 = tree.add_layer(root, "Layer 1").unwrap();
        let _layer2 = tree.add_layer(root, "Layer 2").unwrap();
        let _layer3 = tree.add_layer(root, "Layer 3").unwrap();
        
        // Initial order: layer1, layer2, layer3 (bottom to top)
        // Move layer1 to top (index 2)
        tree.reorder_layer(layer1, 2).unwrap();
        
        let children = tree.children(root);
        assert_eq!(children[2], layer1);
    }

    #[test]
    fn test_visibility() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer = tree.add_layer(root, "Layer").unwrap();
        
        tree.set_visibility(layer, false).unwrap();
        assert!(!tree.get_node(layer).unwrap().effectively_visible);
        
        // Child should also be hidden
        let child = tree.add_layer(layer, "Child").unwrap();
        assert!(!tree.get_node(child).unwrap().effectively_visible);
        
        // Parent visible again
        tree.set_visibility(layer, true).unwrap();
        assert!(tree.get_node(layer).unwrap().effectively_visible);
        assert!(tree.get_node(child).unwrap().effectively_visible);
    }

    #[test]
    fn test_locked() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer = tree.add_layer(root, "Layer").unwrap();
        
        tree.set_locked(layer, true).unwrap();
        assert!(tree.get_node(layer).unwrap().effectively_locked);
        
        // Can't add child to locked layer
        let result = tree.add_layer(layer, "Child");
        assert!(result.is_err());
        
        // Can't add to locked layer
        let result = tree.add_layer(layer, "New Layer");
        assert!(result.is_err());
    }

    #[test]
    fn test_opacity() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let parent = tree.add_layer(root, "Parent").unwrap();
        tree.set_opacity(parent, 0.5).unwrap();
        
        let child = tree.add_layer(parent, "Child").unwrap();
        tree.set_opacity(child, 0.5).unwrap();
        
        // Effective opacity = 0.5 * 0.5 = 0.25
        assert!((tree.get_node(child).unwrap().effective_opacity - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_blend_mode() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let parent = tree.add_layer(root, "Parent").unwrap();
        tree.set_blend_mode(parent, BlendMode::Multiply).unwrap();
        
        let child = tree.add_layer(parent, "Child").unwrap();
        // Child has normal, should inherit parent's multiply
        assert_eq!(tree.get_node(child).unwrap().effective_blend_mode, BlendMode::Multiply);
        
        // Child has own blend mode
        tree.set_blend_mode(child, BlendMode::Screen).unwrap();
        assert_eq!(tree.get_node(child).unwrap().effective_blend_mode, BlendMode::Screen);
    }

    #[test]
    fn test_rename_layer() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer = tree.add_layer(root, "Original").unwrap();
        tree.rename_layer(layer, "Renamed").unwrap();
        
        assert_eq!(tree.get_layer(layer).unwrap().name, "Renamed");
    }

    #[test]
    fn test_rename_conflict() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let _layer1 = tree.add_layer(root, "Layer 1").unwrap();
        let layer2 = tree.add_layer(root, "Layer 2").unwrap();
        
        // Try to rename layer2 to "Layer 1" - should fail
        let result = tree.rename_layer(layer2, "Layer 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_object_management() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer = tree.add_layer(root, "Layer").unwrap();
        let obj1 = ObjectId::new();
        let obj2 = ObjectId::new();
        
        tree.add_object(layer, obj1).unwrap();
        tree.add_object(layer, obj2).unwrap();
        
        let layer_ref = tree.get_layer(layer).unwrap();
        assert_eq!(layer_ref.object_ids.len(), 2);
        assert!(layer_ref.object_ids.contains(&obj1));
        
        tree.remove_object(layer, obj1).unwrap();
        assert_eq!(tree.get_layer(layer).unwrap().object_ids.len(), 1);
    }

    #[test]
    fn test_move_object() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer1 = tree.add_layer(root, "Layer 1").unwrap();
        let layer2 = tree.add_layer(root, "Layer 2").unwrap();
        let obj = ObjectId::new();
        
        tree.add_object(layer1, obj).unwrap();
        tree.move_object(obj, layer1, layer2).unwrap();
        
        assert!(tree.get_layer(layer1).unwrap().object_ids.is_empty());
        assert!(tree.get_layer(layer2).unwrap().object_ids.contains(&obj));
    }

    #[test]
    fn test_groups() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let group_id = tree.create_group(root, "My Group", Some(Color::RED)).unwrap();
        
        let group = tree.get_group(group_id).unwrap();
        assert_eq!(group.name, "My Group");
        assert!(group.expanded);
        assert_eq!(group.color, Some(Color::RED));
        
        tree.set_group_expanded(group_id, false).unwrap();
        assert!(!tree.get_group(group_id).unwrap().expanded);
        
        tree.set_group_color(group_id, Some(Color::BLUE)).unwrap();
        assert_eq!(tree.get_group(group_id).unwrap().color, Some(Color::BLUE));
    }

    #[test]
    fn test_duplicate_layer() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer = tree.add_layer(root, "Original").unwrap();
        tree.set_opacity(layer, 0.7).unwrap();
        tree.set_blend_mode(layer, BlendMode::Multiply).unwrap();
        
        let _child = tree.add_layer(layer, "Child").unwrap();
        
        let dup_id = tree.duplicate_layer(layer).unwrap();
        
        let dup = tree.get_layer(dup_id).unwrap();
        assert_eq!(dup.name, "Original copy");
        assert!((dup.opacity - 0.7).abs() < 0.001);
        assert_eq!(dup.blend_mode, BlendMode::Multiply);
        
        // Child should also be duplicated
        let dup_children = tree.children(dup_id);
        assert_eq!(dup_children.len(), 1);
        assert_eq!(tree.get_layer(dup_children[0]).unwrap().name, "Child");
    }

    #[test]
    fn test_merge_down() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer1 = tree.add_layer(root, "Bottom").unwrap();
        let layer2 = tree.add_layer(root, "Top").unwrap();
        
        let obj = ObjectId::new();
        tree.add_object(layer2, obj).unwrap();
        
        let merged = tree.merge_down(layer2).unwrap();
        assert_eq!(merged, layer1);
        assert!(tree.get_layer(layer1).unwrap().object_ids.contains(&obj));
        assert!(tree.get_layer(layer2).is_none());
    }

    #[test]
    fn test_render_order() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer1 = tree.add_layer(root, "Layer 1").unwrap();
        let layer2 = tree.add_layer(root, "Layer 2").unwrap();
        let layer3 = tree.add_layer(root, "Layer 3").unwrap();
        
        let render = tree.render_layers();
        assert_eq!(render.len(), 4); // root + 3 layers
        // Root is first (bottom), then layers in order added
        assert_eq!(render[0].id, root);
        assert_eq!(render[1].id, layer1);
        assert_eq!(render[2].id, layer2);
        assert_eq!(render[3].id, layer3);
    }

    #[test]
    fn test_flatten_visible() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer1 = tree.add_layer(root, "Layer 1").unwrap();
        let layer2 = tree.add_layer(root, "Layer 2").unwrap();
        let layer3 = tree.add_layer(root, "Layer 3").unwrap();
        
        tree.set_visibility(layer2, false).unwrap(); // Hidden
        
        let obj1 = ObjectId::new();
        let obj3 = ObjectId::new();
        tree.add_object(layer1, obj1).unwrap();
        tree.add_object(layer3, obj3).unwrap();
        
        let flat_id = tree.flatten_visible("Flattened").unwrap();
        
        let flat = tree.get_layer(flat_id).unwrap();
        assert_eq!(flat.object_ids.len(), 2);
        assert!(flat.object_ids.contains(&obj1));
        assert!(flat.object_ids.contains(&obj3));
        
        // Original visible layers should be hidden
        assert!(!tree.get_node(layer1).unwrap().effectively_visible);
        assert!(!tree.get_node(layer3).unwrap().effectively_visible);
        // Hidden layer should stay hidden
        assert!(!tree.get_node(layer2).unwrap().effectively_visible);
    }

    #[test]
    fn test_layer_operations_reverse() {
        let op = LayerOperation::SetVisibility {
            id: LayerId::new(),
            old: true,
            new: false,
        };
        let reversed = reverse_layer_operation(&op);
        
        if let LayerOperation::SetVisibility { old, new, .. } = reversed {
            assert_eq!(old, false);
            assert_eq!(new, true);
        } else {
            panic!("Wrong operation type");
        }
    }

    #[test]
    fn test_apply_layer_operation() {
        let mut tree = LayerTree::new();
        let root = tree.root_id();
        
        let layer = Layer::new("Test Layer");
        let op = LayerOperation::AddLayer {
            parent: root,
            layer: layer.clone(),
        };
        
        apply_layer_operation(&mut tree, &op).unwrap();
        assert!(tree.get_layer(layer.id).is_some());
    }
}